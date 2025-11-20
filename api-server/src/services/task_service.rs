use crate::dtos::{CreateTaskDto, TaskOptionsDto, TaskResponse};
use crate::error::ApiError;
use crate::state::AppState;
use shared::{MediaFile, MediaType, Task, TaskStatus, TaskType};
use std::collections::HashMap;
use std::path::PathBuf;

pub async fn create_task(
    state: &AppState,
    dto: CreateTaskDto,
) -> Result<String, ApiError> {
    // 1. Valider
    validate_task_dto(&dto)?;
    
    // 2. Convertir task_type
    let task_type = parse_task_type(&dto.task_type)?;
    
    // 3. Créer MediaFile
    let mut metadata = HashMap::new();
    add_options_to_metadata(&mut metadata, &dto.options);
    
    let media = MediaFile {
        file_id: uuid::Uuid::new_v4().to_string(),
        file_type: task_type_to_media_type(&task_type),
        file_path: PathBuf::from(&dto.file_path),
        file_size: dto.file_size,
        original_name: dto.original_name,
        mime_type: dto.mime_type,
        metadata,
    };
    
    // 4. Créer Task
    let task = Task::new(task_type.clone(), media);
    let task_id = task.id.clone();
    
    // 5. Sauvegarder MongoDB
    let db = state.get_database();
    let collection = db.collection::<Task>("tasks");
    collection.insert_one(&task, None).await?;
    
    tracing::info!(task_id = %task_id, "Task saved to MongoDB");
    
    // 6. Enqueue Redis
    let mut conn = state.redis_client.get_async_connection().await?;
    let queue_name = format!("queue:{}", task_type);
    let serialized = serde_json::to_string(&task)?;
    
    let _: () = redis::cmd("LPUSH")
        .arg(&queue_name)
        .arg(serialized)
        .query_async(&mut conn)
        .await?;
    
    tracing::info!(task_id = %task_id, queue = %queue_name, "Task enqueued");
    
    // 7. Incrémenter métrique
    state.metrics.increment_created();
    
    Ok(task_id)
}

pub async fn get_task(
    state: &AppState,
    task_id: &str,
) -> Result<TaskResponse, ApiError> {
    let db = state.get_database();
    let collection = db.collection::<Task>("tasks");
    
    let filter = mongodb::bson::doc! { "task_id": task_id };
    let task = collection
        .find_one(filter, None)
        .await?
        .ok_or_else(|| ApiError::TaskNotFound(task_id.to_string()))?;
    
    Ok(task_to_response(task))
}

pub async fn list_tasks(
    state: &AppState,
    status_filter: Option<String>,
    limit: i64,
    skip: u64,
) -> Result<Vec<TaskResponse>, ApiError> {
    let db = state.get_database();
    let collection = db.collection::<Task>("tasks");
    
    let mut filter = mongodb::bson::Document::new();
    if let Some(status) = status_filter {
        filter.insert("status", status);
    }
    
    let options = mongodb::options::FindOptions::builder()
        .limit(limit)
        .skip(skip)
        .sort(mongodb::bson::doc! { "created_at": -1 })
        .build();
    
    let mut cursor = collection.find(filter, options).await?;
    let mut tasks = Vec::new();
    
    use futures::stream::StreamExt;
    while let Some(task) = cursor.next().await {
        tasks.push(task_to_response(task?));
    }
    
    Ok(tasks)
}

pub async fn cancel_task(
    state: &AppState,
    task_id: &str,
) -> Result<(), ApiError> {
    let db = state.get_database();
    let collection = db.collection::<Task>("tasks");
    
    // 1. Récupérer la tâche
    let filter = mongodb::bson::doc! { "task_id": task_id };
    let task = collection
        .find_one(filter.clone(), None)
        .await?
        .ok_or_else(|| ApiError::TaskNotFound(task_id.to_string()))?;
    
    // Vérifier que la tâche peut être annulée
    if task.status == TaskStatus::Completed || task.status == TaskStatus::Failed {
        return Err(ApiError::InvalidInput(
            format!("Cannot cancel task with status: {}", task.status)
        ));
    }
    
    // 2. Publier message de cancellation sur Redis pub/sub
    let channel = format!("task:cancel:{}", task_id);
    let mut conn = state.redis_client.get_async_connection().await?;
    
    let _: i32 = redis::cmd("PUBLISH")
        .arg(&channel)
        .arg("cancel")
        .query_async(&mut conn)
        .await?;
    
    tracing::info!(
        task_id = %task_id,
        channel = %channel,
        "Published cancellation message to Redis"
    );
    
    // 3. Mettre à jour le statut dans MongoDB (Cancelling d'abord)
    let update = mongodb::bson::doc! {
        "$set": {
            "status": "Cancelling",
            "updated_at": mongodb::bson::DateTime::from_millis(chrono::Utc::now().timestamp_millis())
        }
    };
    
    collection.update_one(filter, update, None).await?;
    
    // 4. Nettoyer Redis queue (si la tâche n'a pas encore été prise)
    let removed = remove_task_from_redis_queue(state, &task).await?;
    
    if removed {
        // La tâche était encore en queue, on peut la marquer directement comme Cancelled
        let filter = mongodb::bson::doc! { "task_id": task_id };
        let update = mongodb::bson::doc! {
            "$set": {
                "status": "Cancelled",
                "updated_at": mongodb::bson::DateTime::from_millis(chrono::Utc::now().timestamp_millis())
            }
        };
        collection.update_one(filter, update, None).await?;
        
        tracing::info!(
            task_id = %task_id,
            "Task cancelled and removed from Redis queue (was not processing)"
        );
    } else {
        tracing::info!(
            task_id = %task_id,
            "Task cancellation signal sent (worker will update status to Cancelled)"
        );
    }
    
    // 5. Incrémenter métrique
    state.metrics.increment_cancelled();
    
    Ok(())
}

