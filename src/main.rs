use taskforge::config::Config;
// No longer using taskforge::app_factory from lib.rs
use sqlx::postgres::PgPoolOptions;
// use sqlx::PgPool; // Removing to clear warning, type is inferred for pool

use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};

// Extracted server logic
async fn run_app() -> std::io::Result<()> {
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
        .expect("Failed to create pool"); // This line will be tested for panic

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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    run_app().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[actix_web::test]
    #[should_panic(expected = "Failed to create pool")]
    async fn test_run_app_panics_on_db_connection_failure() {
        // Set an invalid DATABASE_URL to cause PgPoolOptions::connect to fail
        // Ensure the URL is syntactically valid for parsing but logically invalid for connection
        env::set_var("DATABASE_URL", "postgres://user:password@invalid-host-that-does-not-exist:5432/mydb_main_test");
        
        // Set other necessary env vars for Config::from_env() to succeed
        // Use a distinct port to avoid conflicts, though the server might not fully start
        // if the panic happens early, as expected.
        env::set_var("SERVER_PORT", "9999"); 
        env::set_var("SERVER_HOST", "127.0.0.1");

        // Call the extracted function; it should panic
        let _ = run_app().await;

        // Cleanup environment variables (won't run if panic occurs as expected,
        // but good practice if the test were to pass or for other test setups)
        // For #[should_panic] tests, cleanup needs to be handled carefully,
        // often by ensuring tests don't rely on shared mutable state across runs
        // or by using fixtures if the test framework supports them.
        // Since env vars are process-wide, this test implicitly assumes it doesn't mess up others,
        // or that test execution is isolated.
        // env::remove_var("DATABASE_URL");
        // env::remove_var("SERVER_PORT");
        // env::remove_var("SERVER_HOST");
        // Given this is a panic test, we will rely on test isolation for env vars.
    }
}
