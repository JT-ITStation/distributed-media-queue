use crate::state::AppState;
use axum::{extract::State, Json};
use serde_json::{json, Value};
use std::sync::Arc;

pub async fn health_check(
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let mongo_healthy = check_mongo(&state).await;
    let redis_healthy = check_redis(&state).await;
    
    let overall_status = if mongo_healthy && redis_healthy {
        "healthy"
    } else {
        "unhealthy"
    };
    
    Json(json!({
        "status": overall_status,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "checks": {
            "mongodb": if mongo_healthy { "healthy" } else { "unhealthy" },
            "redis": if redis_healthy { "healthy" } else { "unhealthy" },
        }
    }))
}

async fn check_mongo(state: &AppState) -> bool {
    state
        .mongo_client
        .database("admin")
        .run_command(mongodb::bson::doc! { "ping": 1 }, None)
        .await
        .is_ok()
}

async fn check_redis(state: &AppState) -> bool {
    let mut conn = match state.redis_client.get_async_connection().await {
        Ok(conn) => conn,
        Err(_) => return false,
    };
    
    let result: Result<String, _> = redis::cmd("PING")
        .query_async(&mut conn)
        .await;
    
    result.is_ok()
}
