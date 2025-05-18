//! # Route Definitions
//!
//! This module consolidates API route handlers for the application and provides
//! a configuration function (`config`) to register them under the `/api` scope
//! with an Actix Web App.
//!
//! It organizes API routes into submodules for better structure:
//! - `auth`: Handles user authentication (registration, login) under `/api/auth`.
//! - `tasks`: Manages task creation, retrieval, updates, and deletion under `/api/tasks`.
//!
//! Health check routes (from the `health` submodule) are typically registered separately
//! at the application root.

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
            .service(tasks::delete_task)
            .service(tasks::assign_task),
    );
}
