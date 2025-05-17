use actix_web::{web, App, HttpResponse, HttpServer, Responder, HttpRequest};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use validator::Validate;
use uuid::Uuid;
use actix_cors::Cors;
use actix_files::Files;
use std::env;
use std::sync::Arc;
use std::collections::HashMap;
use std::sync::Mutex;
use log::{info, error};
use crate::auth::{AuthMiddleware, get_user_id, LoginRequest, RegisterRequest, AuthResponse, hash_password, verify_password, generate_token};

mod security;
mod tasks;
mod auth;

use security::{UserInput, sanitize_input, validate_sql_input};
use tasks::{Task, TaskInput, TaskQuery};

#[derive(Debug, Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub tasks: Arc<Mutex<HashMap<Uuid, Task>>>,
}

#[derive(Debug, Serialize, FromRow)]
struct User {
    id: i32,
    username: String,
    email: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
struct UserQuery {
    search: Option<String>,
}

async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok"
    }))
}

async fn list_users(pool: web::Data<PgPool>, query: web::Query<UserQuery>) -> impl Responder {
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
             ORDER BY id",
        )
        .bind(format!("%{}%", search))
        .fetch_all(pool.get_ref())
        .await
    } else {
        sqlx::query_as::<_, User>("SELECT id, username, email, created_at FROM users ORDER BY id")
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

#[derive(Debug, Deserialize, Validate)]
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

async fn create_task(
    pool: web::Data<PgPool>,
    task_data: web::Json<TaskInput>,
    req: HttpRequest,
) -> impl Responder {
    let user_id = get_user_id(&req);
    
    // Validate task input
    if let Err(e) = task_data.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid input",
            "details": e
        }));
    }

    let task = Task::new(task_data.into_inner(), user_id);

    // Insert task with parameterized query
    let result = sqlx::query_as::<_, Task>(
        "INSERT INTO tasks (id, title, description, priority, status, due_date, created_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         RETURNING id, title, description, priority, status, due_date, created_at, updated_at, created_by, assigned_to"
    )
    .bind(task.id)
    .bind(task.title)
    .bind(task.description)
    .bind(task.priority)
    .bind(task.status)
    .bind(task.due_date)
    .bind(task.created_by)
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(task) => HttpResponse::Created().json(task),
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().body("Database error")
        }
    }
}

async fn get_tasks(
    pool: web::Data<PgPool>,
    query: web::Query<TaskQuery>,
) -> impl Responder {
    // Build the query dynamically based on filters
    let mut sql = String::from(
        "SELECT id, title, description, priority, status, due_date, created_at, updated_at, created_by, assigned_to 
         FROM tasks WHERE 1=1"
    );
    let mut params: Vec<String> = Vec::new();
    let mut param_count = 1;

    if let Some(status) = &query.status {
        sql.push_str(&format!(" AND status = ${}", param_count));
        params.push(status.clone());
        param_count += 1;
    }

    if let Some(priority) = &query.priority {
        sql.push_str(&format!(" AND priority = ${}", param_count));
        params.push(priority.clone());
        param_count += 1;
    }

    if let Some(assigned_to) = query.assigned_to {
        sql.push_str(&format!(" AND assigned_to = ${}", param_count));
        params.push(assigned_to.to_string());
        param_count += 1;
    }

    if let Some(created_by) = query.created_by {
        sql.push_str(&format!(" AND created_by = ${}", param_count));
        params.push(created_by.to_string());
        param_count += 1;
    }

    if let Some(search) = &query.search {
        if let Err(_) = validate_sql_input(search) {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid search query"
            }));
        }
        let sanitized_search = sanitize_input(search);
        sql.push_str(&format!(
            " AND (title ILIKE ${} OR description ILIKE ${})",
            param_count,
            param_count
        ));
        params.push(format!("%{}%", sanitized_search));
        param_count += 1;
    }

    sql.push_str(" ORDER BY created_at DESC");

    // Execute the query with parameters
    let mut query = sqlx::query_as::<_, Task>(&sql);
    for param in params {
        query = query.bind(param);
    }

    let result = query.fetch_all(pool.get_ref()).await;

    match result {
        Ok(tasks) => HttpResponse::Ok().json(tasks),
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().body("Database error")
        }
    }
}

