use actix_files::NamedFile;
use actix_web::{HttpResponse, Responder, get, post, web};
use serde::Deserialize;

use crate::cache::{Cache, MemoryCache};
use crate::config;
use crate::messages::FlashMessage;
use crate::models::Block;
use crate::utils::{
    calculate_hash, cleanup_scramble, format_data, format_moves, is_htmx_request, parse_moves,
    scramble_from_hash, verify_solution,
};
use crate::views;

#[get("/favicon.ico")]
async fn favicon() -> impl Responder {
    NamedFile::open_async("static/favicon.ico").await
}

#[get("/")]
async fn get_index(
    conf: web::Data<config::Config>,
    cache: web::Data<MemoryCache<String, String>>,
) -> impl Responder {
    let cache_key = "index_page".to_string();

    if let Some(cached_page) = cache
        .get(&cache_key)
        .expect("Failed to get index from cache")
    {
        return HttpResponse::Ok().body(cached_page);
    }

    let cloudflare_code = conf.cloudflare_code.clone();
    let response = views::get_index(cloudflare_code);

    cache
        .set(&cache_key, response.clone(), None)
        .expect("Failed to cache index page");

    HttpResponse::Ok().body(response)
}

#[get("/health")]
async fn get_health() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[derive(Deserialize)]
struct InitialBlockInfo {
    parent_hash: String,
    name: Option<String>,
    message: Option<String>,
}

#[get("/block")]
async fn get_block(
    request: actix_web::HttpRequest,
    conf: web::Data<config::Config>,
    db: web::Data<sqlx::SqlitePool>,
    block_info: web::Query<InitialBlockInfo>,
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
    if is_htmx_request(&request) {
        return HttpResponse::Ok().body(views::get_partial_block(&block_info.parent_hash));
    }

    let cloudflare_code = conf.cloudflare_code.clone();
    HttpResponse::Ok().body(views::get_block(cloudflare_code, &block_info.parent_hash))
}

#[get("/solution")]
async fn get_solution(
    conf: web::Data<config::Config>,
    request: actix_web::HttpRequest,
    db: web::Data<sqlx::SqlitePool>,
    block_info: web::Query<InitialBlockInfo>,
) -> impl Responder {
    if block_info.parent_hash.is_empty() {
        return HttpResponse::BadRequest().body("Parent hash is required.");
    }

    if block_info.name.clone().unwrap_or_default().is_empty()
        || block_info.message.clone().unwrap_or_default().is_empty()
    {
        // Only render block if fields are missing
        return HttpResponse::Ok().body("<div id=\"solution-form\" hidden></div>");
    }

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
    let name = block_info.name.clone().unwrap_or_default();
    let message = block_info.message.clone().unwrap_or_default();
    let data = format_data(&block_info.parent_hash, &name, &message);
    let hash = calculate_hash(&data);
    let mut raw_scramble = scramble_from_hash(&hash);
    cleanup_scramble(&mut raw_scramble);
    let scramble = format_moves(&raw_scramble);

    if is_htmx_request(&request) {
        return HttpResponse::Ok().body(views::get_partial_solution(
            &block_info.parent_hash,
            &name,
            &message,
            &scramble,
            &hash,
        ));
    }

    let cloudflare_code = conf.cloudflare_code.clone();
    HttpResponse::Ok().body(views::get_solution(
        cloudflare_code,
        &block_info.parent_hash,
        &name,
        &message,
        &scramble,
        &hash,
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

#[post("/solution")]
async fn post_solution(
    db: web::Data<sqlx::SqlitePool>,
    block_info: web::Form<CompleteBlockInfo>,
) -> impl Responder {
    if block_info.parent_hash.is_empty()
        || block_info.name.is_empty()
        || block_info.message.is_empty()
        || block_info.solution.is_empty()
        || block_info.solution_description.is_empty()
    {
        return HttpResponse::BadRequest().body("All fields are required.");
    }

    let parent_block = match Block::find_by_hash(&db, &block_info.parent_hash).await {
        Ok(block) => {
            if !block.can_create_child() {
                return HttpResponse::BadRequest()
                    .body("This block cannot be used as a parent for a new block.");
            }
            block
        }
        Err(_) => {
            return HttpResponse::NotFound().body("Parent block not found");
        }
    };

    let data = format_data(
        &block_info.parent_hash,
        &block_info.name,
        &block_info.message,
    );
    let hash = calculate_hash(&data);
    let mut raw_scramble = scramble_from_hash(&hash);
    cleanup_scramble(&mut raw_scramble);

    let data = format_data(&parent_block.hash, &block_info.name, &block_info.message);
    let hash = calculate_hash(&data);
    let raw_scramble = scramble_from_hash(&hash);
    let parsed_solution = parse_moves(&block_info.solution);

    if !verify_solution(&raw_scramble, &parsed_solution) {
        let resp = HttpResponse::BadRequest().body("Incorrect solution");
        return FlashMessage::error(
            "I'm sorry, but your solution doesn't seem to be correct. Please double-check it!",
        )
        .set(resp);
    }

    if Block::hash_and_solution_exists(&db, &hash, &format_moves(&parsed_solution))
        .await
        .expect("Failed to check for existing block")
    {
        let resp = HttpResponse::BadRequest().body("This solution already exists");
        return FlashMessage::error("This solution already exists").set(resp);
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

    let response = HttpResponse::TemporaryRedirect()
        .append_header(("HX-Redirect", "/?all=true"))
        .finish();
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
    request: actix_web::HttpRequest,
    db: web::Data<sqlx::SqlitePool>,
    query_params: web::Query<BlockQueryParams>,
) -> impl Responder {
    if !is_htmx_request(&request) {
        return HttpResponse::NotFound().body("This endpoint is only available for HTMX requests.");
    }

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

    HttpResponse::Ok().body(views::get_partial_blocks(
        blocks,
        main_chain_hashes,
        next_offset,
        page_size,
        show_all,
    ))
}
