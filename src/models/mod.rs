//! # Models Module
//!
//! This module defines data structures (structs and enums) that represent
//! the core entities of the TaskForge application, such as users and tasks.
//! It also includes input structures for data validation and query structures
//! for database interactions.

pub mod task;
pub mod user;

pub use task::{Task, TaskInput, TaskPriority, TaskQuery, TaskStatus};
pub use user::{User, UserInput};
