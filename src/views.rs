use askama::Template;

use crate::models::Block;

#[derive(Template)]
#[template(path = "block_form.html")]
struct BlockFormTemplate<'a> {
    parent_hash: &'a str,
    message: &'a str,
    scramble: Option<&'a str>,
    hash: &'a str,
    solution: &'a str,
    solution_description: &'a str,
}

#[allow(clippy::too_many_arguments)]
pub fn get_block(
    parent_hash: &str,
    message: &str,
    scramble: Option<&str>,
    hash: &str,
    solution: &str,
    solution_description: &str,
) -> String {
    BlockFormTemplate {
        parent_hash,
        message,
        scramble,
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
    next_offset: u32,
    page_size: u32,
    show_all: bool,
}

pub fn get_blocks(blocks: Vec<Block>, next_offset: u32, page_size: u32, show_all: bool) -> String {
    BlocksTemplate {
        blocks,
        next_offset,
        page_size,
        show_all,
    }
    .render()
    .expect("Failed to render template")
}
