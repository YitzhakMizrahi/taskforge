use actix_web::{get, HttpResponse, Responder};
use chrono::Utc;
use serde_json::json;

/// Health check endpoint
///
/// Returns the current status of the API and timestamp.
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
