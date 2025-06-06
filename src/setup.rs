use env_logger::Env;
use sqlx::SqlitePool;

use fm_chain::config;
use fm_chain::models::Block;
use fm_chain::utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let conf = config::Config::from_env();
    let db = SqlitePool::connect(&conf.database_url)
        .await
        .expect("DB failed");

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let blocks = Block::find_all(&db, None, None)
        .await
        .expect("Unable to fetch blocks");

    if !blocks.is_empty() {
        println!("Genesis block already created! Nothing to do here..");
        return Ok(());
    }

    let message = "✨ Hi, Pip!";
    let data = utils::format_data("", message);
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
        &db,
        &hash,
        message,
        &utils::format_moves(&solution),
        solution.len() as u8,
        solution_description,
    )
    .await
    .expect("Unable to create genesis block");

    db.close().await;

    println!("Created genesis block: {:?}", &block);

    Ok(())
}
