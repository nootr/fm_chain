use chrono::NaiveDateTime;
use sqlx::{FromRow, SqlitePool};
use std::collections::HashMap;

#[derive(Debug, Clone, FromRow)]
pub struct Block {
    pub hash: String,
    pub parent_hash: Option<String>,
    pub height: i64,
    pub message: String,
    pub solution: String,
    pub solution_moves: u8,
    pub solution_description: String,
    pub created_at: Option<NaiveDateTime>,
}

impl Block {
    // Fetch all blocks
    pub async fn find_all(
        db: &SqlitePool,
        page_size: Option<u32>,
        page_offset: Option<u32>,
    ) -> Result<Vec<Block>, sqlx::Error> {
        let mut query_str = String::from(
            "SELECT hash, parent_hash, height, message, solution, solution_moves, solution_description, created_at
             FROM blocks
             ORDER BY
                 height DESC,
                 solution_moves DESC",
        );

        // Conditionally add LIMIT and OFFSET clauses
        if page_size.is_some() {
            query_str.push_str(" LIMIT ?");
        }
        if page_offset.is_some() {
            query_str.push_str(" OFFSET ?");
        }

        let mut query = sqlx::query_as::<_, Block>(&query_str);

        if let Some(size) = page_size {
            query = query.bind(size);
        }
        if let Some(offset) = page_offset {
            query = query.bind(offset);
        }

        query.fetch_all(db).await
    }

    // Fetch a block by hash
    pub async fn find_by_hash(db: &SqlitePool, hash: &str) -> Result<Block, sqlx::Error> {
        sqlx::query_as::<_, Block>(
            "SELECT hash, parent_hash, height, message, solution, solution_moves, solution_description, created_at
             FROM blocks
             WHERE hash = ?",
        )
        .bind(hash)
        .fetch_one(db)
        .await
    }

    // Create a child block
    pub async fn create_child(
        &self,
        db: &SqlitePool,
        hash: &str,
        message: &str,
        solution: &str,
        solution_moves: u8,
        solution_description: &str,
    ) -> Result<Self, sqlx::Error> {
        let block = sqlx::query_as::<_, Block>(
            "INSERT INTO blocks (
                hash, parent_hash, height, message, solution, solution_moves, solution_description
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            RETURNING hash, parent_hash, height, message, solution, solution_moves, solution_description, created_at",
        )
        .bind(hash)
        .bind(&self.hash)
        .bind(self.height + 1)
        .bind(message)
        .bind(solution)
        .bind(solution_moves)
        .bind(solution_description)
        .fetch_one(db)
        .await?;

        Ok(block)
    }

    pub fn short_hash(&self) -> String {
        self.hash.chars().take(8).collect()
    }

    // Public function to find the longest chain
    pub async fn find_longest_chain(
        db: &SqlitePool,
        _page_size: u32,
        _page_offset: u32,
    ) -> Result<Vec<Block>, sqlx::Error> {
        let all_blocks = Self::find_all(db, None, None)
            .await
            .expect("Should fetch all blocks");
        // TODO: pagination
        Self::get_longest_chain_from_blocks(all_blocks)
    }

    // Helper function to process blocks and find the longest chain
    fn get_longest_chain_from_blocks(all_blocks: Vec<Block>) -> Result<Vec<Block>, sqlx::Error> {
        let children_map = Self::build_children_map(&all_blocks);
        let root_blocks = Self::find_root_blocks(&children_map);

        // Assumption: only one root block exists
        if root_blocks.len() != 1 {
            return Ok(Vec::new()); // Or handle this error scenario as appropriate for your application
        }
        let root_block = root_blocks[0].clone(); // Clone to move into stack

        let mut longest_chain: Vec<Block> = Vec::new();
        let mut max_length = 0;
        let mut min_solution_moves = u64::MAX;

        let mut stack: Vec<(Block, Vec<Block>, u64)> = vec![(
            root_block.clone(),
            vec![root_block.clone()],
            root_block.solution_moves as u64,
        )];

        while let Some((current_block, chain_so_far, total_moves_so_far)) = stack.pop() {
            let children: &[Block] =
                if let Some(blocks) = children_map.get(&Some(current_block.hash.clone())) {
                    blocks
                } else {
                    &[]
                };

            if children.is_empty() {
                let chain_length = chain_so_far.len();
                if chain_length > max_length
                    || (chain_length == max_length && total_moves_so_far < min_solution_moves)
                {
                    max_length = chain_length;
                    min_solution_moves = total_moves_so_far;
                    longest_chain = chain_so_far;
                }
            } else {
                for child in children {
                    let mut new_chain_so_far = chain_so_far.clone();
                    new_chain_so_far.push(child.clone());
                    stack.push((
                        child.clone(),
                        new_chain_so_far,
                        total_moves_so_far + child.solution_moves as u64,
                    ));
                }
            }
        }

        // Reverse the chain to have the newest block first and the root block last
        longest_chain.reverse();

        Ok(longest_chain)
    }

