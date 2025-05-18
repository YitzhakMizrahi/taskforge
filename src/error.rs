//!
//! # Custom Error Handling
//!
//! This module defines the custom error type `AppError` used throughout the application.
//! It centralizes error management, providing a consistent way to handle and represent
//! various error conditions that can occur, from database issues to validation failures.
//!
//! `AppError` implements `actix_web::error::ResponseError` to seamlessly convert
//! application errors into appropriate HTTP responses with JSON bodies.
//! It also provides `From` trait implementations for common error types like `sqlx::Error`,
//! `validator::ValidationErrors`, `jsonwebtoken::errors::Error`, and `bcrypt::BcryptError`,
//! allowing for easy conversion using the `?` operator.

use actix_web::{error::ResponseError, HttpResponse};
use serde_json::json;
use std::fmt;
use validator::ValidationErrors;

/// Represents all possible errors that can occur within the application.
///
/// Each variant corresponds to a specific type of error, often carrying a message
/// detailing the issue. These errors are then converted into appropriate HTTP responses.
#[derive(Debug)]
pub enum AppError {
    /// Represents an unauthorized access attempt (HTTP 401).
    /// Typically used when authentication fails or is required but missing.
    Unauthorized(String),
    /// Represents a client-side error due to a malformed or invalid request (HTTP 400).
    BadRequest(String),
    /// Represents a situation where a requested resource was not found (HTTP 404).
    NotFound(String),
    /// Represents an unexpected server-side error (HTTP 500).
    /// This can be used for generic internal errors not covered by more specific types.
    InternalServerError(String),
    /// Represents an error originating from database operations (HTTP 500).
    /// Wraps errors from the `sqlx` crate.
    DatabaseError(String),
    /// Represents an error due to failed input validation (HTTP 422 Unprocessable Entity).
    /// Wraps errors from the `validator` crate.
    ValidationError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            AppError::InternalServerError(msg) => write!(f, "Internal Server Error: {}", msg),
            AppError::DatabaseError(msg) => write!(f, "Database Error: {}", msg),
            AppError::ValidationError(msg) => write!(f, "Validation Error: {}", msg),
        }
    }
}

/// Converts `AppError` variants into `HttpResponse` objects.
///
/// This implementation allows Actix Web to automatically translate `AppError`
/// results from handlers into the correct HTTP status codes and JSON error responses.
impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::Unauthorized(msg) => HttpResponse::Unauthorized().json(json!({
                "error": msg
            })),
            AppError::BadRequest(msg) => HttpResponse::BadRequest().json(json!({
                "error": msg
            })),
            AppError::NotFound(msg) => HttpResponse::NotFound().json(json!({
                "error": msg
            })),
            AppError::InternalServerError(msg) => HttpResponse::InternalServerError().json(json!({
                "error": msg
            })),
            // Database errors are also presented as generic internal server errors to the client.
            AppError::DatabaseError(msg) => HttpResponse::InternalServerError().json(json!({
                "error": msg
            })),
            AppError::ValidationError(msg) => HttpResponse::UnprocessableEntity().json(json!({
                "error": msg
            })),
        }
    }
}

/// Converts `sqlx::Error` into `AppError`.
///
/// Specific cases like `sqlx::Error::RowNotFound` are mapped to `AppError::NotFound`,
/// while other database errors become `AppError::DatabaseError`.
impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> AppError {
        match error {
            sqlx::Error::RowNotFound => AppError::NotFound("Record not found".into()),
            sqlx::Error::Database(db_err) => {
                // db_err is Box<dyn sqlx::error::DatabaseError + ...>
                match db_err.code() {
                    // db_err.code() is Option<Cow<'_, str>>
                    Some(code_cow) => {
                        let code_str = code_cow.as_ref(); // code_str is &str
                        if code_str == "23505" {
                            // Unique violation
                            if let Some(constraint_cow) = db_err.constraint() {
                                // constraint_cow is Cow<'_, str>
                                let constraint_str: &str = constraint_cow.as_ref(); // constraint_str is &str
                                if constraint_str.contains("username") {
                                    return AppError::BadRequest("Username already taken".into());
                                }
                                if constraint_str.contains("email") {
                                    return AppError::BadRequest("Email already registered".into());
                                }
                            }
                            // Generic unique violation message if constraint name doesn't give more info
                            return AppError::BadRequest(
                                "A unique value constraint was violated".into(),
                            );
                        }
                        // Fallback for other DB error codes
                        AppError::DatabaseError(db_err.to_string())
                    }
                    None => {
                        // No error code available from the DatabaseError trait
                        AppError::DatabaseError(db_err.to_string())
                    }
                }
            }
            _ => AppError::DatabaseError(error.to_string()), // For other sqlx::Error variants
        }
    }
}

