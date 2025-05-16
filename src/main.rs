use actix_web::{web, App, HttpResponse, HttpServer, Responder};

async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok"
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    println!("Starting TaskForge server at http://127.0.0.1:8080");
    HttpServer::new(|| App::new().route("/health", web::get().to(health)))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
