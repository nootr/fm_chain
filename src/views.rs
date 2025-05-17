use askama::Template;
use chrono::{self, Datelike};

use crate::models::Block;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    current_year: i32,
    blocks: Vec<Block>,
    parent_hash: String,
    message: String,
    scramble: Option<String>,
    hash: String,
    solution: String,
    solution_description: String,
}

pub fn get_index(blocks: Vec<Block>) -> String {
    let current_year = chrono::Utc::now().year();
    let parent_hash = blocks
        .get(0)
        .map(|block| block.hash.clone())
        .unwrap_or_default();

    IndexTemplate {
        current_year,
        blocks,
        parent_hash,
        message: String::new(),
        scramble: None,
        hash: String::new(),
        solution: String::new(),
        solution_description: String::new(),
    }
    .render()
    .expect("Failed to render template")
}

#[derive(Template)]
#[template(path = "block_form.html")]
struct BlockFormTemplate {
    parent_hash: String,
    message: String,
    scramble: Option<String>,
    hash: String,
    solution: String,
    solution_description: String,
}

pub fn get_block(
    parent_hash: String,
    message: String,
    scramble: String,
    hash: String,
    solution: String,
    solution_description: String,
) -> String {
    BlockFormTemplate {
        parent_hash,
        message,
        scramble: Some(scramble),
        hash,
        solution,
        solution_description,
    }
    .render()
    .expect("Failed to render template")
}

#[derive(Template)]
#[template(path = "blocks_overview.html")]
struct BlocksTemplate {
    blocks: Vec<Block>,
}

pub fn get_blocks(blocks: Vec<Block>) -> String {
    BlocksTemplate { blocks }
        .render()
        .expect("Failed to render template")
}
