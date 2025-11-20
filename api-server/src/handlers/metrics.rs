use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use serde_json::json;
use std::sync::Arc;

/// GET /metrics - Expose metrics in Prometheus format
pub async fn get_metrics(State(state): State<Arc<AppState>>) -> String {
    let created = state.metrics.get_created();
    let completed = state.metrics.get_completed();
    let failed = state.metrics.get_failed();
    let cancelled = state.metrics.get_cancelled();
    
    format!(
        "# HELP tasks_total Total number of tasks by status\n\
         # TYPE tasks_total counter\n\
         tasks_total{{status=\"created\"}} {}\n\
         tasks_total{{status=\"completed\"}} {}\n\
         tasks_total{{status=\"failed\"}} {}\n\
         tasks_total{{status=\"cancelled\"}} {}\n",
        created, completed, failed, cancelled
    )
}

/// POST /metrics/reset - Reset all metrics counters to zero
pub async fn reset_metrics(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    state.metrics.reset_all();
    
    Json(json!({
        "message": "Metrics reset successfully",
        "created": 0,
        "completed": 0,
        "failed": 0,
        "cancelled": 0
    }))
}

/// GET /metrics/sync - Synchronize metrics with MongoDB
pub async fn sync_metrics(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let db = state.get_database();
    let collection = db.collection::<shared::Task>("tasks");
    
    // Compter par statut
    let created = collection.count_documents(mongodb::bson::doc! {}, None)
        .await
        .unwrap_or(0);
    
    let completed = collection
        .count_documents(mongodb::bson::doc! { "status": "completed" }, None)
        .await
        .unwrap_or(0);
    
    let failed = collection
        .count_documents(mongodb::bson::doc! { "status": "failed" }, None)
        .await
        .unwrap_or(0);
    
    let cancelled = collection
        .count_documents(mongodb::bson::doc! { "status": "cancelled" }, None)
        .await
        .unwrap_or(0);
    
    // Reset puis set aux valeurs de la DB
    state.metrics.reset_all();
    state.metrics.set_created(created);
    state.metrics.set_completed(completed);
    state.metrics.set_failed(failed);
    state.metrics.set_cancelled(cancelled);
    
    Json(json!({
        "message": "Metrics synchronized with MongoDB",
        "created": created,
        "completed": completed,
        "failed": failed,
        "cancelled": cancelled
    }))
}
