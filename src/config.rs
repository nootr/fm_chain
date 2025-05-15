use dotenv::dotenv;
use std::env;

pub struct Config {
    pub host: String,
    pub port: u16,
    pub static_dir: String,
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().ok();

        Self {
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            static_dir: env::var("STATIC_DIR").unwrap_or_else(|_| "/static".to_string()),
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
        }
    }
}
