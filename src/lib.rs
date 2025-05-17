pub mod auth;
pub mod config;
pub mod error;
pub mod models;
pub mod routes;

// lib.rs now primarily declares modules for the library crate.
// The main application setup (app factory) has been moved to main.rs
// to resolve persistent compilation issues related to HttpServiceFactory trait bounds
// when the factory was defined in this library file.

// Re-export key types if desired for easier use of the library crate.
// Example:
// pub use crate::error::AppError;
// pub use crate::models::user::{User, UserInput};
// pub use crate::models::task::{Task, TaskInput, TaskQuery};
