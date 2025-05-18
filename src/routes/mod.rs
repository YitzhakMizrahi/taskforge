//! # Route Definitions
//!
//! This module consolidates all route handlers for the application and provides
//! a configuration function to register them with an Actix Web App.
//!
//! It organizes routes into submodules for better structure:
//! - `auth`: Handles user authentication (registration, login).
//! - `health`: Provides health check endpoints.
//! - `tasks`: Manages task creation, retrieval, updates, and deletion.

pub mod auth;
pub mod health;
pub mod tasks;

use actix_web::web;

/// Configures and registers all application routes.
///
/// This function is intended to be used during Actix Web application setup.
/// It scopes routes under `/api` and then further under specific modules
/// like `/api/auth` and `/api/tasks`.
///
/// # Arguments
///
/// * `cfg` - A mutable reference to Actix Web's `ServiceConfig` to which
///   the routes will be added.
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
