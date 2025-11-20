use crate::handlers;
use crate::state::AppState;
use axum::{
    routing::get,
    Router,
};
use std::sync::Arc;
use tower_http::{
    services::ServeDir,
    trace::TraceLayer,
};

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        // API routes
        .route("/api/stats", get(handlers::get_stats))
        .route("/api/tasks/recent", get(handlers::get_recent_tasks))
        
        // Serve static files (dashboard HTML/CSS/JS)
        .nest_service("/", ServeDir::new("monitor/static"))
        
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}
