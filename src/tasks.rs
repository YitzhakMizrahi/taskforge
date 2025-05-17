use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct TaskInput {
    #[validate(length(min = 1, max = 200))]
    pub title: String,

    #[validate(length(max = 1000))]
    pub description: Option<String>,

    #[validate(length(max = 50))]
    pub priority: Option<String>,

    pub due_date: Option<DateTime<Utc>>,

    #[validate(length(max = 50))]
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub status: String,
    pub due_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: i32,
    pub assigned_to: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskQuery {
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assigned_to: Option<i32>,
    pub created_by: Option<i32>,
    pub search: Option<String>,
}

impl Task {
    pub fn new(input: TaskInput, user_id: i32) -> Self {
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
            created_by: user_id,
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
            priority: Some("High".to_string()),
            status: "Todo".to_string(),
            due_date: Some(Utc::now()),
        };

        let task = Task::new(input, 1);
        assert_eq!(task.title, "Test Task");
        assert_eq!(task.created_by, 1);
        assert!(task.assigned_to.is_none());
    }

    #[test]
    fn test_task_validation() {
        let valid_input = TaskInput {
            title: "Valid Task".to_string(),
            description: Some("Valid Description".to_string()),
            priority: Some("High".to_string()),
            status: "Todo".to_string(),
            due_date: Some(Utc::now()),
        };
        assert!(valid_input.validate().is_ok());

        let invalid_input = TaskInput {
            title: "".to_string(), // Empty title
            description: Some("Valid Description".to_string()),
            priority: Some("High".to_string()),
            status: "Todo".to_string(),
            due_date: Some(Utc::now()),
        };
        assert!(invalid_input.validate().is_err());
    }
}
