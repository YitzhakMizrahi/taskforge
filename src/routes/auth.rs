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
    // Cleaned up imports for pure DTO validation tests
    use crate::auth::{LoginRequest, RegisterRequest}; 
    use validator::Validate;
    // No longer needed: 
    // use super::*;
    // use actix_web::test;
    // use serde_json::json;
    // use sqlx::PgPool;
    // use std::env;
    // use dotenv::dotenv; // dotenv::dotenv().ok(); was removed from test bodies

    #[test]
    fn test_register_request_validation() { /* ... as refactored ... */ 
        let invalid_email_input = RegisterRequest {
            username: "testuser".to_string(),
            email: "invalid-email".to_string(),
            password: "ValidPassword123!".to_string(),
        };
        assert!(invalid_email_input.validate().is_err(), "Validation should fail for invalid email format.");

        let short_password_input = RegisterRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "short".to_string(), 
        };
        assert!(short_password_input.validate().is_err(), "Validation should fail for short password.");

        let short_username_input = RegisterRequest {
            username: "u".to_string(), 
            email: "test@example.com".to_string(),
            password: "ValidPassword123!".to_string(),
        };
        assert!(short_username_input.validate().is_err(), "Validation should fail for short username.");

        let long_username = "a".repeat(33);
        let long_username_input = RegisterRequest {
            username: long_username,
            email: "test@example.com".to_string(),
            password: "ValidPassword123!".to_string(),
        };
        assert!(long_username_input.validate().is_err(), "Validation should fail for long username.");
        
        let invalid_char_username_input = RegisterRequest {
            username: "user!".to_string(), 
            email: "test@example.com".to_string(),
            password: "ValidPassword123!".to_string(),
        };
        assert!(invalid_char_username_input.validate().is_err(), "Validation should fail for username with invalid characters.");

        let valid_input = RegisterRequest {
            username: "test_user_123".to_string(),
            email: "test@example.com".to_string(),
            password: "ValidPassword123!".to_string(),
        };
        assert!(valid_input.validate().is_ok(), "Validation should pass for valid input: {:?}", valid_input.validate().err());
    }

    #[test]
    fn test_login_request_validation() { /* ... as refactored ... */ 
        let invalid_email_input = LoginRequest {
            email: "invalid-email".to_string(),
            password: "ValidPassword123!".to_string(),
        };
        assert!(invalid_email_input.validate().is_err(), "Validation should fail for invalid email format.");

        let short_password_input = LoginRequest {
            email: "test@example.com".to_string(),
            password: "short".to_string(), 
        };
        assert!(short_password_input.validate().is_err(), "Validation should fail for short password.");

        let empty_password_input = LoginRequest {
            email: "test@example.com".to_string(),
            password: "".to_string(), 
        };
        assert!(empty_password_input.validate().is_err(), "Validation should fail for empty password if min_length > 0.");

        let valid_input = LoginRequest {
            email: "test@example.com".to_string(),
            password: "ValidPassword123!".to_string(),
        };
        assert!(valid_input.validate().is_ok(), "Validation should pass for valid input.");
    }
}
