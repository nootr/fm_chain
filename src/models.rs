use chrono::{Datelike, NaiveDateTime, Utc, Weekday};
use sqlx::{FromRow, SqlitePool};
use std::collections::HashSet;

use crate::utils;

#[derive(Debug, PartialEq, Eq)]
pub enum BlockTag {
    Genesis,
    New,
    Recommended,
    MainChain,
}

impl BlockTag {
    pub fn label(&self) -> &str {
        match self {
            BlockTag::Genesis => "Genesis",
            BlockTag::New => "New",
            BlockTag::Recommended => "Recommended",
            BlockTag::MainChain => "Main Chain",
        }
    }

    pub fn value(&self) -> &str {
        match self {
            BlockTag::Genesis => "genesis",
            BlockTag::New => "new",
            BlockTag::Recommended => "recommended",
            BlockTag::MainChain => "main_chain",
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct Block {
    pub version: u8,
    pub hash: String,
    pub parent_hash: Option<String>,
    pub height: i64,
    pub name: String,
    pub message: String,
    pub solution: String,
    pub solution_moves: u8,
    pub solution_description: String,
    pub created_at: Option<NaiveDateTime>,
}

impl Block {
    // Check if the block hash and solutions are valid
    pub fn is_valid(&self) -> bool {
        let expected_hash = utils::calculate_hash(&utils::format_data(
            self.parent_hash.as_deref().unwrap_or(""),
            &self.name,
            &self.message,
        ));
        let scramble = utils::parse_moves(&self.scramble());
        let solution = utils::parse_moves(&self.solution);

        utils::verify_solution(&scramble, &solution)
            && self.solution_moves == solution.len() as u8
            && !self.hash.is_empty()
            && self.hash == expected_hash
    }

    // Get scramble for this block
    pub fn scramble(&self) -> String {
        let scramble = match self.version {
            1 => utils::scramble_from_hash_v1(&self.hash),
            2 => utils::scramble_from_hash(&self.hash),
            _ => panic!("Unsupported block version"),
        };
        utils::format_moves(&scramble)
    }

    // Returns true if the user is allowed to create a child block
    pub fn can_create_child(&self, time: Option<NaiveDateTime>) -> bool {
        self.is_from_last_week(time) || self.height == 0
    }

    // Returns the number of recommended blocks
    pub async fn get_recommended_count(db: &SqlitePool) -> Result<usize, sqlx::Error> {
        let blocks = Self::find_all(db, false, None, None)
            .await?
            .into_iter()
            .filter(|b| b.can_create_child(None))
            .collect::<Vec<_>>();

        if blocks.is_empty() {
            return Ok(0);
        }

        let optimal_height: i64 = blocks
            .first()
            .expect("At least one block should be available")
            .height;
        Ok(blocks.iter().filter(|b| b.height == optimal_height).count())
    }

    // Returns a list of tags for this block
    pub fn tags(
        &self,
        time: Option<NaiveDateTime>,
        main_chain_hashes: &HashSet<String>,
        optimal_height: i64,
    ) -> Vec<BlockTag> {
        let mut tags = vec![];

        if !self.is_from_last_week(time) && self.height > 0 {
            tags.push(BlockTag::New);
        }

        if (self.height == optimal_height) && self.can_create_child(time) {
            tags.push(BlockTag::Recommended);
        }

        if main_chain_hashes.contains(&self.hash) {
            tags.push(BlockTag::MainChain);
        }

        if self.height == 0 {
            tags.push(BlockTag::Genesis);
        }

        tags
    }

    // Return true if the solution moves for a given hash already exists
    pub async fn hash_and_solution_exists(
        db: &SqlitePool,
        hash: &str,
        solution: &str,
    ) -> Result<bool, sqlx::Error> {
        let exists: Option<i32> = sqlx::query_scalar!(
            "SELECT EXISTS(
                SELECT 1 FROM blocks
                WHERE hash = ? AND solution = ?
            )",
            hash,
            solution
        )
        .fetch_one(db)
        .await?;

        Ok(matches!(exists, Some(1)))
    }

    // Fetch Block from database using its hash
    pub async fn from_hash(hash: &str) -> Self {
        sqlx::query_as::<_, Block>(
            "SELECT hash, parent_hash, height, name, message, solution, solution_moves, solution_description, created_at
             FROM blocks
             WHERE hash = ?",
        )
        .bind(hash)
        .fetch_one(&SqlitePool::connect("your_database_url").await.unwrap())
        .await
        .expect("Failed to fetch block by hash")
    }

    // Returns true if the block is from before Monday this week
    fn is_from_last_week(&self, time: Option<NaiveDateTime>) -> bool {
        let created_at = self.created_at.expect("Block should have a creation date");
        let now = time.unwrap_or_else(|| Utc::now().naive_utc());
        let today = now.date();
        let days_since_monday = match today.weekday() {
            Weekday::Mon => 0,
            Weekday::Tue => 1,
            Weekday::Wed => 2,
            Weekday::Thu => 3,
            Weekday::Fri => 4,
            Weekday::Sat => 5,
            Weekday::Sun => 6,
        };

        let start_of_week_date = today - chrono::Duration::days(days_since_monday);
        let start_of_week = start_of_week_date
            .and_hms_opt(0, 0, 0)
            .expect("Failed to get start of week");

        created_at < start_of_week
    }

    // Get a list of hashes of blocks in the main chain
    pub async fn get_main_chain_hashes(db: &SqlitePool) -> Result<HashSet<String>, sqlx::Error> {
        Ok(sqlx::query_scalar!(
            r#"
            WITH RECURSIVE main_chain AS (
              SELECT hash, parent_hash, height, solution_moves
              FROM blocks
              WHERE hash = (
                SELECT hash
                FROM blocks
                ORDER BY height DESC, solution_moves ASC
                LIMIT 1
              )

              UNION ALL

              SELECT b.hash, b.parent_hash, b.height, b.solution_moves
              FROM blocks b
              INNER JOIN main_chain mc ON b.hash = mc.parent_hash
            )
            SELECT hash FROM main_chain
            "#
        )
        .fetch_all(db)
        .await?
        .into_iter()
        .flatten()
        .collect())
    }

    // Fetch all blocks
    pub async fn find_all(
        db: &SqlitePool,
        main_chain_only: bool,
        page_size: Option<u32>,
        page_offset: Option<u32>,
    ) -> Result<Vec<Block>, sqlx::Error> {
        let mut query_str = String::from(
            "SELECT version, hash, parent_hash, height, name, message, solution, solution_moves, solution_description, created_at
             FROM blocks"
        );

        if main_chain_only {
            let hashes = Self::get_main_chain_hashes(db).await?;
            if hashes.is_empty() {
                return Ok(Vec::new());
            }
            query_str.push_str(" WHERE hash IN (");
            query_str.push_str(
                &hashes
                    .iter()
                    .map(|h| format!("'{}'", h))
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            query_str.push(')');
        }

        query_str.push_str(" ORDER BY height DESC, solution_moves ASC");

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
            "SELECT version, hash, parent_hash, height, name, message, solution, solution_moves, solution_description, created_at
             FROM blocks
             WHERE hash = ?",
        )
        .bind(hash)
        .fetch_one(db)
        .await
    }

    // Create a genesis block
    pub async fn create_genesis(
        db: &SqlitePool,
        hash: &str,
        name: &str,
        message: &str,
        solution: &str,
        solution_moves: u8,
        solution_description: &str,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Block>(
            "INSERT INTO blocks (
                hash, height, name, message, solution, solution_moves, solution_description
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            RETURNING version, hash, parent_hash, height, name, message, solution, solution_moves, solution_description, created_at",
        )
        .bind(hash)
        .bind(0)
        .bind(name)
        .bind(message)
        .bind(solution)
        .bind(solution_moves)
        .bind(solution_description)
        .fetch_one(db)
        .await
    }

    // Create a child block
    #[allow(clippy::too_many_arguments)]
    pub async fn create_child(
        &self,
        db: &SqlitePool,
        hash: &str,
        name: &str,
        message: &str,
        solution: &str,
        solution_moves: u8,
        solution_description: &str,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Block>(
            "INSERT INTO blocks (
                hash, parent_hash, height, name, message, solution, solution_moves, solution_description
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING version, hash, parent_hash, height, name, message, solution, solution_moves, solution_description, created_at",
        )
        .bind(hash)
        .bind(&self.hash)
        .bind(self.height + 1)
        .bind(name)
        .bind(message)
        .bind(solution)
        .bind(solution_moves)
        .bind(solution_description)
        .fetch_one(db)
        .await
    }

    pub fn short_hash(&self) -> String {
        self.hash.chars().take(8).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, NaiveDateTime};
    use sqlx::SqlitePool;
    use std::collections::HashSet;

    #[sqlx::test(fixtures("../fixtures/blocks.sql"))]
    async fn test_create_and_find_genesis_block(pool: SqlitePool) {
        let hash = "new_genesis_hash";
        let name = "test_user_new";
        let message = "Hello, world!";
        let solution = "U D L R F B";
        let solution_moves = 6;
        let solution_description = "Simple solution";

        let genesis_block = Block::create_genesis(
            &pool,
            hash,
            name,
            message,
            solution,
            solution_moves,
            solution_description,
        )
        .await
        .expect("Failed to create genesis block");

        assert_eq!(genesis_block.hash, hash);
        assert_eq!(genesis_block.parent_hash, None);
        assert_eq!(genesis_block.height, 0);
        assert_eq!(genesis_block.name, name);
        assert_eq!(genesis_block.message, message);
        assert_eq!(genesis_block.solution, solution);
        assert_eq!(genesis_block.solution_moves, solution_moves);
        assert_eq!(genesis_block.solution_description, solution_description);
        assert!(genesis_block.created_at.is_some());

        let found_block = Block::find_by_hash(&pool, hash)
            .await
            .expect("Failed to find genesis block by hash");

        assert_eq!(genesis_block.hash, found_block.hash);
        assert_eq!(genesis_block.height, found_block.height);
    }

    #[sqlx::test(fixtures("../fixtures/blocks.sql"))]
    async fn test_create_child_block(pool: SqlitePool) {
        let parent_block = Block::create_genesis(
            &pool,
            "parent_test_hash",
            "parent_user",
            "Parent message",
            "U",
            1,
            "Parent desc",
        )
        .await
        .expect("Failed to create parent block for child test");

        let child_hash = "child_test_hash";
        let child_name = "child_user";
        let child_message = "Child message";
        let child_solution = "D";
        let child_solution_moves = 1;
        let child_solution_description = "Child desc";

        let child_block = parent_block
            .create_child(
                &pool,
                child_hash,
                child_name,
                child_message,
                child_solution,
                child_solution_moves,
                child_solution_description,
            )
            .await
            .expect("Failed to create child block");

        assert_eq!(child_block.hash, child_hash);
        assert_eq!(child_block.parent_hash, Some(parent_block.hash));
        assert_eq!(child_block.height, parent_block.height + 1);
        assert_eq!(child_block.name, child_name);

        let found_child_block = Block::find_by_hash(&pool, child_hash)
            .await
            .expect("Failed to find child block by hash");
        assert_eq!(child_block.hash, found_child_block.hash);
    }

    #[sqlx::test(fixtures("../fixtures/blocks.sql"))]
    async fn test_find_all_blocks(pool: SqlitePool) {
        let all_blocks = Block::find_all(&pool, false, None, None)
            .await
            .expect("Failed to find all blocks");
        assert_eq!(all_blocks.len(), 7);

        let main_chain_blocks = Block::find_all(&pool, true, None, None)
            .await
            .expect("Failed to find main chain blocks");

        assert_eq!(main_chain_blocks.len(), 4);
        let main_chain_hashes: HashSet<String> =
            main_chain_blocks.into_iter().map(|b| b.hash).collect();
        assert!(main_chain_hashes.contains("genesis_block_hash_001"));
        assert!(main_chain_hashes.contains("main_chain_block_002"));
        assert!(main_chain_hashes.contains("main_chain_block_003"));
        assert!(main_chain_hashes.contains("main_chain_block_004"));
        assert!(!main_chain_hashes.contains("fork_chain_block_A_001"));
        assert!(!main_chain_hashes.contains("fork_chain_block_A_002"));
        assert!(!main_chain_hashes.contains("fork_chain_block_B_001"));

        let paginated_blocks_page1 = Block::find_all(&pool, false, Some(2), Some(0))
            .await
            .expect("Failed to paginate blocks (page 1)");
        assert_eq!(paginated_blocks_page1.len(), 2);
        assert_eq!(paginated_blocks_page1[0].hash, "main_chain_block_004");
        assert_eq!(paginated_blocks_page1[1].hash, "fork_chain_block_A_002");

        let paginated_blocks_page2 = Block::find_all(&pool, false, Some(2), Some(2))
            .await
            .expect("Failed to paginate blocks (page 2)");
        assert_eq!(paginated_blocks_page2.len(), 2);
        assert_eq!(paginated_blocks_page2[0].hash, "main_chain_block_003");
        assert_eq!(paginated_blocks_page2[1].hash, "fork_chain_block_A_001");

        let paginated_blocks_page3 = Block::find_all(&pool, false, Some(2), Some(4))
            .await
            .expect("Failed to paginate blocks (page 3)");
        assert_eq!(paginated_blocks_page3.len(), 2);
        assert_eq!(paginated_blocks_page3[0].hash, "main_chain_block_002");
        assert_eq!(paginated_blocks_page3[1].hash, "fork_chain_block_B_001");

        let paginated_blocks_page4 = Block::find_all(&pool, false, Some(2), Some(6))
            .await
            .expect("Failed to paginate blocks (page 4)");
        assert_eq!(paginated_blocks_page4.len(), 1);
        assert_eq!(paginated_blocks_page4[0].hash, "genesis_block_hash_001");
    }

    #[sqlx::test(fixtures("../fixtures/blocks.sql"))]
    async fn test_get_main_chain_hashes(pool: SqlitePool) {
        let hashes = Block::get_main_chain_hashes(&pool).await.unwrap();

        assert_eq!(hashes.len(), 4);
        assert!(hashes.contains("genesis_block_hash_001"));
        assert!(hashes.contains("main_chain_block_002"));
        assert!(hashes.contains("main_chain_block_003"));
        assert!(hashes.contains("main_chain_block_004"));
        assert!(!hashes.contains("fork_chain_block_A_001"));
        assert!(!hashes.contains("fork_chain_block_A_002"));
        assert!(!hashes.contains("fork_chain_block_B_001"));
    }

    #[sqlx::test(fixtures("../fixtures/blocks.sql"))]
    async fn test_block_scramble_method(pool: SqlitePool) {
        let block = Block::create_genesis(&pool, "A0C1E2G3", "test", "message", "U", 1, "desc")
            .await
            .unwrap();

        let expected_scramble_moves = utils::scramble_from_hash("A0C1E2G3");
        let expected_scramble_string = utils::format_moves(&expected_scramble_moves);

        assert_eq!(block.scramble(), expected_scramble_string);
    }

    #[sqlx::test(fixtures("../fixtures/blocks.sql"))]
    async fn test_block_short_hash_method(pool: SqlitePool) {
        let long_hash = "abcdefghijklmnop";
        let block = Block::create_genesis(&pool, long_hash, "test", "message", "U", 1, "desc")
            .await
            .unwrap();

        assert_eq!(block.short_hash(), "abcdefgh");
    }

    #[sqlx::test]
    async fn test_same_solution_moves(pool: SqlitePool) {
        // Regression test for issue where same solution moves were incorrectly both present in the
        // main chain hashes.
        let root = Block::create_genesis(
            &pool,
            "A0C1E2G3",
            "test",
            "message",
            "U D L R F B",
            6,
            "desc",
        )
        .await
        .unwrap();
        let _ = root
            .create_child(
                &pool,
                "A0C1E2G4",
                "test_a",
                "message_a",
                "U D L R F B",
                6,
                "desc_a",
            )
            .await;
        let _ = root
            .create_child(
                &pool,
                "A0C1E2G5",
                "test_b",
                "message_b",
                "U D L R F B",
                6,
                "desc_b",
            )
            .await;
        let main_chain_hashes = Block::get_main_chain_hashes(&pool)
            .await
            .expect("Failed to get main chain hashes");

        assert!(main_chain_hashes.contains(&root.hash));
        assert_eq!(main_chain_hashes.len(), 2);
    }

    #[test]
    fn test_tags() {
        // TODO: update test
        let current_test_time =
            NaiveDateTime::parse_from_str("2024-09-19 18:45:00", "%Y-%m-%d %H:%M:%S")
                .expect("Failed to parse test time");

        let genesis_block = Block {
            version: 2,
            hash: "genesis_hash".to_string(),
            parent_hash: None,
            height: 0,
            name: "Genesis Block".to_string(),
            message: "Initial block".to_string(),
            solution: "".to_string(),
            solution_moves: 0,
            solution_description: "".to_string(),
            created_at: Some(current_test_time),
        };

        let block_a = Block {
            version: 2,
            hash: "block_a_hash".to_string(),
            parent_hash: Some("genesis_hash".to_string()),
            height: 1,
            name: "Block A".to_string(),
            message: "Child of Genesis".to_string(),
            solution: "A".to_string(),
            solution_moves: 5,
            solution_description: "Solution A".to_string(),
            created_at: Some(current_test_time - Duration::minutes(1)),
        };

        let block_b = Block {
            version: 2,
            hash: "block_b_hash".to_string(),
            parent_hash: Some("genesis_hash".to_string()),
            height: 1,
            name: "Block B".to_string(),
            message: "Another Child of Genesis".to_string(),
            solution: "B".to_string(),
            solution_moves: 8,
            solution_description: "Solution B".to_string(),
            created_at: Some(current_test_time - Duration::hours(1)),
        };

        let block_c = Block {
            version: 2,
            hash: "block_c_hash".to_string(),
            parent_hash: Some("genesis_hash".to_string()),
            height: 1,
            name: "Block C".to_string(),
            message: "Old Child of Genesis".to_string(),
            solution: "C".to_string(),
            solution_moves: 5,
            solution_description: "Solution C".to_string(),
            created_at: Some(current_test_time - Duration::weeks(2)),
        };

        let block_d = Block {
            version: 2,
            hash: "block_d_hash".to_string(),
            parent_hash: Some("block_a_hash".to_string()),
            height: 2,
            name: "Block D".to_string(),
            message: "Child of Block A".to_string(),
            solution: "D".to_string(),
            solution_moves: 3,
            solution_description: "Solution D".to_string(),
            created_at: Some(current_test_time - Duration::minutes(30)),
        };
        let optimal_height = block_a.height;

        let mut blocks = vec![
            block_a.clone(),
            block_d.clone(),
            genesis_block.clone(),
            block_b.clone(),
            block_c.clone(),
        ];

        // Sort by height DESC and then by solution_moves ASC
        blocks.sort_by(|a, b| {
            if a.height == b.height {
                a.solution_moves.cmp(&b.solution_moves)
            } else {
                b.height.cmp(&a.height)
            }
        });

        let mut main_chain_hashes = HashSet::new();
        main_chain_hashes.insert(genesis_block.hash.clone());
        main_chain_hashes.insert(block_a.hash.clone());
        main_chain_hashes.insert(block_d.hash.clone());

        let actual_tags_a = blocks
            .iter()
            .find(|b| b.hash == block_a.hash)
            .expect("Block A should be present")
            .tags(Some(current_test_time), &main_chain_hashes, optimal_height);
        let expected_tags_a = vec![BlockTag::New, BlockTag::MainChain];
        assert_eq!(actual_tags_a, expected_tags_a, "Tags mismatch for Block A");

        let actual_tags_d = blocks
            .iter()
            .find(|b| b.hash == block_d.hash)
            .expect("Block D should be present")
            .tags(Some(current_test_time), &main_chain_hashes, optimal_height);
        let expected_tags_d = vec![BlockTag::New, BlockTag::MainChain];
        assert_eq!(actual_tags_d, expected_tags_d, "Tags mismatch for Block D");

        let actual_tags_genesis = blocks
            .iter()
            .find(|b| b.hash == genesis_block.hash)
            .expect("Genesis block should be present")
            .tags(Some(current_test_time), &main_chain_hashes, optimal_height);
        let expected_tags_genesis = vec![BlockTag::MainChain, BlockTag::Genesis];
        assert_eq!(
            actual_tags_genesis, expected_tags_genesis,
            "Tags mismatch for Genesis Block"
        );

        let actual_tags_b = blocks
            .iter()
            .find(|b| b.hash == block_b.hash)
            .expect("Block B should be present")
            .tags(Some(current_test_time), &main_chain_hashes, optimal_height);
        let expected_tags_b = vec![BlockTag::New];
        assert_eq!(actual_tags_b, expected_tags_b, "Tags mismatch for Block B");

        let actual_tags_c = blocks
            .iter()
            .find(|b| b.hash == block_c.hash)
            .expect("Block C should be present")
            .tags(Some(current_test_time), &main_chain_hashes, optimal_height);
        let expected_tags_c = vec![BlockTag::Recommended];
        assert_eq!(actual_tags_c, expected_tags_c, "Tags mismatch for Block C");
    }

    #[sqlx::test]
    async fn test_duplicate_solutions(pool: SqlitePool) {
        let hash = "duplicate_solution_hash";
        let solution = "U D L R F B";

        let exists = Block::hash_and_solution_exists(&pool, &hash, &solution)
            .await
            .expect("Failed to check if hash and solution exist");

        assert!(
            !exists,
            "Expected the block's hash and solution not to exist"
        );

        let _ = Block::create_genesis(
            &pool,
            hash,
            "test_user",
            "Test message",
            solution,
            6,
            "Test description",
        )
        .await
        .expect("Failed to create genesis block for duplicate solution test");

        let exists = Block::hash_and_solution_exists(&pool, &hash, &solution)
            .await
            .expect("Failed to check if hash and solution exist");

        assert!(exists, "Expected the block's hash and solution to exist");
    }

    #[sqlx::test(fixtures("../fixtures/blocks.sql"))]
    async fn test_recommended_count(pool: SqlitePool) {
        let recommended_count = Block::get_recommended_count(&pool)
            .await
            .expect("Failed to get recommended block count");

        assert_eq!(recommended_count, 2, "Recommended count should be 2");
    }

    #[test]
    fn test_scramble_v1_v2() {
        let mut block = Block {
            version: 1,
            hash: "A0C1E2G3".to_string(),
            parent_hash: None,
            height: 0,
            name: "Test Block".to_string(),
            message: "Test Message".to_string(),
            solution: "U D L R F B".to_string(),
            solution_moves: 6,
            solution_description: "Test Description".to_string(),
            created_at: Some(
                NaiveDateTime::parse_from_str("2024-09-19 18:45:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            ),
        };
        let scramble = block.scramble();

        assert!(
            !scramble.starts_with("R' U' F"),
            "Scramble should not start with R' U' F"
        );
        assert!(
            !scramble.ends_with("R' U' F"),
            "Scramble should not end with R' U' F"
        );

        block.version = 2;

        let scramble = block.scramble();

        assert!(
            scramble.starts_with("R' U' F"),
            "Scramble should start with R' U' F"
        );
        assert!(
            scramble.ends_with("R' U' F"),
            "Scramble should end with R' U' F"
        );
    }

    #[sqlx::test]
    async fn test_new_block_is_v2(pool: SqlitePool) {
        let new_block = Block::create_genesis(
            &pool,
            "new_block_hash",
            "New User",
            "New Message",
            "U D L R F B",
            6,
            "New Description",
        )
        .await
        .expect("Failed to create new block");

        assert_eq!(new_block.version, 2, "New block should be version 2");
    }
}
