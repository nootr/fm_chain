use actix_web::{
    HttpResponse,
    cookie::{Cookie, time::Duration},
};
use serde::{Deserialize, Serialize};

const FLASH_COOKIE_NAME: &str = "flash";

#[derive(Debug, Serialize, Deserialize)]
pub enum FlashMessageLevel {
    Info,
    Error,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FlashMessage {
    level: FlashMessageLevel,
    message: String,
}

impl FlashMessage {
    pub fn new(level: FlashMessageLevel, message: &str) -> Self {
        FlashMessage {
            level,
            message: message.to_string(),
        }
    }

    pub fn info(message: &str) -> Self {
        FlashMessage::new(FlashMessageLevel::Info, message)
    }

    pub fn error(message: &str) -> Self {
        FlashMessage::new(FlashMessageLevel::Error, message)
    }

    pub fn set(&self, resp: HttpResponse) -> HttpResponse {
        let cookie = Cookie::build(FLASH_COOKIE_NAME, serde_json::to_string(self).unwrap())
            .path("/")
            .max_age(Duration::minutes(1))
            .finish();

        let mut resp = resp;
        resp.add_cookie(&cookie).unwrap();
        resp
    }
}
