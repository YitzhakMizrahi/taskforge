pub mod middleware;
pub mod password;
pub mod token;

use actix_web::{HttpMessage, HttpRequest};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::error::AppError;

// Re-export necessary items
pub use middleware::AuthMiddleware;
pub use password::{hash_password, verify_password};
pub use token::{generate_token, verify_token, Claims};

lazy_static! {
    // Regex for username validation: alphanumeric, underscores, hyphens
    static ref USERNAME_REGEX: regex::Regex = regex::Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap();
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(
        length(min = 3, max = 32),
        regex(
            path = "USERNAME_REGEX",
            message = "Username must be alphanumeric, underscores, or hyphens"
        )
    )]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6))]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: i32,
}

/// Extracts user ID from request extensions.
///
/// # Arguments
/// * `req` - A reference to the `HttpRequest`.
///
/// # Returns
/// A `Result<i32, AppError>` containing the user ID if found,
/// or an `AppError::Unauthorized` if not found.
pub fn get_user_id(req: &HttpRequest) -> Result<i32, AppError> {
    req.extensions()
        .get::<Claims>()
        .map(|claims| claims.sub)
        .ok_or_else(|| {
            AppError::Unauthorized(
                "User ID not found in token claims. Valid token required.".to_string(),
            )
        })
}

/*
// Old Option<i32> version - kept for reference if needed elsewhere
pub fn get_user_id_optional(req: &HttpRequest) -> Option<i32> {
    req.extensions().get::<Claims>().map(|claims| claims.sub)
}
*/

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_login_request_validation() {
        let valid_login = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };
        assert!(valid_login.validate().is_ok());

        let invalid_email_login = LoginRequest {
            email: "testexample.com".to_string(),
            password: "password123".to_string(),
        };
        assert!(invalid_email_login.validate().is_err());

        let short_password_login = LoginRequest {
            email: "test@example.com".to_string(),
            password: "123".to_string(),
        };
        assert!(short_password_login.validate().is_err());
    }

    #[test]
    fn test_register_request_validation() {
        let valid_register = RegisterRequest {
            username: "test_user-123".to_string(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };
        assert!(valid_register.validate().is_ok());

        let invalid_username_register = RegisterRequest {
            username: "test user!".to_string(), // Contains space and exclamation
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };
        assert!(invalid_username_register.validate().is_err());

        let short_username_register = RegisterRequest {
            username: "tu".to_string(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };
        assert!(short_username_register.validate().is_err());
    }
}
