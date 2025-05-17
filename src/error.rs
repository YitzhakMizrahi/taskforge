use actix_web::{error::ResponseError, HttpResponse};
use serde_json::json;
use std::fmt;
use validator::ValidationErrors;

#[derive(Debug)]
pub enum AppError {
    Unauthorized(String),
    BadRequest(String),
    NotFound(String),
    InternalServerError(String),
    DatabaseError(String),
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
            AppError::DatabaseError(msg) => HttpResponse::InternalServerError().json(json!({
                "error": msg
            })),
            AppError::ValidationError(msg) => HttpResponse::UnprocessableEntity().json(json!({
                "error": msg
            })),
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> AppError {
        match error {
            sqlx::Error::RowNotFound => AppError::NotFound("Record not found".into()),
            _ => AppError::DatabaseError(error.to_string()),
        }
    }
}

impl From<ValidationErrors> for AppError {
    fn from(error: ValidationErrors) -> AppError {
        AppError::ValidationError(error.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(error: jsonwebtoken::errors::Error) -> AppError {
        AppError::Unauthorized(error.to_string())
    }
}

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
