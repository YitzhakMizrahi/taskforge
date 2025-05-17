use regex::Regex;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UserInput {
    #[validate(length(min = 3, max = 32))]
    #[validate(regex(path = "USERNAME_REGEX", message = "Username must be alphanumeric"))]
    pub username: String,

    #[validate(email)]
    pub email: String,

    #[validate(length(min = 8, max = 100))]
    pub password: String,
}

lazy_static::lazy_static! {
    static ref USERNAME_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap();
}

pub fn sanitize_input(input: &str) -> String {
    // Remove any potential SQL injection patterns
    let sanitized = input
        .replace("'", "''")
        .replace(";", "")
        .replace("--", "")
        .replace("/*", "")
        .replace("*/", "");

    sanitized.trim().to_string()
}

pub fn validate_sql_input(input: &str) -> Result<(), ValidationError> {
    // Check for common SQL injection patterns
    let sql_patterns = [
        "SELECT", "INSERT", "UPDATE", "DELETE", "DROP", "UNION", "ALTER", "EXEC", "EXECUTE",
        "DECLARE", "WAITFOR",
    ];

    for pattern in sql_patterns.iter() {
        if input.to_uppercase().contains(pattern) {
            return Err(ValidationError::new("sql_injection"));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_input() {
        let input = "test'; DROP TABLE users; --";
        let sanitized = sanitize_input(input);
        assert_eq!(sanitized, "test'' DROP TABLE users");
    }

    #[test]
    fn test_validate_sql_input() {
        let input = "SELECT * FROM users";
        assert!(validate_sql_input(input).is_err());

        let input = "normal text";
        assert!(validate_sql_input(input).is_ok());
    }

    #[test]
    fn test_user_input_validation() {
        let valid_input = UserInput {
            username: "valid_user123".to_string(),
            email: "test@example.com".to_string(),
            password: "secure_password123".to_string(),
        };
        assert!(valid_input.validate().is_ok());

        let invalid_input = UserInput {
            username: "invalid user!".to_string(),
            email: "invalid-email".to_string(),
            password: "short".to_string(),
        };
        assert!(invalid_input.validate().is_err());
    }
}
