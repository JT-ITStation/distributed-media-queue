use mongodb::{Client, Database, bson::doc};
use crate::models::Task;

pub struct MongoClient {
    client: Client,
    database_name: String,
}

impl MongoClient {
    pub async fn new(
        connection_string: &str,
        database_name: &str,
    ) -> Result<Self, mongodb::error::Error> {
        let client = Client::with_uri_str(connection_string).await?;
        
        // Test connection
        client
            .database("admin")
            .run_command(doc! { "ping": 1 }, None)
            .await?;
        
        tracing::info!("Successfully connected to MongoDB");
        
        Ok(Self {
            client,
            database_name: database_name.to_string(),
        })
    }

    pub fn get_database(&self) -> Database {
        self.client.database(&self.database_name)
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub async fn insert_task(&self, task: &Task) -> Result<(), mongodb::error::Error> {
        let db = self.get_database();
        let collection = db.collection::<Task>("tasks");
        
        collection.insert_one(task, None).await?;
        
        tracing::debug!(task_id = %task.id, "Task inserted into MongoDB");
        
        Ok(())
    }

    pub async fn get_task(&self, task_id: &str) -> Result<Option<Task>, mongodb::error::Error> {
        let db = self.get_database();
        let collection = db.collection::<Task>("tasks");
        
        let filter = doc! { "task_id": task_id };
        collection.find_one(filter, None).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{MediaFile, MediaType, TaskType};
    use std::path::PathBuf;

    // Note: Ces tests nécessitent une instance MongoDB en cours d'exécution
    // Vous pouvez les ignorer avec: cargo test -- --skip mongo

    #[tokio::test]
    #[ignore] // Ignorer par défaut
    async fn test_mongo_connection() {
        let client = MongoClient::new(
            "mongodb://localhost:27017",
            "test_db"
        ).await;
        
        assert!(client.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn test_insert_and_get_task() {
        let client = MongoClient::new(
            "mongodb://localhost:27017",
            "test_db"
        )
        .await
        .expect("Failed to connect to MongoDB");

        let media = MediaFile::new(
            "test-123".to_string(),
            MediaType::Video,
            PathBuf::from("/path/to/video.mp4"),
            1024000,
            "video.mp4".to_string(),
            "video/mp4".to_string(),
        );

        let task = Task::new(TaskType::VideoCompression, media);
        let task_id = task.id.clone();

        // Insert
        let result = client.insert_task(&task).await;
        assert!(result.is_ok());

        // Get
        let retrieved = client.get_task(&task_id).await.unwrap();
        assert!(retrieved.is_some());
        
        let retrieved_task = retrieved.unwrap();
        assert_eq!(retrieved_task.id, task_id);
    }
}
