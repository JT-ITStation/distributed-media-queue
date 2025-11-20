use crate::dtos::{ApiResponse, CreateTaskDto, CreateTaskResponse, TaskResponse};
use crate::error::ApiError;
use crate::services;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

pub async fn create_task(
    State(state): State<Arc<AppState>>,
    Json(dto): Json<CreateTaskDto>,
) -> Result<Json<CreateTaskResponse>, ApiError> {
    tracing::info!("Creating task: {:?}", dto.task_type);
    
    let task_id = services::create_task(&state, dto).await?;
    
    Ok(Json(CreateTaskResponse {
        success: true,
        task_id,
        message: "Task created and queued successfully".to_string(),
    }))
}

pub async fn get_task(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<String>,
) -> Result<Json<ApiResponse<TaskResponse>>, ApiError> {
    tracing::debug!("Getting task: {}", task_id);
    
    let task = services::get_task(&state, &task_id).await?;
    
    Ok(Json(ApiResponse::success(task)))
}

#[derive(Deserialize)]
pub struct ListTasksQuery {
    pub status: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub skip: u64,
}

fn default_limit() -> i64 {
    50
}

pub async fn list_tasks(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListTasksQuery>,
) -> Result<Json<ApiResponse<Vec<TaskResponse>>>, ApiError> {
    tracing::debug!("Listing tasks with filters: {:?}", query.status);
    
    let tasks = services::list_tasks(&state, query.status, query.limit, query.skip).await?;
    
    Ok(Json(ApiResponse::success(tasks)))
}

pub async fn cancel_task(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<String>,
) -> Result<Json<ApiResponse<String>>, ApiError> {
    tracing::info!("Cancelling task: {}", task_id);
    
    services::cancel_task(&state, &task_id).await?;
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some("Task cancelled successfully".to_string()),
        message: None,
    }))
}
