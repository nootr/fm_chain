use sqlx::SqlitePool;

use crate::models::Block;
use crate::utils;

pub async fn run_setup(db: &SqlitePool) -> std::io::Result<()> {
    sqlx::migrate!("./migrations")
        .run(db)
        .await
        .expect("Failed to run migrations");

    let blocks = Block::find_all(db, false, None, None)
        .await
        .expect("Unable to fetch blocks");

    if !blocks.is_empty() {
        println!("Genesis block already created! Nothing to do here..");
        return Ok(());
    }

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

    let block = Block::create_genesis(
        db,
        &hash,
        name,
        message,
        &utils::format_moves(&solution),
        solution.len() as u8,
        solution_description,
    )
    .await
    .expect("Unable to create genesis block");

    println!("Created genesis block: {:?}", &block);

    Ok(())
}
