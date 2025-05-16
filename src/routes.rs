use actix_files::NamedFile;
use actix_web::{HttpResponse, Responder, get, post, web};
use serde::Deserialize;

use crate::utils::{
    calculate_hash, cleanup_scramble, format_data, format_scramble, scramble_from_hash,
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
    previous_hash: String,
    message: String,
}

#[get("/block")]
async fn get_block(block_info: web::Query<InitialBlockInfo>) -> impl Responder {
    let data = format_data(block_info.previous_hash.clone(), block_info.message.clone());
    let hash = calculate_hash(&data);
    let mut raw_scramble = scramble_from_hash(&hash);
    cleanup_scramble(&mut raw_scramble);
    let scramble = format_scramble(&raw_scramble);

    HttpResponse::Ok().body(views::get_block(
        block_info.previous_hash.clone(),
        block_info.message.clone(),
        scramble,
        hash.clone(),
        String::new(),
        String::new(),
    ))
}

#[derive(Debug, Deserialize)]
struct CompleteBlockInfo {
    previous_hash: String,
    message: String,
    solution: String,
    solution_description: String,
}

#[post("/block")]
async fn post_block(block_info: web::Form<CompleteBlockInfo>) -> impl Responder {
    let data = format_data(block_info.previous_hash.clone(), block_info.message.clone());
    let hash = calculate_hash(&data);
    let mut raw_scramble = scramble_from_hash(&hash);
    cleanup_scramble(&mut raw_scramble);

    // TODO: verify and save the solution

    HttpResponse::Ok().body(views::get_block(
        hash,
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
    ))
}
