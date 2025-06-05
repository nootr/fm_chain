use actix_files::NamedFile;
use actix_web::{HttpResponse, Responder, get, post, web};
use serde::Deserialize;

use crate::cube::parse_moves;
use crate::messages::FlashMessage;
use crate::models::Block;
use crate::utils::{
    calculate_hash, cleanup_scramble, format_data, format_moves, scramble_from_hash,
    verify_solution,
};
use crate::views;

#[get("/favicon.ico")]
async fn favicon() -> impl Responder {
    NamedFile::open_async("static/favicon.ico").await
}

#[get("/")]
async fn get_index() -> impl Responder {
    HttpResponse::Ok().body(views::get_index())
}

#[derive(Deserialize)]
struct InitialBlockInfo {
    parent_hash: String,
    message: String,
}

#[get("/block")]
async fn get_block(block_info: web::Query<InitialBlockInfo>) -> impl Responder {
    let data = format_data(block_info.parent_hash.clone(), block_info.message.clone());
    let hash = calculate_hash(&data);
    let mut raw_scramble = scramble_from_hash(&hash);
    cleanup_scramble(&mut raw_scramble);
    let scramble = match block_info.message.len() {
        0 => None,
        _ => Some(format_moves(&raw_scramble)),
    };

    HttpResponse::Ok().body(views::get_block(
        &block_info.parent_hash,
        &block_info.message,
        scramble.as_deref(),
        &hash,
        "",
        "",
    ))
}

#[derive(Debug, Deserialize)]
struct CompleteBlockInfo {
    parent_hash: String,
    message: String,
    solution: String,
    solution_description: String,
}

#[post("/block")]
async fn post_block(
    db: web::Data<sqlx::SqlitePool>,
    block_info: web::Form<CompleteBlockInfo>,
) -> impl Responder {
    let data = format_data(block_info.parent_hash.clone(), block_info.message.clone());
    let hash = calculate_hash(&data);
    let mut raw_scramble = scramble_from_hash(&hash);
    cleanup_scramble(&mut raw_scramble);
    let scramble = format_moves(&raw_scramble);
    let parent_block = match Block::find_by_hash(&db, &block_info.parent_hash).await {
        Ok(block) => block,
        Err(_) => {
            let resp = HttpResponse::Ok().body(views::get_block(
                &block_info.parent_hash,
                &block_info.message,
                Some(&scramble),
                &hash,
                &block_info.solution,
                &block_info.solution_description,
            ));
            return FlashMessage::error("Parent block not found").set(resp);
        }
    };

    let data = format_data(parent_block.hash.clone(), block_info.message.clone());
    let hash = calculate_hash(&data);
    let raw_scramble = scramble_from_hash(&hash);
    let parsed_solution = parse_moves(&block_info.solution);

    if !verify_solution(&raw_scramble, &parsed_solution) {
        let resp = HttpResponse::Ok().body(views::get_block(
            &block_info.parent_hash,
            &block_info.message,
            Some(&scramble),
            &hash,
            &block_info.solution,
            &block_info.solution_description,
        ));
        return FlashMessage::error(
            "I'm sorry, but your solution doesn't seem to be correct. Please double-check it!",
        )
        .set(resp);
    }

    if parent_block
        .create_child(
            &db,
            &hash,
            &block_info.message,
            &format_moves(&raw_scramble),
            parsed_solution.len() as u8,
            &block_info.solution_description,
        )
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError()
            .body("Failed to create block. Please try again later.");
    };

    let response = HttpResponse::Ok().body("");
    FlashMessage::info("Block created successfully. Thank you!").set(response)
}

#[derive(Debug, Deserialize)]
pub struct BlockQueryParams {
    pub all: Option<bool>,
}

#[get("/blocks")]
async fn get_blocks(
    db: web::Data<sqlx::SqlitePool>,
    query_params: web::Query<BlockQueryParams>,
) -> impl Responder {
    let blocks = if query_params.all.unwrap_or(false) {
        Block::find_all(&db)
            .await
            .expect("Unable to fetch all blocks")
    } else {
        Block::find_longest_chain(&db)
            .await
            .expect("Unable to fetch longest chain")
    };
    HttpResponse::Ok().body(views::get_blocks(blocks))
}
