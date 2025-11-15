use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Task not found")]
    NotFound,

    #[error("Validation error")]
    Validation(HashMap<String, String>),

    #[error("Internal server error")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message, details) = match self {
            AppError::Database(ref e) => {
                tracing::error!("Database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error occurred".to_string(),
                    None,
                )
            }
            AppError::NotFound => (
                StatusCode::NOT_FOUND,
                "Resource not found".to_string(),
                None,
            ),
            AppError::Validation(errors) => (
                StatusCode::BAD_REQUEST,
                "Validation failed".to_string(),
                Some(errors),
            ),
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                    None,
                )
            }
        };

        let body = if let Some(details) = details {
            json!({
                "error": error_message,
                "details": details
            })
        } else {
            json!({
                "error": error_message
            })
        };

        (status, Json(body)).into_response()
    }
}

// Convert validation errors to AppError
impl From<validator::ValidationErrors> for AppError {
    fn from(errors: validator::ValidationErrors) -> Self {
        let mut error_map = HashMap::new();

        for (field, errors) in errors.field_errors() {
            if let Some(first_error) = errors.first() {
                let message = first_error
                    .message
                    .as_ref()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| "Validation failed".to_string());
                error_map.insert(field.to_string(), message);
            }
        }

        AppError::Validation(error_map)
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
