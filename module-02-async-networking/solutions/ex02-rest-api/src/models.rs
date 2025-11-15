use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: Priority,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(rename_all = "PascalCase")]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Todo => write!(f, "Todo"),
            TaskStatus::InProgress => write!(f, "InProgress"),
            TaskStatus::Done => write!(f, "Done"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(rename_all = "PascalCase")]
pub enum Priority {
    Low,
    Medium,
    High,
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::Low => write!(f, "Low"),
            Priority::Medium => write!(f, "Medium"),
            Priority::High => write!(f, "High"),
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTaskRequest {
    #[validate(length(min = 1, max = 200, message = "Title must be between 1 and 200 characters"))]
    pub title: String,

    #[validate(length(max = 1000, message = "Description must be at most 1000 characters"))]
    pub description: Option<String>,

    pub status: Option<TaskStatus>,
    pub priority: Option<Priority>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateTaskRequest {
    #[validate(length(min = 1, max = 200, message = "Title must be between 1 and 200 characters"))]
    pub title: Option<String>,

    #[validate(length(max = 1000, message = "Description must be at most 1000 characters"))]
    pub description: Option<String>,

    pub status: Option<TaskStatus>,
    pub priority: Option<Priority>,
}

#[derive(Debug, Deserialize)]
pub struct ListTasksQuery {
    pub status: Option<TaskStatus>,
    pub priority: Option<Priority>,
    pub completed: Option<bool>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

impl Default for ListTasksQuery {
    fn default() -> Self {
        Self {
            status: None,
            priority: None,
            completed: None,
            page: Some(1),
            per_page: Some(10),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TaskListResponse {
    pub tasks: Vec<Task>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_create_task_validation() {
        let valid_request = CreateTaskRequest {
            title: "Valid title".to_string(),
            description: Some("Description".to_string()),
            status: Some(TaskStatus::Todo),
            priority: Some(Priority::Medium),
        };
        assert!(valid_request.validate().is_ok());

        let empty_title = CreateTaskRequest {
            title: "".to_string(),
            description: None,
            status: None,
            priority: None,
        };
        assert!(empty_title.validate().is_err());

        let long_title = CreateTaskRequest {
            title: "a".repeat(201),
            description: None,
            status: None,
            priority: None,
        };
        assert!(long_title.validate().is_err());
    }

    #[test]
    fn test_task_status_display() {
        assert_eq!(TaskStatus::Todo.to_string(), "Todo");
        assert_eq!(TaskStatus::InProgress.to_string(), "InProgress");
        assert_eq!(TaskStatus::Done.to_string(), "Done");
    }
}
