use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
// sqlx::FromRow might be needed if User is directly mapped from a query result later
// use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize)] // Add FromRow if needed
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UserInput {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6))]
    pub password: String, // This password field here is for input, it won't be stored directly in User model
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
