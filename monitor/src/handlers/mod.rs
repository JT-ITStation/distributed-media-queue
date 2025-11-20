use crate::dtos::{DashboardStats, QueueLengths, TaskSummary};
use crate::state::AppState;
use axum::{extract::State, Json};
use shared::Task;
use std::sync::Arc;

/// GET /api/stats - Statistiques globales
pub async fn get_stats(
    State(state): State<Arc<AppState>>,
) -> Json<DashboardStats> {
    let db = state.get_database();
    let collection = db.collection::<Task>("tasks");
    
    // Compter par statut
    let total_tasks = collection.count_documents(mongodb::bson::doc! {}, None)
        .await
        .unwrap_or(0);
    
    let pending = collection
        .count_documents(mongodb::bson::doc! { "status": "pending" }, None)
        .await
        .unwrap_or(0);
    
    let processing = collection
        .count_documents(mongodb::bson::doc! { "status": "processing" }, None)
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
    
    // Longueurs des queues Redis
    let queue_lengths = get_queue_lengths(&state).await;
    
    Json(DashboardStats {
        total_tasks,
        pending_tasks: pending,
        processing_tasks: processing,
        completed_tasks: completed,
        failed_tasks: failed,
        cancelled_tasks: cancelled,
        queue_lengths,
    })
}

/// GET /api/tasks/recent - Tâches récentes
pub async fn get_recent_tasks(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<TaskSummary>> {
    let db = state.get_database();
    let collection = db.collection::<Task>("tasks");
    
    let options = mongodb::options::FindOptions::builder()
        .sort(mongodb::bson::doc! { "created_at": -1 })
        .limit(50)
        .build();
    
    let mut cursor = collection
        .find(mongodb::bson::doc! {}, options)
        .await
        .unwrap();
    
    let mut tasks = Vec::new();
    
    use futures::stream::StreamExt;
    while let Some(result) = cursor.next().await {
        if let Ok(task) = result {
            tasks.push(TaskSummary {
                id: task.id,
                task_type: task.task_type.to_string(),
                status: task.status.to_string(),
                progress: task.progress,
                created_at: task.created_at.to_rfc3339(),
                updated_at: task.updated_at.to_rfc3339(),
                error: task.error,
            });
        }
    }
    
    Json(tasks)
}

async fn get_queue_lengths(state: &AppState) -> QueueLengths {
    let mut conn = match state.redis_client.get_async_connection().await {
        Ok(c) => c,
        Err(_) => {
            return QueueLengths {
                video: 0,
                audio: 0,
                image: 0,
            }
        }
    };
    
    let video: i64 = redis::cmd("LLEN")
        .arg("queue:video")
        .query_async(&mut conn)
        .await
        .unwrap_or(0);
    
    let audio: i64 = redis::cmd("LLEN")
        .arg("queue:audio")
        .query_async(&mut conn)
        .await
        .unwrap_or(0);
    
    let image: i64 = redis::cmd("LLEN")
        .arg("queue:image")
        .query_async(&mut conn)
        .await
        .unwrap_or(0);
    
    QueueLengths {
        video,
        audio,
        image,
    }
}
