use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures::future::{ready, LocalBoxFuture, Ready};

use crate::auth::token::verify_token;

/// Authentication middleware factory.
///
/// This middleware is responsible for checking the `Authorization` header
/// for a Bearer token and verifying it. If the token is valid, the claims
/// are inserted into the request extensions for later use by handlers.
///
/// Certain paths like `/health`, `/api/auth/login`, and `/api/auth/register`
/// are excluded from authentication checks.
pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService { service }))
    }
}

/// Authentication middleware service.
///
/// This service is created by `AuthMiddleware` and performs the actual
/// authentication logic for each request before passing it to the next service.
pub struct AuthMiddlewareService<S> {
    /// The next service in the Actix Web processing chain.
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Skip authentication for health check and auth endpoints
        let path = req.path();
        if path == "/health"
            || path.starts_with("/api/auth/login")
            || path.starts_with("/api/auth/register")
        {
            let fut = self.service.call(req);
            return Box::pin(fut);
        }

        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "));

        match auth_header {
            Some(token) => {
                match verify_token(token) {
                    // verify_token returns Result<Claims, AppError>
                    Ok(claims) => {
                        let user_id_to_insert = claims.sub;
                        req.extensions_mut().insert(user_id_to_insert);
                        let fut = self.service.call(req);
                        Box::pin(fut)
                    }
                    Err(app_err) => {
                        // app_err is AppError
                        Box::pin(async move { Err(app_err.into()) }) // Convert AppError to actix_web::Error
                    }
                }
            }
            None => {
                let app_err = crate::error::AppError::Unauthorized("Missing token".into());
                Box::pin(async move { Err(app_err.into()) }) // Convert AppError to actix_web::Error
            }
        }
    }
}
