use taskforge::config::Config;
// No longer using taskforge::app_factory from lib.rs
use sqlx::postgres::PgPoolOptions;
// use sqlx::PgPool; // Removing to clear warning, type is inferred for pool

use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize logging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Load configuration
    let config = Config::from_env();
    log::info!("Starting server at {}", config.server_url());

    // Create database connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to create pool");

    // Start HTTP server
    HttpServer::new(move || {
        // App factory logic inlined here, as this resolved previous compilation issues
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600),
            )
            .app_data(web::Data::new(pool.clone())) // pool is captured by the closure
            .service(
                web::scope("/api")
                    .wrap(taskforge::auth::AuthMiddleware) // Sourced from lib.rs modules
                    .configure(taskforge::routes::config), // Sourced from lib.rs modules
            )
            .service(taskforge::routes::health::health) // Sourced from lib.rs modules
    })
    .bind((config.server_host, config.server_port))?
    .run()
    .await
}
