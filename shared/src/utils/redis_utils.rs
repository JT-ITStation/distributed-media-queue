use redis::{Client, RedisError, AsyncCommands};
use serde::Serialize;
use crate::models::Task;

pub struct RedisClient {
    client: Client,
}

impl RedisClient {
    pub async fn new(redis_url: &str) -> Result<Self, RedisError> {
        let client = Client::open(redis_url)?;
        
        // Test connection
        let mut conn = client.get_async_connection().await?;
        let _: String = redis::cmd("PING").query_async(&mut conn).await?;
        
        tracing::info!("Successfully connected to Redis");
        
        Ok(Self { client })
    }

    pub async fn enqueue_task(&self, task: &Task) -> Result<(), RedisError> {
        let mut conn = self.client.get_async_connection().await?;
        let queue_name = format!("queue:{}", task.task_type);
        let serialized = serde_json::to_string(task)
            .map_err(|e| RedisError::from((
                redis::ErrorKind::TypeError,
                "Serialization error",
                e.to_string()
            )))?;
        
        conn.lpush(&queue_name, serialized).await?;
        
        tracing::debug!(
            task_id = %task.id,
            task_type = ?task.task_type,
            "Task enqueued to {}",
            queue_name
        );
        
        Ok(())
    }

    pub async fn get_queue_length(&self, queue_name: &str) -> Result<usize, RedisError> {
        let mut conn = self.client.get_async_connection().await?;
        let len: usize = conn.llen(queue_name).await?;
        Ok(len)
    }

    pub fn client(&self) -> &Client {
        &self.client
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{MediaFile, MediaType, TaskType};
    use std::path::PathBuf;
    use std::collections::HashMap;

    // Note: Ces tests nécessitent une instance Redis en cours d'exécution
    // Vous pouvez les ignorer avec: cargo test -- --skip redis

    #[tokio::test]
    #[ignore] // Ignorer par défaut, activer quand Redis est disponible
    async fn test_redis_connection() {
        let client = RedisClient::new("redis://127.0.0.1:6379").await;
        assert!(client.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn test_enqueue_task() {
        let client = RedisClient::new("redis://127.0.0.1:6379")
            .await
            .expect("Failed to connect to Redis");

        let media = MediaFile::new(
            "test-123".to_string(),
            MediaType::Video,
            PathBuf::from("/path/to/video.mp4"),
            1024000,
            "video.mp4".to_string(),
            "video/mp4".to_string(),
        );

        let task = Task::new(TaskType::VideoCompression, media);
        let result = client.enqueue_task(&task).await;
        
        assert!(result.is_ok());

        // Vérifier la longueur de la queue
        let queue_len = client.get_queue_length("queue:video").await.unwrap();
        assert!(queue_len > 0);
    }
}