/// Converts `validator::ValidationErrors` into `AppError::ValidationError`.
///
/// The detailed validation messages are preserved.
impl From<ValidationErrors> for AppError {
    fn from(error: ValidationErrors) -> AppError {
        AppError::ValidationError(error.to_string())
    }
}

/// Converts `jsonwebtoken::errors::Error` into `AppError::Unauthorized`.
///
/// This is typically used when JWT processing (e.g., verification) fails.
impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(error: jsonwebtoken::errors::Error) -> AppError {
        AppError::Unauthorized(error.to_string())
    }
}

/// Converts `bcrypt::BcryptError` into `AppError::InternalServerError`.
///
/// This handles errors during password hashing or verification.
impl From<bcrypt::BcryptError> for AppError {
    fn from(error: bcrypt::BcryptError) -> AppError {
        AppError::InternalServerError(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::body::to_bytes;
    use actix_web::http::StatusCode;
    use serde_json::Value;
    use validator::Validate;

    #[test]
    fn test_app_error_display() {
        assert_eq!(
            AppError::Unauthorized("test".into()).to_string(),
            "Unauthorized: test"
        );
        assert_eq!(
            AppError::BadRequest("test".into()).to_string(),
            "Bad Request: test"
        );
        assert_eq!(
            AppError::NotFound("test".into()).to_string(),
            "Not Found: test"
        );
        assert_eq!(
            AppError::InternalServerError("test".into()).to_string(),
            "Internal Server Error: test"
        );
        assert_eq!(
            AppError::DatabaseError("test".into()).to_string(),
            "Database Error: test"
        );
        assert_eq!(
            AppError::ValidationError("test".into()).to_string(),
            "Validation Error: test"
        );
    }

    #[actix_web::test]
    async fn test_error_responses() {
        let test_cases = vec![
            (
                AppError::Unauthorized("Invalid token".into()),
                StatusCode::UNAUTHORIZED,
                json!({"error": "Invalid token"}),
            ),
            (
                AppError::BadRequest("Invalid input".into()),
                StatusCode::BAD_REQUEST,
                json!({"error": "Invalid input"}),
            ),
            (
                AppError::NotFound("Resource not found".into()),
                StatusCode::NOT_FOUND,
                json!({"error": "Resource not found"}),
            ),
            (
                AppError::InternalServerError("Server error".into()),
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"error": "Server error"}),
            ),
            (
                AppError::DatabaseError("DB issue".into()),
                StatusCode::INTERNAL_SERVER_ERROR, // As per impl, DatabaseError maps to 500
                json!({"error": "DB issue"}),
            ),
            (
                AppError::ValidationError("Validation failed".into()),
                StatusCode::UNPROCESSABLE_ENTITY,
                json!({"error": "Validation failed"}),
            ),
        ];

        for (error, expected_status, expected_body) in test_cases {
            let response = error.error_response();
            assert_eq!(response.status(), expected_status);

            let body = response.into_body();
            let bytes = match to_bytes(body).await {
                Ok(bytes) => bytes,
                Err(_) => panic!("Failed to convert body to bytes for error: {:?}", error),
            };
            let body_json: Value =
                serde_json::from_slice(&bytes).expect("Response body was not valid JSON");
            assert_eq!(body_json, expected_body);
        }
    }

    // Helper struct for testing From<ValidationErrors>
    #[derive(Debug, Validate)]
    struct TestInput {
        #[validate(length(min = 5))]
        field: String,
    }

    #[test]
    fn test_from_validation_errors() {
        let input = TestInput {
            field: "abc".to_string(), // Fails validation (min length 5)
        };
        let validation_errors = input.validate().unwrap_err();
        let app_error: AppError = validation_errors.into();
        match app_error {
            AppError::ValidationError(msg) => {
                // Removed eprintln!
                // Corrected assertion based on actual error message format
                assert!(msg.contains("field: Validation error: length"));
            }
            _ => panic!("Expected AppError::ValidationError, got {:?}", app_error),
        }
    }

    #[test]
    fn test_from_jwt_error() {
        let jwt_error_kind = jsonwebtoken::errors::ErrorKind::InvalidToken;
        let jwt_error = jsonwebtoken::errors::Error::from(jwt_error_kind);
        let app_error: AppError = jwt_error.into(); // Relies on our From impl
        match app_error {
            AppError::Unauthorized(msg) => {
                // Check that the message from our From impl is related to the original error
                assert!(msg.contains("InvalidToken"));
            }
            _ => panic!(
                "Expected AppError::Unauthorized for jwt error, got {:?}",
                app_error
            ),
        }
    }

