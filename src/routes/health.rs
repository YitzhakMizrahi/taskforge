use actix_web::{get, HttpResponse, Responder};
use chrono::Utc;
use serde_json::json;

/// Health check endpoint.
///
/// Provides a simple health check for the API. It returns a JSON response
/// indicating the operational status and the current server timestamp.
///
/// ## Responses:
/// - `200 OK`: Always returns a success status with a JSON body:
///   ```json
///   {
///     "status": "ok",
///     "timestamp": "2023-10-27T12:00:00.000000Z"
///   }
///   ```
#[get("/health")]
pub async fn health() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "ok",
        "timestamp": Utc::now()
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;

    #[actix_web::test]
    async fn test_health_endpoint() {
        let app = test::init_service(actix_web::App::new().service(health)).await;

        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["status"], "ok");
        assert!(json["timestamp"].is_string());
    }
}
