use askama::Template;
use std::collections::HashSet;

use crate::models::Block;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    cloudflare_code: Option<String>,
}

pub fn get_index(cloudflare_code: Option<String>) -> String {
    IndexTemplate { cloudflare_code }
        .render()
        .expect("Failed to render template")
}

#[derive(Template)]
#[template(path = "block_form.html")]
struct BlockFormTemplate<'a> {
    parent_hash: &'a str,
}

pub fn get_block(parent_hash: &str) -> String {
    BlockFormTemplate { parent_hash }
        .render()
        .expect("Failed to render template")
}

#[derive(Template)]
#[template(path = "solution_form.html")]
struct SolutionFormTemplate<'a> {
    parent_hash: &'a str,
    name: &'a str,
    message: &'a str,
    scramble: &'a str,
    hash: &'a str,
}

#[allow(clippy::too_many_arguments)]
pub fn get_solution(
    parent_hash: &str,
    name: &str,
    message: &str,
    scramble: &str,
    hash: &str,
) -> String {
    SolutionFormTemplate {
        parent_hash,
        name,
        message,
        scramble,
        hash,
    }
    .render()
    .expect("Failed to render template")
}

#[derive(Template)]
#[template(path = "blocks_overview.html")]
struct BlocksTemplate {
    blocks: Vec<Block>,
    main_chain_hashes: HashSet<String>,
    next_offset: u32,
    page_size: u32,
    show_all: bool,
}

pub fn get_blocks(
    blocks: Vec<Block>,
    main_chain_hashes: HashSet<String>,
    next_offset: u32,
    page_size: u32,
    show_all: bool,
) -> String {
    BlocksTemplate {
        blocks,
        main_chain_hashes,
        next_offset,
        page_size,
        show_all,
    }
    .render()
    .expect("Failed to render template")
}
