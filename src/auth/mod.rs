pub mod extractors;
pub mod middleware;
pub mod password;
pub mod token;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use validator::Validate;

// Re-export necessary items
pub use middleware::AuthMiddleware;
pub use password::{hash_password, verify_password};
pub use token::{generate_token, verify_token, Claims};

lazy_static! {
    // Regex for username validation: alphanumeric, underscores, hyphens
    static ref USERNAME_REGEX: regex::Regex = regex::Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap();
}

/// Represents the payload for a user login request.
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    /// User's email address.
    /// Must be a valid email format.
    #[validate(email)]
    pub email: String,
    /// User's password.
    /// Must be at least 6 characters long.
    #[validate(length(min = 6))]
    pub password: String,
}

/// Represents the payload for a new user registration request.
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    /// Desired username for the new account.
    /// Must be between 3 and 32 characters, alphanumeric, and can include underscores or hyphens.
    #[validate(
        length(min = 3, max = 32),
        regex(
            path = "USERNAME_REGEX",
            message = "Username must be alphanumeric, underscores, or hyphens"
        )
    )]
    pub username: String,
    /// Email address for the new account.
    /// Must be a valid email format.
    #[validate(email)]
    pub email: String,
    /// Password for the new account.
    /// Must be at least 6 characters long.
    #[validate(length(min = 6))]
    pub password: String,
}

/// Response structure after successful authentication (login or registration).
/// Contains the JWT access token and the ID of the authenticated user.
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    /// The JWT (JSON Web Token) for session authentication.
    pub token: String,
    /// The unique identifier of the authenticated user.
    pub user_id: i32,
}

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
