use actix_web::dev::Payload;
use actix_web::{Error as ActixError, FromRequest, HttpMessage, HttpRequest};
use std::future::{ready, Ready};

use crate::error::AppError;

/// Extracts the authenticated user's ID from request extensions.
///
/// This extractor is intended to be used on routes protected by `AuthMiddleware`,
/// which is responsible for validating the JWT and inserting the user's ID into
/// request extensions.
///
/// If the user ID is not found in the extensions (e.g., if `AuthMiddleware` did not run
/// or failed to insert it), this extractor will return an `AppError::Unauthorized` error.
#[derive(Debug, Clone, Copy)]
pub struct AuthenticatedUserId(pub i32);

impl FromRequest for AuthenticatedUserId {
    type Error = ActixError; // AppError will be converted into ActixError via ResponseError
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        // eprintln!(
        //     "[DEBUG EXTRACTOR] Attempting to extract user_id from extensions for path: {}. Extensions available: {:?}",
        //     req.path(),
        //     req.extensions()
        // );
        match req.extensions().get::<i32>().cloned() {
            Some(user_id) => ready(Ok(AuthenticatedUserId(user_id))),
            None => {
                // This case should ideally not be reached if AuthMiddleware is correctly applied
                // and has successfully inserted the user_id. If it's missing, it implies
                // an issue with middleware setup or an internal logic error after auth.
                // Responding with Unauthorized is a safe default.
                let err = AppError::Unauthorized(
                    "User ID not found in request. Ensure AuthMiddleware is active.".to_string(),
                );
                ready(Err(err.into())) // Convert AppError to ActixError
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::dev::Payload;
    use actix_web::http::StatusCode;
    use actix_web::test;

    #[actix_rt::test]
    async fn test_authenticated_user_id_extractor_success() {
        let req = test::TestRequest::default().to_http_request();
        req.extensions_mut().insert(123_i32); // HttpMessage trait brings .extensions_mut()

        let mut payload = Payload::None;
        let extracted_id = AuthenticatedUserId::from_request(&req, &mut payload).await;
        assert!(extracted_id.is_ok());
        assert_eq!(extracted_id.unwrap().0, 123);
    }

    #[actix_rt::test]
    async fn test_authenticated_user_id_extractor_failure() {
        let req = test::TestRequest::default().to_http_request();
        // No user_id inserted into extensions

        let mut payload = Payload::None;
        let extracted_id_result = AuthenticatedUserId::from_request(&req, &mut payload).await;
        assert!(extracted_id_result.is_err());

        let err = extracted_id_result.unwrap_err();
        let response = err.error_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
