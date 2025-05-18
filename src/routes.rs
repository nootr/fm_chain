use actix_files::NamedFile;
use actix_web::{HttpResponse, Responder, get, post, web};
use serde::Deserialize;

use crate::cube::parse_moves;
use crate::models::Block;
use crate::utils::{
    calculate_hash, cleanup_scramble, format_data, format_scramble, scramble_from_hash,
    verify_solution,
};
use crate::views;

#[get("/favicon.ico")]
async fn favicon() -> impl Responder {
    NamedFile::open_async("static/favicon.ico").await
}

#[get("/")]
async fn get_index(db: web::Data<sqlx::SqlitePool>) -> impl Responder {
    let blocks = Block::find_all(&db).await.expect("Unable to fetch blocks");
    HttpResponse::Ok().body(views::get_index(blocks))
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
        _ => Some(format_scramble(&raw_scramble)),
    };

    HttpResponse::Ok().body(views::get_block(
        &block_info.parent_hash,
        &block_info.message,
        scramble.as_deref(),
        &hash,
        "",
        "",
        None,
        None,
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
    let scramble = format_scramble(&raw_scramble);
    let parent_block = match Block::find_by_hash(&db, &block_info.parent_hash).await {
        Ok(block) => block,
        Err(_) => {
            return HttpResponse::Ok().body(views::get_block(
                &block_info.parent_hash,
                &block_info.message,
                Some(&scramble),
                &hash,
                &block_info.solution,
                &block_info.solution_description,
                Some("Parent block not found"),
                None,
            ));
        }
    };

    let data = format_data(parent_block.hash.clone(), block_info.message.clone());
    let hash = calculate_hash(&data);
    let raw_scramble = scramble_from_hash(&hash);
    let parsed_solution = parse_moves(&block_info.solution);

    if !verify_solution(&raw_scramble, &parsed_solution) {
        return HttpResponse::Ok().body(views::get_block(
            &block_info.parent_hash,
            &block_info.message,
            Some(&scramble),
            &hash,
            &block_info.solution,
            &block_info.solution_description,
            Some("Invalid solution"),
            None,
        ));
    }

    let block = match parent_block
        .create_child(
            &db,
            &hash,
            &block_info.message,
            &block_info.solution,
            parsed_solution.len() as u8,
            &block_info.solution_description,
        )
        .await
    {
        Ok(block) => block,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .body("Failed to create block. Please try again later.");
        }
    };

    HttpResponse::Ok().body(views::get_block(
        &block.hash,
        "",
        None,
        "",
        "",
        "",
        None,
        Some("Block created successfully!"),
    ))
}

#[get("/blocks")]
async fn get_blocks(db: web::Data<sqlx::SqlitePool>) -> impl Responder {
    let blocks = Block::find_all(&db).await.expect("Unable to fetch blocks");
    HttpResponse::Ok().body(views::get_blocks(blocks))
}
