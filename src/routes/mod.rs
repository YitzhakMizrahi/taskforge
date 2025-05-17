pub mod auth;
pub mod health;
pub mod tasks;

use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .service(auth::login)
            .service(auth::register),
    )
    .service(
        web::scope("/tasks")
            .service(tasks::get_tasks)
            .service(tasks::create_task)
            .service(tasks::get_task)
            .service(tasks::update_task)
            .service(tasks::delete_task),
    );
}
