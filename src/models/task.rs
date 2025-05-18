use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Represents the priority of a task.
/// Corresponds to the `task_priority` SQL enum.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "task_priority", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum TaskPriority {
    /// Low priority.
    Low,
    /// Medium priority.
    Medium,
    /// High priority.
    High,
    /// Urgent priority.
    Urgent,
}

/// Represents the status of a task.
/// Corresponds to the `task_status` SQL enum.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "task_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// Task is yet to be started.
    Todo,
    /// Task is currently being worked on.
    InProgress,
    /// Task is completed and under review.
    Review,
    /// Task is completed.
    Done,
}

/// Input structure for creating or updating a task.
/// Contains validation rules for its fields.
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct TaskInput {
    /// The title of the task.
    /// Must be between 1 and 200 characters.
    #[validate(length(min = 1, max = 200))]
    pub title: String,

    /// An optional description for the task.
    /// Maximum length of 1000 characters if provided.
    #[validate(length(max = 1000))]
    pub description: Option<String>,

    /// The priority of the task. Optional for updates, may be set by default on creation if not provided.
    pub priority: Option<TaskPriority>,

    /// Optional due date for the task.
    pub due_date: Option<DateTime<Utc>>,

    /// The current status of the task.
    pub status: TaskStatus,
}

/// Represents a task entity as stored in the database and returned by the API.
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Task {
    /// Unique identifier for the task (UUID v4).
    pub id: Uuid,
    /// The title of the task.
    pub title: String,
    /// An optional description for the task.
    pub description: Option<String>,
    /// The priority of the task.
    pub priority: Option<TaskPriority>,
    /// The current status of the task.
    pub status: TaskStatus,
    /// Optional due date for the task.
    pub due_date: Option<DateTime<Utc>>,
    /// Timestamp of when the task was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp of the last update to the task.
    pub updated_at: DateTime<Utc>,
    /// Identifier of the user who owns/created the task.
    pub user_id: i32,
    /// Identifier of the user to whom the task is assigned (optional).
    pub assigned_to: Option<i32>,
}

/// Represents query parameters for filtering tasks when listing them.
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskQuery {
    /// Filter tasks by status.
    pub status: Option<TaskStatus>,
    /// Filter tasks by priority.
    pub priority: Option<TaskPriority>,
    /// Filter tasks by assignee's user ID.
    pub assigned_to: Option<i32>,
    /// Filter tasks by creator's user ID. (Note: listing tasks is already scoped to the authenticated user).
    pub user_id: Option<i32>,
    /// Search term to filter tasks by title or description (case-insensitive).
    pub search: Option<String>,
}

impl Task {
    /// Creates a new `Task` instance from `TaskInput` and the creator's `user_id`.
    /// Sets `created_at`, `updated_at` to the current time, and `id` to a new UUID.
    /// `assigned_to` is initialized to `None`.
    pub fn new(input: TaskInput, user_id_param: i32) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title: input.title,
            description: input.description,
            priority: input.priority,
            status: input.status,
            due_date: input.due_date,
            created_at: now,
            updated_at: now,
            user_id: user_id_param,
            assigned_to: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let input = TaskInput {
            title: "Test Task".to_string(),
            description: Some("Test Description".to_string()),
            priority: Some(TaskPriority::High),
            status: TaskStatus::Todo,
            due_date: Some(Utc::now()),
        };

        let task = Task::new(input, 1);
        assert_eq!(task.title, "Test Task");
        assert_eq!(task.user_id, 1);
        assert!(task.assigned_to.is_none());
    }

    #[test]
    fn test_task_validation() {
        let valid_input = TaskInput {
            title: "Valid Task".to_string(),
            description: Some("Valid Description".to_string()),
            priority: Some(TaskPriority::High),
            status: TaskStatus::Todo,
            due_date: Some(Utc::now()),
        };
        assert!(valid_input.validate().is_ok());

        let invalid_input = TaskInput {
            title: "".to_string(), // Empty title
            description: Some("Valid Description".to_string()),
            priority: Some(TaskPriority::High),
            status: TaskStatus::Todo,
            due_date: Some(Utc::now()),
        };
        assert!(invalid_input.validate().is_err());
    }
}
