use askama::Template;

use crate::models::Block;
use crate::utils::{BranchBlock, generate_branch_display};

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

pub fn get_index() -> String {
    IndexTemplate.render().expect("Failed to render template")
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
    blocks: Vec<BranchBlock>,
    head_hash: String,
}

pub fn get_blocks(blocks: Vec<Block>) -> String {
    BlocksTemplate {
        blocks: generate_branch_display(&blocks),
        head_hash: blocks[0].hash.clone(),
    }
    .render()
    .expect("Failed to render template")
}
