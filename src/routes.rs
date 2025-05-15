use actix_files::NamedFile;
use actix_web::{HttpResponse, Responder, get, web};
use serde::Deserialize;

use crate::views;

#[get("/favicon.ico")]
async fn favicon() -> impl Responder {
    NamedFile::open_async("static/favicon.ico").await
}

#[get("/")]
async fn index_get() -> impl Responder {
    HttpResponse::Ok().body(views::get_index())
}

#[derive(Deserialize)]
struct InitialBlockInfo {
    previous_hash: String,
    message: String,
}

#[get("/block")]
async fn block_get(block_info: web::Query<InitialBlockInfo>) -> impl Responder {
    HttpResponse::Ok().body(views::get_block(
        block_info.previous_hash.clone(),
        block_info.message.clone(),
    ))
}
