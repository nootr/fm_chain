use sqlx::SqlitePool;

use crate::models::Block;
use crate::utils;

async fn create_genesis_block(db: &SqlitePool) -> std::io::Result<()> {
    let name = "Nootr";
    let message = "Let the solves begin! ✨";
    let data = utils::format_data("", name, message);
    let hash = utils::calculate_hash(&data);
    let scramble = utils::scramble_from_hash(&hash);
    let solution = scramble
        .iter()
        .rev()
        .map(|m| m.inverse())
        .collect::<Vec<_>>();
    let solution_description = "Simply took the reverse of the scramble, sorry!";

    assert!(
        utils::verify_solution(&scramble, &solution),
        "Solution should be valid"
    );

    Block::create_genesis(
        db,
        &hash,
        name,
        message,
        &utils::format_moves(&solution),
        solution.len() as u8,
        solution_description,
    )
    .await
    .expect("Failed to create genesis block");

    Ok(())
}

pub async fn run_setup(db: &SqlitePool) -> std::io::Result<()> {
    sqlx::migrate!("./migrations")
        .run(db)
        .await
        .expect("Failed to run migrations");

    let blocks = Block::find_all(db, false, None, None)
        .await
        .expect("Unable to fetch blocks");

    if let Some(invalid_block) = blocks.iter().find(|b| !b.is_valid()) {
        panic!(
            "Invalid block found in database: {:?} at height {}",
            invalid_block.hash, invalid_block.height
        );
    } else {
        println!("No invalid blocks found in database.");
    }

    if blocks.is_empty() {
        create_genesis_block(db).await?;
        println!("Created genesis block");
    }

    Ok(())
}
