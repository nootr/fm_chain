#[cfg(test)]
mod integration_tests {
    use fm_chain::models::Block;
    use fm_chain::utils;
    use sqlx::SqlitePool;
    use std::collections::HashSet;

    #[sqlx::test(fixtures("blocks"))]
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

    #[sqlx::test(fixtures("blocks"))]
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

    #[sqlx::test(fixtures("blocks"))]
    async fn test_find_all_blocks(pool: SqlitePool) {
        let all_blocks = Block::find_all(&pool, false, None, None)
            .await
            .expect("Failed to find all blocks");
        assert_eq!(all_blocks.len(), 7);

        let main_chain_blocks = Block::find_all(&pool, true, None, None)
            .await
            .expect("Failed to find main chain blocks");

        assert_eq!(main_chain_blocks.len(), 4);
        let main_chain_hashes: HashSet<String> = main_chain_blocks.into_iter().map(|b| b.hash).collect();
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

    #[sqlx::test(fixtures("blocks"))]
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

    #[sqlx::test(fixtures("blocks"))]
    async fn test_block_scramble_method(pool: SqlitePool) {
        let block = Block::create_genesis(
            &pool,
            "A0C1E2G3",
            "test",
            "message",
            "U",
            1,
            "desc",
        )
        .await
        .unwrap();

        let expected_scramble_moves = utils::scramble_from_hash("A0C1E2G3");
        let expected_scramble_string = utils::format_moves(&expected_scramble_moves);

        assert_eq!(block.scramble(), expected_scramble_string);
    }

    #[sqlx::test(fixtures("blocks"))]
    async fn test_block_short_hash_method(pool: SqlitePool) {
        let long_hash = "abcdefghijklmnop";
        let block = Block::create_genesis(
            &pool,
            long_hash,
            "test",
            "message",
            "U",
            1,
            "desc",
        )
        .await
        .unwrap();

        assert_eq!(block.short_hash(), "abcdefgh");
    }
}