    #[test]
    fn test_from_bcrypt_error() {
        // bcrypt::BcryptError is an opaque struct.
        // We trigger it by trying to verify a password with a malformed hash.
        let malformed_hash = "$2b$12$thisisnotavalidbcrypthash"; // Example of a malformed hash
        let bcrypt_result = bcrypt::verify("anypassword", malformed_hash);

        assert!(
            bcrypt_result.is_err(),
            "bcrypt::verify should fail with a malformed hash."
        );

        if let Err(bcrypt_err) = bcrypt_result {
            let app_error: AppError = bcrypt_err.into(); // Relies on our From impl
            match app_error {
                AppError::InternalServerError(msg) => {
                    // The exact message from bcrypt::Error::to_string() can be generic.
                    // We're ensuring our From impl correctly wraps it.
                    // bcrypt might output "invalid hash" or a similar message.
                    assert!(
                        !msg.is_empty(),
                        "Error message from bcrypt error should not be empty"
                    );
                }
                _ => panic!(
                    "Expected AppError::InternalServerError for bcrypt error, got {:?}",
                    app_error
                ),
            }
        } else {
            panic!("bcrypt::verify did not return an error as expected for a malformed hash.");
        }
    }

    #[test]
    fn test_from_sqlx_error_variants() {
        // Test sqlx::Error::RowNotFound
        let row_not_found_err = sqlx::Error::RowNotFound;
        let app_error_not_found: AppError = row_not_found_err.into();
        match app_error_not_found {
            AppError::NotFound(msg) => {
                assert_eq!(msg, "Record not found");
            }
            _ => panic!(
                "Expected AppError::NotFound for sqlx::Error::RowNotFound, got {:?}",
                app_error_not_found
            ),
        }

        // --- Tests for sqlx::Error::Database with MockPgError ---
        #[derive(Debug)]
        struct MockPgError {
            code: String,
            message: String,
            constraint: Option<String>,
        }

        impl MockPgError {
            fn new(code: &str, message: &str, constraint: Option<&str>) -> Self {
                MockPgError {
                    code: code.to_string(),
                    message: message.to_string(),
                    constraint: constraint.map(String::from),
                }
            }
        }

        impl std::fmt::Display for MockPgError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "SQLSTATE {}: {}", self.code, self.message)
            }
        }

        impl std::error::Error for MockPgError {}

        impl sqlx::error::DatabaseError for MockPgError {
            fn message(&self) -> &str {
                &self.message
            }

            fn kind(&self) -> sqlx::error::ErrorKind {
                sqlx::error::ErrorKind::Other // Generic kind for mock
            }

            fn as_error(&self) -> &(dyn std::error::Error + Send + Sync + 'static) {
                self
            }

            fn as_error_mut(&mut self) -> &mut (dyn std::error::Error + Send + Sync + 'static) {
                self
            }

            fn into_error(self: Box<Self>) -> Box<dyn std::error::Error + Send + Sync + 'static> {
                self
            }

            // Implement methods to mimic PgDatabaseError for testing purposes
            fn code(&self) -> Option<std::borrow::Cow<'_, str>> {
                Some(std::borrow::Cow::Owned(self.code.clone()))
            }

            fn constraint(&self) -> Option<&str> {
                self.constraint.as_deref()
            }
        }

        // Helper to create a sqlx::Error::Database from MockPgError
        // This also ensures our From impl actually tries to downcast to PgDatabaseError
        // by tricking it with a type that looks like PgDatabaseError but isn't actually it.
        // The `try_downcast_ref` in the main code relies on the actual type,
        // so we need to ensure our mock *is* the error trait object directly.
        // The correct way is to ensure the `db_err` in `from()` *is* our mock directly,
        // and that our mock provides the methods `code()` and `constraint()`.

        // Test sqlx::Error::Database for PgDatabaseError - Unique Violation (username)
        let mock_pg_unique_username_error =
            MockPgError::new("23505", "unique_violation", Some("users_username_key"));
        let db_error_username_taken: sqlx::Error =
            sqlx::Error::Database(Box::new(mock_pg_unique_username_error));
        let app_error_username_taken: AppError = db_error_username_taken.into();
        match app_error_username_taken {
            AppError::BadRequest(msg) => {
                assert_eq!(msg, "Username already taken");
            }
            _ => panic!(
                "Expected AppError::BadRequest for username unique violation, got {:?}",
                app_error_username_taken
            ),
        }

        // Test sqlx::Error::Database for PgDatabaseError - Unique Violation (email)
        let mock_pg_unique_email_error =
            MockPgError::new("23505", "unique_violation", Some("users_email_key"));
        let db_error_email_taken: sqlx::Error =
            sqlx::Error::Database(Box::new(mock_pg_unique_email_error));
        let app_error_email_taken: AppError = db_error_email_taken.into();
        match app_error_email_taken {
            AppError::BadRequest(msg) => {
                assert_eq!(msg, "Email already registered");
            }
            _ => panic!(
                "Expected AppError::BadRequest for email unique violation, got {:?}",
                app_error_email_taken
            ),
        }

        // Test sqlx::Error::Database for PgDatabaseError - Unique Violation (generic constraint)
        let mock_pg_unique_generic_error =
            MockPgError::new("23505", "unique_violation", Some("some_other_unique_key"));
        let db_error_generic_unique: sqlx::Error =
            sqlx::Error::Database(Box::new(mock_pg_unique_generic_error));
        let app_error_generic_unique: AppError = db_error_generic_unique.into();
        match app_error_generic_unique {
            AppError::BadRequest(msg) => {
                assert_eq!(msg, "A unique value constraint was violated");
            }
            _ => panic!(
                "Expected AppError::BadRequest for generic unique violation, got {:?}",
                app_error_generic_unique
            ),
        }

        // Test sqlx::Error::Database for PgDatabaseError - Other PG Error (not 23505)
        let other_pg_error_code = "22007"; // Example: invalid_datetime_format
        let other_pg_error_message = "Invalid datetime format simulated".to_string();
        let mock_pg_other_error =
            MockPgError::new(other_pg_error_code, &other_pg_error_message, None);
        let db_error_other_pg: sqlx::Error = sqlx::Error::Database(Box::new(mock_pg_other_error));
        let app_error_other_pg: AppError = db_error_other_pg.into();
        match app_error_other_pg {
            AppError::DatabaseError(msg) => {
                // The message from pg_err.to_string() is `SQLSTATE <code>: <message>`
                assert!(msg.contains(other_pg_error_code));
                assert!(msg.contains(&other_pg_error_message));
            }
            _ => panic!(
                "Expected AppError::DatabaseError for other PG error, got {:?}",
                app_error_other_pg
            ),
        }
        // --- End of MockPgError tests ---

        // Test sqlx::Error::Database for a non-PgDatabaseError (generic DB error)
        // This covers the path where db_err.try_downcast_ref::<sqlx::postgres::PgDatabaseError>() is None
        #[derive(Debug)]
        struct MockNonPgError(String);
        impl std::fmt::Display for MockNonPgError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
        impl std::error::Error for MockNonPgError {}
        impl sqlx::error::DatabaseError for MockNonPgError {
            fn message(&self) -> &str {
                &self.0
            }
            fn kind(&self) -> sqlx::error::ErrorKind {
                sqlx::error::ErrorKind::Other
            }
            fn as_error(&self) -> &(dyn std::error::Error + Send + Sync + 'static) {
                self
            }
            fn as_error_mut(&mut self) -> &mut (dyn std::error::Error + Send + Sync + 'static) {
                self
            }
            fn into_error(self: Box<Self>) -> Box<dyn std::error::Error + Send + Sync + 'static> {
                self
            }
        }
        let non_pg_db_error_str = "Simulated non-PG database error".to_string();
        let non_pg_db_err =
            sqlx::Error::Database(Box::new(MockNonPgError(non_pg_db_error_str.clone())));
        let app_error_non_pg_db: AppError = non_pg_db_err.into();
        match app_error_non_pg_db {
            AppError::DatabaseError(msg) => {
                assert!(msg.contains(&non_pg_db_error_str));
            }
            _ => panic!(
                "Expected AppError::DatabaseError for non-PG DB error, got {:?}",
                app_error_non_pg_db
            ),
        }

        // Test a generic sqlx::Error (e.g., Configuration) to cover the general fallback (_ case)
        let config_error_str = "Simulated config error".to_string();
        let config_err = sqlx::Error::Configuration(config_error_str.clone().into());
        let app_error_config: AppError = config_err.into();
        match app_error_config {
            AppError::DatabaseError(msg) => {
                assert!(msg.contains(&config_error_str));
            }
            _ => panic!(
                "Expected AppError::DatabaseError for sqlx::Error::Configuration, got {:?}",
                app_error_config
            ),
        }
    }
}
