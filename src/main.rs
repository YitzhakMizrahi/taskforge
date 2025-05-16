use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use sqlx::{PgPool, FromRow};
use serde::Serialize;
use chrono::{DateTime, Utc};

async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok"
    }))
}

#[derive(Serialize, FromRow)]
struct User {
    id: i32,
    username: String,
    email: String,
    created_at: DateTime<Utc>,
}

async fn list_users(pool: web::Data<PgPool>) -> impl Responder {
    let users = sqlx::query_as::<_, User>(
        "SELECT id, username, email, created_at FROM users ORDER BY id"
    )
    .fetch_all(pool.get_ref())
    .await;

    match users {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().body("Database error")
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await.expect("Failed to connect to database");

    println!("Starting TaskForge server at http://127.0.0.1:8080");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/health", web::get().to(health))
            .route("/users", web::get().to(list_users))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}