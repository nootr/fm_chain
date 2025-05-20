use actix_files as fs;
use actix_web::{App, HttpServer, middleware::Logger, web};
use env_logger::Env;
use sqlx::SqlitePool;

use fm_chain::cache::MemoryCache;
use fm_chain::config;
use fm_chain::routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let conf = config::Config::from_env();
    let db = SqlitePool::connect(&conf.database_url)
        .await
        .expect("DB failed");
    let cache = MemoryCache::<String, String>::default();
    cache.start_cleanup_task(60);

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    println!("Starting server at http://{}:{}/", &conf.host, conf.port);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(cache.clone()))
            .service(fs::Files::new(&conf.static_dir, "static"))
            .service(routes::favicon)
            .service(routes::get_index)
            .service(routes::get_block)
            .service(routes::post_block)
            .service(routes::get_blocks)
    })
    .bind((conf.host, conf.port))?
    .run()
    .await
}
