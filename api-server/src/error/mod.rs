use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Task not found: {0}")]
    TaskNotFound(String),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] mongodb::error::Error),
    
    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),
    
    #[error("Internal server error: {0}")]
    InternalError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::TaskNotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::DatabaseError(err) => {
                tracing::error!("Database error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred".to_string())
            }
            ApiError::RedisError(err) => {
                tracing::error!("Redis error: {}", err);
                (StatusCode::SERVICE_UNAVAILABLE, "Queue service unavailable".to_string())
            }
            ApiError::InternalError(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
            ApiError::SerializationError(err) => {
                tracing::error!("Serialization error: {}", err);
                (StatusCode::BAD_REQUEST, "Invalid data format".to_string())
            }
        };

        let body = Json(json!({
            "success": false,
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
