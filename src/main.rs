use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use validator::Validate;

mod security;
use security::{UserInput, sanitize_input, validate_sql_input};

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

#[derive(Deserialize)]
struct UserQuery {
    search: Option<String>,
}

async fn list_users(
    pool: web::Data<PgPool>,
    query: web::Query<UserQuery>,
) -> impl Responder {
    // Validate and sanitize search input if provided
    let search = if let Some(search) = &query.search {
        if let Err(_) = validate_sql_input(search) {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid search query"
            }));
        }
        Some(sanitize_input(search))
    } else {
        None
    };

    let users = if let Some(search) = search {
        sqlx::query_as::<_, User>(
            "SELECT id, username, email, created_at FROM users 
             WHERE username ILIKE $1 OR email ILIKE $1 
             ORDER BY id"
        )
        .bind(format!("%{}%", search))
        .fetch_all(pool.get_ref())
        .await
    } else {
        sqlx::query_as::<_, User>(
            "SELECT id, username, email, created_at FROM users ORDER BY id"
        )
        .fetch_all(pool.get_ref())
        .await
    };

    match users {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().body("Database error")
        }
    }
}

#[derive(Deserialize)]
struct CreateUserRequest {
    #[serde(flatten)]
    user_input: UserInput,
}

async fn create_user(
    pool: web::Data<PgPool>,
    user_data: web::Json<CreateUserRequest>,
) -> impl Responder {
    // Validate user input
    if let Err(e) = user_data.user_input.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid input",
            "details": e
        }));
    }

    // Hash password (you should implement this)
    let password_hash = "hashed_password"; // TODO: Implement proper password hashing

    // Insert user with parameterized query
    let result = sqlx::query!(
        "INSERT INTO users (username, email, password_hash) 
         VALUES ($1, $2, $3) 
         RETURNING id, username, email, created_at",
        user_data.user_input.username,
        user_data.user_input.email,
        password_hash
    )
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(user) => HttpResponse::Created().json(serde_json::json!({
            "id": user.id,
            "username": user.username,
            "email": user.email,
            "created_at": user.created_at
        })),
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
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    println!("Starting TaskForge server at http://127.0.0.1:8080");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/health", web::get().to(health))
            .route("/users", web::get().to(list_users))
            .route("/users", web::post().to(create_user))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
