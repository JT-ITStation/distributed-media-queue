use crate::handlers;
use crate::state::AppState;
use axum::{
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;
use tower_http::trace::TraceLayer;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(handlers::health_check))
        .route("/metrics", get(handlers::get_metrics))
        .route("/metrics/reset", post(handlers::reset_metrics))
        .route("/metrics/sync", get(handlers::sync_metrics))
        .route("/tasks", post(handlers::create_task))
        .route("/tasks", get(handlers::list_tasks))
        .route("/tasks/:id", get(handlers::get_task))
        .route("/tasks/:id", delete(handlers::cancel_task))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}
