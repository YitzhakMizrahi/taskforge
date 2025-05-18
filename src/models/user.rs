use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
// sqlx::FromRow might be needed if User is directly mapped from a query result later
// use sqlx::FromRow;
use validator::Validate;

/// Represents a user entity as returned by the API (without sensitive information like password hash).
#[derive(Debug, Serialize, Deserialize)] // Add FromRow if User model is fetched directly from DB
pub struct User {
    /// Unique identifier for the user.
    pub id: i32,
    /// The username of the user.
    pub username: String,
    /// The email address of the user.
    pub email: String,
    /// Timestamp of when the user account was created.
    pub created_at: DateTime<Utc>,
}

/// Input structure for creating a new user (registration).
/// Contains validation rules for its fields.
/// The password field is for input only and is not stored directly.
#[derive(Debug, Deserialize, Validate)]
pub struct UserInput {
    /// The username for the new user.
    /// Must be between 3 and 50 characters.
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    /// The email address for the new user.
    /// Must be a valid email format.
    #[validate(email)]
    pub email: String,
    /// The password for the new user.
    /// Must be at least 6 characters long.
    #[validate(length(min = 6))]
    pub password: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_user_input_validation() {
        // Test valid input
        let input = UserInput {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };
        assert!(input.validate().is_ok());

        // Test invalid email
        let input = UserInput {
            username: "testuser".to_string(),
            email: "invalid-email".to_string(),
            password: "password123".to_string(),
        };
        assert!(input.validate().is_err());

        // Test short password
        let input = UserInput {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "short".to_string(),
        };
        assert!(input.validate().is_err());
    }
}
