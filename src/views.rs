use askama::Template;
use chrono::{self, Datelike};

use crate::utils::{
    calculate_hash, cleanup_scramble, format_data, format_scramble, scramble_from_hash,
};

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    current_year: i32,
    previous_hash: String,
    message: String,
    scramble: Option<String>,
}

pub fn get_index() -> String {
    let current_year = chrono::Utc::now().year();
    IndexTemplate {
        current_year,
        previous_hash: String::new(),
        message: String::new(),
        scramble: None,
    }
    .render()
    .expect("Failed to render template")
}

#[derive(Template)]
#[template(path = "block_form.html")]
struct BlockFormTemplate {
    previous_hash: String,
    message: String,
    scramble: Option<String>,
}

pub fn get_block(previous_hash: String, message: String) -> String {
    let data = format_data(previous_hash.clone(), message.clone());
    let hash = calculate_hash(&data);
    let mut raw_scramble = scramble_from_hash(&hash);
    cleanup_scramble(&mut raw_scramble);
    let scramble = format_scramble(&raw_scramble);

    BlockFormTemplate {
        previous_hash,
        message,
        scramble: Some(scramble),
    }
    .render()
    .expect("Failed to render template")
}
