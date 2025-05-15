use askama::Template;
use chrono::{self, Datelike};

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
    // TODO: calculate scramble
    let scramble = String::from("R U R'");
    BlockFormTemplate {
        previous_hash,
        message,
        scramble: Some(scramble),
    }
    .render()
    .expect("Failed to render template")
}
