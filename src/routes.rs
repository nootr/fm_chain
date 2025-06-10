use actix_files::NamedFile;
use actix_web::{HttpResponse, Responder, get, post, web};
use serde::Deserialize;

use crate::messages::FlashMessage;
use crate::models::Block;
use crate::utils::{
    calculate_hash, cleanup_scramble, format_data, format_moves, parse_moves, scramble_from_hash,
    verify_solution,
};
use crate::views;

#[get("/favicon.ico")]
async fn favicon() -> impl Responder {
    NamedFile::open_async("static/favicon.ico").await
}

#[get("/")]
async fn get_index() -> impl Responder {
    NamedFile::open_async("static/index.html").await
}

#[derive(Deserialize)]
struct InitialBlockInfo {
    parent_hash: Option<String>,
    name: Option<String>,
    message: Option<String>,
}

#[get("/block")]
async fn get_block(
    db: web::Data<sqlx::SqlitePool>,
    block_info: web::Query<InitialBlockInfo>,
) -> impl Responder {
    let main_chain_head_hash = Block::find_main_chain_head(&db)
        .await
        .expect("Unable to find head");
    let parent_hash = match &block_info.parent_hash {
        Some(x) => x.clone(),
        None => main_chain_head_hash.clone(),
    };
    match Block::find_by_hash(&db, &parent_hash).await {
        Ok(block) => {
            if !block.can_create_child() {
                return HttpResponse::BadRequest()
                    .body("This block cannot be used as a parent for a new block.");
            }
        }
        Err(_) => {
            return HttpResponse::NotFound().body("Parent block not found");
        }
    };
    let name = block_info.name.clone().unwrap_or_default();
    let message = block_info.message.clone().unwrap_or_default();
    let data = format_data(&parent_hash, &name, &message);
    let hash = calculate_hash(&data);
    let mut raw_scramble = scramble_from_hash(&hash);
    cleanup_scramble(&mut raw_scramble);
    let scramble = match message.is_empty() || name.is_empty() {
        true => None,
        false => Some(format_moves(&raw_scramble)),
    };

    HttpResponse::Ok().body(views::get_block(
        &parent_hash,
        parent_hash == main_chain_head_hash,
        &name,
        &message,
        scramble.as_deref(),
        &hash,
        "",
        "",
    ))
}

#[derive(Debug, Deserialize)]
struct CompleteBlockInfo {
    parent_hash: String,
    name: String,
    message: String,
    solution: String,
    solution_description: String,
}

#[post("/block")]
async fn post_block(
    db: web::Data<sqlx::SqlitePool>,
    block_info: web::Form<CompleteBlockInfo>,
) -> impl Responder {
    match Block::find_by_hash(&db, &block_info.parent_hash).await {
        Ok(block) => {
            if !block.can_create_child() {
                return HttpResponse::BadRequest()
                    .body("This block cannot be used as a parent for a new block.");
            }
        }
        Err(_) => {
            return HttpResponse::NotFound().body("Parent block not found");
        }
    };
    let main_chain_head_hash = Block::find_main_chain_head(&db)
        .await
        .expect("Unable to find head");
    let data = format_data(
        &block_info.parent_hash,
        &block_info.name,
        &block_info.message,
    );
    let hash = calculate_hash(&data);
    let mut raw_scramble = scramble_from_hash(&hash);
    cleanup_scramble(&mut raw_scramble);
    let scramble = format_moves(&raw_scramble);
    let parent_block = match Block::find_by_hash(&db, &block_info.parent_hash).await {
        Ok(block) => block,
        Err(_) => {
            let resp = HttpResponse::Ok().body(views::get_block(
                &block_info.parent_hash,
                block_info.parent_hash == main_chain_head_hash,
                &block_info.name,
                &block_info.message,
                Some(&scramble),
                &hash,
                &block_info.solution,
                &block_info.solution_description,
            ));
            return FlashMessage::error("Parent block not found").set(resp);
        }
    };

    let data = format_data(&parent_block.hash, &block_info.name, &block_info.message);
    let hash = calculate_hash(&data);
    let raw_scramble = scramble_from_hash(&hash);
    let parsed_solution = parse_moves(&block_info.solution);

    if !verify_solution(&raw_scramble, &parsed_solution) {
        let resp = HttpResponse::Ok().body(views::get_block(
            &block_info.parent_hash,
            block_info.parent_hash == main_chain_head_hash,
            &block_info.name,
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
            &block_info.name,
            &block_info.message,
            &format_moves(&parsed_solution),
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
    pub page_size: Option<u32>,
    pub page_offset: Option<u32>,
}

#[get("/blocks")]
async fn get_blocks(
    db: web::Data<sqlx::SqlitePool>,
    query_params: web::Query<BlockQueryParams>,
) -> impl Responder {
    let show_all = query_params.all.unwrap_or(false);
    let page_size = query_params.page_size.unwrap_or(10);
    let page_offset = query_params.page_offset.unwrap_or(0);
    let next_offset = page_offset + page_size;

    let main_chain_hashes = Block::get_main_chain_hashes(&db)
        .await
        .expect("Unable to fetch main chain hashes");
    let blocks = Block::find_all(&db, !show_all, Some(page_size), Some(page_offset))
        .await
        .expect("Unable to fetch all blocks");

    HttpResponse::Ok().body(views::get_blocks(
        blocks,
        main_chain_hashes,
        next_offset,
        page_size,
        show_all,
    ))
}
