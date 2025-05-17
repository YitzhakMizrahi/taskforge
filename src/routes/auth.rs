use crate::{
    auth::{
        generate_token, hash_password, verify_password, AuthResponse, LoginRequest, RegisterRequest,
    },
    error::AppError,
};
use actix_web::{post, web, HttpResponse, Responder};
use sqlx::PgPool;
use validator::Validate;

/// Register a new user
///
/// Creates a new user account and returns an authentication token.
#[post("/register")]
pub async fn register(
    pool: web::Data<PgPool>,
    register_data: web::Json<RegisterRequest>,
) -> Result<impl Responder, AppError> {
    // Validate input
    register_data.validate()?;

    // Check if email already exists
    let existing_user = sqlx::query!("SELECT id FROM users WHERE email = $1", register_data.email)
        .fetch_optional(&**pool)
        .await?;

    if existing_user.is_some() {
        return Err(AppError::BadRequest("Email already registered".into()));
    }

    // Hash password
    let password_hash = hash_password(&register_data.password)?;

    // Insert new user
    let user = sqlx::query!(
        "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING id",
        register_data.username,
        register_data.email,
        password_hash
    )
    .fetch_one(&**pool)
    .await?;

    // Generate token
    let token = generate_token(user.id)?;

    Ok(HttpResponse::Created().json(AuthResponse {
        token,
        user_id: user.id,
    }))
}

/// Login user
///
/// Authenticates a user and returns an authentication token.
#[post("/login")]
pub async fn login(
    pool: web::Data<PgPool>,
    login_data: web::Json<LoginRequest>,
) -> Result<impl Responder, AppError> {
    // Validate input
    login_data.validate()?;

    // Get user from database
    let user = sqlx::query!(
        "SELECT id, password_hash FROM users WHERE email = $1",
        login_data.email
    )
    .fetch_optional(&**pool)
    .await?;

    match user {
        Some(user) => {
            // Verify password
            if verify_password(&login_data.password, &user.password_hash)? {
                // Generate token
                let token = generate_token(user.id)?;
                Ok(HttpResponse::Ok().json(AuthResponse {
                    token,
                    user_id: user.id,
                }))
            } else {
                Err(AppError::Unauthorized("Invalid credentials".into()))
            }
        }
        None => Err(AppError::Unauthorized("Invalid credentials".into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;
    use serde_json::json;
    use sqlx::PgPool;
    use std::env;

    // TODO: Fix DB connection for these tests or move to integration tests.
    #[ignore]
    #[actix_rt::test]
    async fn test_register_validation() {
        dotenv::dotenv().ok();
        let pool = PgPool::connect(&env::var("DATABASE_URL").expect("DATABASE_URL not set"))
            .await
            .unwrap();

        let app = test::init_service(
            actix_web::App::new()
                .app_data(web::Data::new(pool))
                .service(register),
        )
        .await;

        // Test invalid email
        let req = test::TestRequest::post()
            .uri("/register")
            .set_json(json!({
                "username": "test",
                "email": "invalid-email",
                "password": "password123"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());

        // Test short password
        let req = test::TestRequest::post()
            .uri("/register")
            .set_json(json!({
                "username": "test",
                "email": "test@example.com",
                "password": "short"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());
    }

    // TODO: Fix DB connection for these tests or move to integration tests.
    #[ignore]
    #[actix_rt::test]
    async fn test_login_validation() {
        dotenv::dotenv().ok();
        let pool = PgPool::connect(&env::var("DATABASE_URL").expect("DATABASE_URL not set"))
            .await
            .unwrap();

        let app = test::init_service(
            actix_web::App::new()
                .app_data(web::Data::new(pool))
                .service(login),
        )
        .await;

        // Test invalid email
        let req = test::TestRequest::post()
            .uri("/login")
            .set_json(json!({
                "email": "invalid-email",
                "password": "password123"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());

        // Test short password
        let req = test::TestRequest::post()
            .uri("/login")
            .set_json(json!({
                "email": "test@example.com",
                "password": "short"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());
    }
}
