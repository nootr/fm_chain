use askama::Template;
use chrono::{self, Datelike};

use crate::models::Block;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    current_year: i32,
    blocks: Vec<Block>,
}

pub fn get_index(blocks: Vec<Block>) -> String {
    let current_year = chrono::Utc::now().year();

    IndexTemplate {
        current_year,
        blocks,
    }
    .render()
    .expect("Failed to render template")
}

#[derive(Template)]
#[template(path = "block_form.html")]
struct BlockFormTemplate<'a> {
    parent_hash: &'a str,
    message: &'a str,
    scramble: Option<&'a str>,
    hash: &'a str,
    solution: &'a str,
    solution_description: &'a str,
    error: Option<&'a str>,
    alert: Option<&'a str>,
}

pub fn get_block(
    parent_hash: &str,
    message: &str,
    scramble: Option<&str>,
    hash: &str,
    solution: &str,
    solution_description: &str,
    error: Option<&str>,
    alert: Option<&str>,
) -> String {
    BlockFormTemplate {
        parent_hash,
        message,
        scramble,
        hash,
        solution,
        solution_description,
        error,
        alert,
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
