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
            _ => AppError::DatabaseError(error.to_string()),
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

    #[test]
    fn test_error_responses() {
        // Test Unauthorized
        let error = AppError::Unauthorized("Invalid token".into());
        let response = error.error_response();
        assert_eq!(response.status(), 401);

        // Test BadRequest
        let error = AppError::BadRequest("Invalid input".into());
        let response = error.error_response();
        assert_eq!(response.status(), 400);

        // Test NotFound
        let error = AppError::NotFound("Resource not found".into());
        let response = error.error_response();
        assert_eq!(response.status(), 404);

        // Test InternalServerError
        let error = AppError::InternalServerError("Server error".into());
        let response = error.error_response();
        assert_eq!(response.status(), 500);
    }
}