async fn get_task(
    pool: web::Data<PgPool>,
    task_id: web::Path<Uuid>,
) -> impl Responder {
    let result = sqlx::query_as::<_, Task>(
        "SELECT id, title, description, priority, status, due_date, created_at, updated_at, created_by, assigned_to 
         FROM tasks WHERE id = $1"
    )
    .bind(task_id.into_inner())
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(task)) => HttpResponse::Ok().json(task),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Task not found"
        })),
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().body("Database error")
        }
    }
}

async fn update_task(
    pool: web::Data<PgPool>,
    task_id: web::Path<Uuid>,
    task_data: web::Json<TaskInput>,
) -> impl Responder {
    // Validate task input
    if let Err(e) = task_data.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid input",
            "details": e
        }));
    }

    let result = sqlx::query_as::<_, Task>(
        "UPDATE tasks 
         SET title = $1, description = $2, priority = $3, status = $4, due_date = $5
         WHERE id = $6
         RETURNING id, title, description, priority, status, due_date, created_at, updated_at, created_by, assigned_to"
    )
    .bind(&task_data.title)
    .bind(&task_data.description)
    .bind(&task_data.priority)
    .bind(&task_data.status)
    .bind(&task_data.due_date)
    .bind(task_id.into_inner())
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(task) => HttpResponse::Ok().json(task),
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Task not found"
        })),
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().body("Database error")
        }
    }
}

async fn delete_task(
    pool: web::Data<PgPool>,
    task_id: web::Path<Uuid>,
) -> impl Responder {
    let result = sqlx::query!(
        "DELETE FROM tasks WHERE id = $1",
        task_id.into_inner()
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().body("Database error")
        }
    }
}

async fn login(
    pool: web::Data<PgPool>,
    login_data: web::Json<LoginRequest>,
) -> impl Responder {
    match sqlx::query!(
        "SELECT id, password_hash FROM users WHERE email = $1",
        login_data.email
    )
    .fetch_optional(&**pool)
    .await
    {
        Ok(Some(user)) => {
            if verify_password(&login_data.password, &user.password_hash).unwrap_or(false) {
                match generate_token(user.id) {
                    Ok(token) => HttpResponse::Ok().json(AuthResponse {
                        token,
                        user_id: user.id,
                    }),
                    Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Failed to generate token"
                    })),
                }
            } else {
                HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "Invalid credentials"
                }))
            }
        }
        Ok(None) => HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Invalid credentials"
        })),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Database error"
        })),
    }
}

async fn register(
    pool: web::Data<PgPool>,
    register_data: web::Json<RegisterRequest>,
) -> impl Responder {
    // Check if email already exists
    match sqlx::query!("SELECT id FROM users WHERE email = $1", register_data.email)
        .fetch_optional(&**pool)
        .await
    {
        Ok(Some(_)) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Email already registered"
            }))
        }
        Ok(None) => {}
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database error"
            }))
        }
    }

    // Hash password
    let password_hash = match hash_password(&register_data.password) {
        Ok(hash) => hash,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to hash password"
            }))
        }
    };

    // Insert new user
    match sqlx::query!(
        "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING id",
        register_data.username,
        register_data.email,
        password_hash
    )
    .fetch_one(&**pool)
    .await
    {
        Ok(user) => {
            match generate_token(user.id) {
                Ok(token) => HttpResponse::Created().json(AuthResponse {
                    token,
                    user_id: user.id,
                }),
                Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to generate token"
                })),
            }
        }
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to create user"
        })),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    let app_state = web::Data::new(AppState {
        pool: pool.clone(),
        tasks: Arc::new(Mutex::new(HashMap::new())),
    });

    info!("Starting server at http://localhost:8080");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(AuthMiddleware)
            .app_data(app_state.clone())
            .service(
                web::scope("/api")
                    .service(
                        web::resource("/tasks")
                            .route(web::get().to(get_tasks))
                            .route(web::post().to(create_task)),
                    )
                    .service(
                        web::resource("/tasks/{id}")
                            .route(web::get().to(get_task))
                            .route(web::put().to(update_task))
                            .route(web::delete().to(delete_task)),
                    )
                    .service(
                        web::scope("/auth")
                            .service(
                                web::resource("/login")
                                    .route(web::post().to(login))
                            )
                            .service(
                                web::resource("/register")
                                    .route(web::post().to(register))
                            )
                    ),
            )
            .service(Files::new("/", "./static").index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