    // Helper function to build the children map
    fn build_children_map(all_blocks: &[Block]) -> HashMap<Option<String>, Vec<Block>> {
        let mut children_map: HashMap<Option<String>, Vec<Block>> = HashMap::new();
        for block in all_blocks {
            children_map
                .entry(block.parent_hash.clone())
                .or_default()
                .push(block.clone());
        }
        children_map
    }

    // Helper function to find root blocks
    fn find_root_blocks(children_map: &HashMap<Option<String>, Vec<Block>>) -> Vec<Block> {
        children_map.get(&None).unwrap_or(&vec![]).clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_dummy_block(
        hash: &str,
        parent_hash: Option<&str>,
        height: i64,
        solution_moves: u8,
    ) -> Block {
        Block {
            hash: hash.to_string(),
            parent_hash: parent_hash.map(|s| s.to_string()),
            height,
            message: "test".to_string(),
            solution: "U".to_string(),
            solution_moves,
            solution_description: "test".to_string(),
            created_at: Some(Utc::now().naive_utc()),
        }
    }

    #[test]
    fn test_build_children_map() {
        let block1 = create_dummy_block("hash1", None, 0, 1);
        let block2 = create_dummy_block("hash2", Some("hash1"), 1, 2);
        let block3 = create_dummy_block("hash3", Some("hash1"), 1, 3);
        let block4 = create_dummy_block("hash4", Some("hash2"), 2, 1);

        let all_blocks = vec![
            block1.clone(),
            block2.clone(),
            block3.clone(),
            block4.clone(),
        ];
        let children_map = Block::build_children_map(&all_blocks);

        assert!(children_map.get(&None).is_some());
        assert_eq!(children_map.get(&None).unwrap().len(), 1);
        assert_eq!(children_map.get(&None).unwrap()[0].hash, "hash1");

        assert!(children_map.get(&Some("hash1".to_string())).is_some());
        assert_eq!(
            children_map.get(&Some("hash1".to_string())).unwrap().len(),
            2
        );
        let hash1_children_hashes: Vec<String> = children_map
            .get(&Some("hash1".to_string()))
            .unwrap()
            .iter()
            .map(|b| b.hash.clone())
            .collect();
        assert!(hash1_children_hashes.contains(&"hash2".to_string()));
        assert!(hash1_children_hashes.contains(&"hash3".to_string()));

        assert!(children_map.get(&Some("hash2".to_string())).is_some());
        assert_eq!(
            children_map.get(&Some("hash2".to_string())).unwrap().len(),
            1
        );
        assert_eq!(
            children_map.get(&Some("hash2".to_string())).unwrap()[0].hash,
            "hash4"
        );
    }

    #[test]
    fn test_find_root_blocks() {
        let block1 = create_dummy_block("hash1", None, 0, 1);
        let block2 = create_dummy_block("hash2", Some("hash1"), 1, 2);
        let block3 = create_dummy_block("hash3", None, 0, 3);

        let all_blocks = vec![block1.clone(), block2.clone(), block3.clone()];
        let children_map = Block::build_children_map(&all_blocks);
        let root_blocks = Block::find_root_blocks(&children_map);

        assert_eq!(root_blocks.len(), 2);
        let root_hashes: Vec<String> = root_blocks.iter().map(|b| b.hash.clone()).collect();
        assert!(root_hashes.contains(&"hash1".to_string()));
        assert!(root_hashes.contains(&"hash3".to_string()));
    }

    #[test]
    fn test_get_longest_chain_from_blocks_single_chain() {
        let block1 = create_dummy_block("hash1", None, 0, 1);
        let block2 = create_dummy_block("hash2", Some("hash1"), 1, 2);
        let block3 = create_dummy_block("hash3", Some("hash2"), 2, 3);

        let all_blocks = vec![block1.clone(), block2.clone(), block3.clone()];
        let longest_chain = Block::get_longest_chain_from_blocks(all_blocks).unwrap();

        assert_eq!(longest_chain.len(), 3);
        assert_eq!(longest_chain[0].hash, "hash3"); // Newest first
        assert_eq!(longest_chain[1].hash, "hash2");
        assert_eq!(longest_chain[2].hash, "hash1"); // Root last
    }

    #[test]
    fn test_get_longest_chain_from_blocks_forked_chain() {
        let block1 = create_dummy_block("hash1", None, 0, 1);
        let block2_a = create_dummy_block("hash2a", Some("hash1"), 1, 2);
        let block3_a = create_dummy_block("hash3a", Some("hash2a"), 2, 1); // Length 3, total moves 4
        let block2_b = create_dummy_block("hash2b", Some("hash1"), 1, 1);
        let block3_b = create_dummy_block("hash3b", Some("hash2b"), 2, 1);
        let block4_b = create_dummy_block("hash4b", Some("hash3b"), 3, 1); // Length 4, total moves 4

        let all_blocks = vec![
            block1.clone(),
            block2_a.clone(),
            block3_a.clone(),
            block2_b.clone(),
            block3_b.clone(),
            block4_b.clone(),
        ];
        let longest_chain = Block::get_longest_chain_from_blocks(all_blocks).unwrap();

        assert_eq!(longest_chain.len(), 4);
        assert_eq!(longest_chain[0].hash, "hash4b"); // Newest first
        assert_eq!(longest_chain[1].hash, "hash3b");
        assert_eq!(longest_chain[2].hash, "hash2b");
        assert_eq!(longest_chain[3].hash, "hash1"); // Root last
    }

    #[test]
    fn test_get_longest_chain_from_blocks_equal_length_shortest_solution_wins() {
        let block1 = create_dummy_block("hash1", None, 0, 1);
        let block2_a = create_dummy_block("hash2a", Some("hash1"), 1, 5);
        let block3_a = create_dummy_block("hash3a", Some("hash2a"), 2, 1); // Chain A: length 3, total moves 1 + 5 + 1 = 7

        let block2_b = create_dummy_block("hash2b", Some("hash1"), 1, 1);
        let block3_b = create_dummy_block("hash3b", Some("hash2b"), 2, 2); // Chain B: length 3, total moves 1 + 1 + 2 = 4

        let all_blocks = vec![
            block1.clone(),
            block2_a.clone(),
            block3_a.clone(),
            block2_b.clone(),
            block3_b.clone(),
        ];
        let longest_chain = Block::get_longest_chain_from_blocks(all_blocks).unwrap();

        assert_eq!(longest_chain.len(), 3);
        assert_eq!(longest_chain[0].hash, "hash3b"); // Newest first
        assert_eq!(longest_chain[1].hash, "hash2b");
        assert_eq!(longest_chain[2].hash, "hash1"); // Root last
    }

    #[test]
    fn test_get_longest_chain_from_blocks_no_blocks() {
        let all_blocks: Vec<Block> = vec![];
        let longest_chain = Block::get_longest_chain_from_blocks(all_blocks).unwrap();
        assert!(longest_chain.is_empty());
    }

    #[test]
    fn test_get_longest_chain_from_blocks_multiple_roots_returns_empty() {
        let block_root1 = create_dummy_block("root1", None, 0, 1);
        let block_c1_r1 = create_dummy_block("c1_r1", Some("root1"), 1, 1);

        let block_root2 = create_dummy_block("root2", None, 0, 1);
        let block_c1_r2 = create_dummy_block("c1_r2", Some("root2"), 1, 1);

        let all_blocks = vec![
            block_root1.clone(),
            block_c1_r1.clone(),
            block_root2.clone(),
            block_c1_r2.clone(),
        ];
        let longest_chain = Block::get_longest_chain_from_blocks(all_blocks).unwrap();

        // Expect empty because there are multiple root blocks, violating the assumption.
        assert!(longest_chain.is_empty());
    }
}
