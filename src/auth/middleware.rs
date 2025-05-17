use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures::future::{ready, LocalBoxFuture, Ready};

use crate::auth::token::verify_token;

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

pub struct AuthMiddlewareService<S> {
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
                        req.extensions_mut().insert(claims);
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
