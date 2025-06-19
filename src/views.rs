use askama::Template;
use std::collections::HashSet;

use crate::models::Block;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    cloudflare_code: Option<String>,
    modal: Option<String>,
    recommended_block_count: usize,
}

pub fn get_index(cloudflare_code: Option<String>, recommended_block_count: usize) -> String {
    IndexTemplate {
        cloudflare_code,
        modal: None,
        recommended_block_count,
    }
    .render()
    .expect("Failed to render template")
}

#[derive(Template)]
#[template(path = "parent_form.html")]
struct ParentFormTemplate {
    blocks: Vec<Block>,
}

pub fn get_partial_parent(blocks: Vec<Block>) -> String {
    ParentFormTemplate { blocks }
        .render()
        .expect("Failed to render template")
}

pub fn get_parent(
    cloudflare_code: Option<String>,
    recommended_block_count: usize,
    blocks: Vec<Block>,
) -> String {
    let modal = ParentFormTemplate { blocks }
        .render()
        .expect("Failed to render template");

    IndexTemplate {
        cloudflare_code,
        modal: Some(modal),
        recommended_block_count,
    }
    .render()
    .expect("Failed to render template")
}

#[derive(Template)]
#[template(path = "block_form.html")]
struct BlockFormTemplate<'a> {
    parent_hash: &'a str,
    message: Option<String>,
    solution_html: Option<String>,
}

pub fn get_partial_block(parent_hash: &str) -> String {
    BlockFormTemplate {
        parent_hash,
        message: None,
        solution_html: None,
    }
    .render()
    .expect("Failed to render template")
}

pub fn get_block(
    cloudflare_code: Option<String>,
    parent_hash: &str,
    recommended_block_count: usize,
) -> String {
    let modal = BlockFormTemplate {
        parent_hash,
        message: None,
        solution_html: None,
    }
    .render()
    .expect("Failed to render template");

    IndexTemplate {
        cloudflare_code,
        modal: Some(modal),
        recommended_block_count,
    }
    .render()
    .expect("Failed to render template")
}

#[derive(Template)]
#[template(path = "solution_form_placeholder.html")]
struct SolutionFormPlaceholderTemplate;

pub fn get_solution_placeholder() -> String {
    SolutionFormPlaceholderTemplate
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
pub fn get_partial_solution(
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

#[allow(clippy::too_many_arguments)]
pub fn get_solution(
    cloudflare_code: Option<String>,
    parent_hash: &str,
    name: &str,
    message: &str,
    scramble: &str,
    hash: &str,
    recommended_block_count: usize,
) -> String {
    let solution_partial = SolutionFormTemplate {
        parent_hash,
        name,
        message,
        scramble,
        hash,
    }
    .render()
    .expect("Failed to render template");

    let modal = BlockFormTemplate {
        parent_hash,
        message: Some(message.to_string()),
        solution_html: Some(solution_partial),
    }
    .render()
    .expect("Failed to render template");

    IndexTemplate {
        cloudflare_code,
        modal: Some(modal),
        recommended_block_count,
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

pub fn get_partial_blocks(
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
