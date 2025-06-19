use actix_files as fs;
use actix_web::{App, HttpServer, middleware::Logger, web};
use env_logger::Env;
use sqlx::SqlitePool;

use fm_chain::cache::MemoryCache;
use fm_chain::config;
use fm_chain::routes;
use fm_chain::setup::run_setup;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let conf = config::Config::from_env();
    let db = SqlitePool::connect(&conf.database_url)
        .await
        .expect("DB failed");

    run_setup(&db).await.expect("Failed to setup database");

    let cache = MemoryCache::<String, String>::default();
    cache.start_cleanup_task(60);

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    println!("Starting server at http://{}:{}/", &conf.host, conf.port);

    let conf_clone = conf.clone();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(conf_clone.clone()))
            .app_data(web::Data::new(cache.clone()))
            .service(fs::Files::new(&conf.static_dir, "static"))
            .service(routes::favicon)
            .service(routes::get_health)
            .service(routes::get_index)
            .service(routes::get_parent)
            .service(routes::get_block)
            .service(routes::get_solution)
            .service(routes::post_solution)
            .service(routes::get_blocks)
    })
    .bind((conf.host, conf.port))?
    .run()
    .await
}