/// Retire une tâche de la queue Redis
async fn remove_task_from_redis_queue(
    state: &AppState,
    task: &Task,
) -> Result<bool, ApiError> {
    let mut conn = state.redis_client.get_async_connection().await?;
    let queue_name = format!("queue:{}", task.task_type);
    
    // Récupérer toutes les tâches de la queue
    let tasks: Vec<String> = redis::cmd("LRANGE")
        .arg(&queue_name)
        .arg(0)
        .arg(-1)
        .query_async(&mut conn)
        .await?;
    
    // Chercher la tâche à retirer
    for task_json in tasks.iter() {
        if let Ok(t) = serde_json::from_str::<Task>(task_json) {
            if t.id == task.id {
                // Retirer cette occurrence (LREM)
                let count: i32 = redis::cmd("LREM")
                    .arg(&queue_name)
                    .arg(1) // Retirer 1 occurrence
                    .arg(task_json)
                    .query_async(&mut conn)
                    .await?;
                
                tracing::debug!(
                    task_id = %task.id,
                    queue = %queue_name,
                    removed_count = count,
                    "Task removed from Redis queue"
                );
                
                return Ok(count > 0);
            }
        }
    }
    
    Ok(false) // Tâche non trouvée dans Redis
}

fn validate_task_dto(dto: &CreateTaskDto) -> Result<(), ApiError> {
    if !["video", "audio", "image"].contains(&dto.task_type.as_str()) {
        return Err(ApiError::InvalidInput(
            format!("Invalid task_type: {}. Must be 'video', 'audio', or 'image'", dto.task_type)
        ));
    }
    
    if dto.file_path.is_empty() {
        return Err(ApiError::InvalidInput("file_path cannot be empty".to_string()));
    }
    
    if dto.task_type == "image" {
        if let Some(quality) = dto.options.quality {
            if quality > 100 {
                return Err(ApiError::InvalidInput(
                    "Image quality must be between 0-100".to_string()
                ));
            }
        }
    }
    
    Ok(())
}

fn parse_task_type(task_type: &str) -> Result<TaskType, ApiError> {
    match task_type {
        "video" => Ok(TaskType::VideoCompression),
        "audio" => Ok(TaskType::AudioProcessing),
        "image" => Ok(TaskType::ImageOptimization),
        _ => Err(ApiError::InvalidInput(format!("Invalid task_type: {}", task_type))),
    }
}

fn task_type_to_media_type(task_type: &TaskType) -> MediaType {
    match task_type {
        TaskType::VideoCompression => MediaType::Video,
        TaskType::AudioProcessing => MediaType::Audio,
        TaskType::ImageOptimization => MediaType::Image,
    }
}

fn add_options_to_metadata(metadata: &mut HashMap<String, String>, options: &TaskOptionsDto) {
    if let Some(ref codec) = options.video_codec {
        metadata.insert("video_codec".to_string(), codec.clone());
    }
    if let Some(ref resolution) = options.resolution {
        metadata.insert("resolution".to_string(), resolution.clone());
    }
    if let Some(ref bitrate) = options.bitrate {
        metadata.insert("bitrate".to_string(), bitrate.clone());
    }
    if let Some(ref format) = options.audio_format {
        metadata.insert("audio_format".to_string(), format.clone());
    }
    if let Some(sample_rate) = options.sample_rate {
        metadata.insert("sample_rate".to_string(), sample_rate.to_string());
    }
    if let Some(ref format) = options.image_format {
        metadata.insert("image_format".to_string(), format.clone());
    }
    if let Some(quality) = options.quality {
        metadata.insert("quality".to_string(), quality.to_string());
    }
    if let Some(width) = options.max_width {
        metadata.insert("max_width".to_string(), width.to_string());
    }
    if let Some(height) = options.max_height {
        metadata.insert("max_height".to_string(), height.to_string());
    }
}

fn task_to_response(task: Task) -> TaskResponse {
    TaskResponse {
        id: task.id,
        task_type: task.task_type.to_string(),
        status: task.status.to_string(),
        progress: task.progress,
        error: task.error,
        output_path: task.output_path,
        created_at: task.created_at.to_rfc3339(),
        updated_at: task.updated_at.to_rfc3339(),
    }
}
