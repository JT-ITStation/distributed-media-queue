use crate::processors::{TaskProcessor, ProgressCallback, CancelFlag};
use anyhow::{Context, Result};
use mongodb::Database;
use redis::Client as RedisClient;
use shared::{Task, TaskStatus, TaskType, PubSubClient};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use futures_util::stream::StreamExt;

pub struct WorkerEngine {
    redis_client: RedisClient,
    mongo_db: Database,
    processor: Arc<dyn TaskProcessor>,
    queue_name: String,
    worker_id: String,
    pubsub_client: PubSubClient,
    cancel_tx: mpsc::Sender<String>,
    cancel_rx: mpsc::Receiver<String>,
}

impl WorkerEngine {
    pub fn new(
        redis_client: RedisClient,
        mongo_db: Database,
        processor: Arc<dyn TaskProcessor>,
        task_type: TaskType,
        worker_id: String,
    ) -> Self {
        let queue_name = format!("queue:{}", task_type);
        let pubsub_client = PubSubClient::new(redis_client.clone());
        let (cancel_tx, cancel_rx) = mpsc::channel(100);
        
        Self {
            redis_client,
            mongo_db,
            processor,
            queue_name,
            worker_id,
            pubsub_client,
            cancel_tx,
            cancel_rx,
        }
    }
    
    /// Spawne un listener global qui écoute tous les messages de cancellation
    async fn spawn_cancel_listener(&self) {
        let redis_client = self.redis_client.clone();
        let cancel_tx = self.cancel_tx.clone();
        let worker_id = self.worker_id.clone();
        
        tokio::spawn(async move {
            tracing::info!(worker_id = %worker_id, "Starting global cancel listener");
            
            // Créer une connexion pub/sub dédiée
            let conn = match redis_client.get_async_connection().await {
                Ok(conn) => conn,
                Err(e) => {
                    tracing::error!("Failed to get Redis connection for pubsub: {}", e);
                    return;
                }
            };
            
            let mut pubsub = conn.into_pubsub();
            
            // Subscribe au pattern
            if let Err(e) = pubsub.psubscribe("task:cancel:*").await {
                tracing::error!("Failed to psubscribe: {}", e);
                return;
            }
            
            tracing::info!(worker_id = %worker_id, "Successfully subscribed to task:cancel:*");
            
            // Écouter les messages
            let mut pubsub_stream = pubsub.on_message();
            
            loop {
                match pubsub_stream.next().await {
                    Some(msg) => {
                        let channel: String = msg.get_channel_name().to_string();
                        if let Some(task_id) = channel.strip_prefix("task:cancel:") {
                            tracing::info!(
                                worker_id = %worker_id,
                                task_id = %task_id,
                                "Received cancellation command"
                            );
                            
                            if let Err(e) = cancel_tx.send(task_id.to_string()).await {
                                tracing::error!("Failed to send cancel message: {}", e);
                            }
                        }
                    }
                    None => {
                        tracing::warn!("Pubsub stream ended");
                        break;
                    }
                }
            }
        });
    }
    
    pub async fn run(mut self) -> Result<()> {
        tracing::info!(
            worker_id = %self.worker_id,
            queue = %self.queue_name,
            "Worker started"
        );
        
        // Spawner le listener global AVANT la boucle
        self.spawn_cancel_listener().await;
        
        // Attendre un peu que le listener soit prêt
        sleep(Duration::from_millis(500)).await;
        
        loop {
            match self.process_next_task().await {
                Ok(Some(())) => {
                    tracing::debug!(worker_id = %self.worker_id, "Task processed successfully");
                }
                Ok(None) => {
                    // Aucune tâche disponible, attendre
                    tracing::trace!(worker_id = %self.worker_id, "No tasks available, waiting...");
                    sleep(Duration::from_secs(2)).await;
                }
                Err(e) => {
                    tracing::error!(
                        worker_id = %self.worker_id,
                        error = %e,
                        "Error processing task"
                    );
                    sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }
    
    async fn process_next_task(&mut self) -> Result<Option<()>> {
        // 1. Dequeue depuis Redis (RPOP = prendre depuis la fin)
        let mut conn = self.redis_client.get_async_connection().await?;
        
        let task_json: Option<String> = redis::cmd("RPOP")
            .arg(&self.queue_name)
            .query_async(&mut conn)
            .await
            .context("Failed to dequeue task from Redis")?;
        
        let task_json = match task_json {
            Some(json) => json,
            None => return Ok(None), // Pas de tâche
        };
        
        // 2. Désérialiser la tâche
        let mut task: Task = serde_json::from_str(&task_json)
            .context("Failed to deserialize task")?;
        
        tracing::info!(
            worker_id = %self.worker_id,
            task_id = %task.id,
            "Dequeued task"
        );
        
        // 3. Vérifier le statut dans MongoDB AVANT de traiter
        let db_task = self.get_task_from_db(&task.id).await?;
        
        match db_task {
            Some(db_task) if db_task.status == TaskStatus::Cancelled => {
                tracing::warn!(
                    worker_id = %self.worker_id,
                    task_id = %task.id,
                    "Task was cancelled, skipping"
                );
                return Ok(Some(()));
            }
            None => {
                tracing::error!(
                    worker_id = %self.worker_id,
                    task_id = %task.id,
                    "Task not found in database, skipping"
                );
                return Ok(Some(()));
            }
            _ => {
                // Task is valid, proceed
            }
        }
        
        tracing::info!(
            worker_id = %self.worker_id,
            task_id = %task.id,
            "Processing task"
        );
        
        // 4. Créer le callback de progression
        let task_id = task.id.clone();
        let db = self.mongo_db.clone();
        let progress_callback: ProgressCallback = Arc::new(move |progress| {
            let task_id = task_id.clone();
            let db = db.clone();
            
            // Spawner une tâche async pour update MongoDB
            tokio::spawn(async move {
                if let Err(e) = update_task_progress(&db, &task_id, progress).await {
                    tracing::error!(
                        task_id = %task_id,
                        progress = progress,
                        error = %e,
                        "Failed to update progress in MongoDB"
                    );
                }
            });
        });
        
        // 5. Créer le cancel_flag
        let cancel_flag = Arc::new(AtomicBool::new(false));
        let cancel_flag_clone = cancel_flag.clone();
        
        // 6. Traiter avec tokio::select!
        let task_id_for_select = task.id.clone();
        let process_future = self.processor.process(&mut task, progress_callback, cancel_flag_clone);
        
        tokio::select! {
            result = process_future => {
                // Traitement terminé normalement
                match result {
                    Ok(()) => {
                        self.update_task_in_db(&task).await?;
                        tracing::info!(
                            worker_id = %self.worker_id,
                            task_id = %task.id,
                            "Task completed successfully"
                        );
                    }
                    Err(e) => {
                        tracing::error!(
                            worker_id = %self.worker_id,
                            task_id = %task.id,
                            error = %e,
                            "Task processing failed"
                        );
                        
                        task.increment_retry();
                        task.error = Some(e.to_string());
                        
                        if task.should_retry() {
                            task.update_status(TaskStatus::Pending);
                            self.requeue_task(&task).await?;
                            tracing::warn!(
                                worker_id = %self.worker_id,
                                task_id = %task.id,
                                retry_count = task.retry_count,
                                "Task requeued for retry"
                            );
                        } else {
                            task.update_status(TaskStatus::Failed);
                            self.update_task_in_db(&task).await?;
                            tracing::error!(
                                worker_id = %self.worker_id,
                                task_id = %task.id,
                                "Task failed permanently after max retries"
                            );
                        }
                    }
                }
            }
            Some(cancelled_task_id) = self.cancel_rx.recv() => {
                if cancelled_task_id == task_id_for_select {
                    // Marquer comme cancelled
                    cancel_flag.store(true, Ordering::SeqCst);
                    tracing::warn!(
                        worker_id = %self.worker_id,
                        task_id = %task.id,
                        "Cancellation signal received, flagging task"
                    );
                    
                    // Attendre un peu que le processor détecte la cancellation
                    sleep(Duration::from_millis(500)).await;
                    
                    // Mettre à jour MongoDB
                    task.update_status(TaskStatus::Cancelled);
                    self.update_task_in_db(&task).await?;
                }
            }
        }
        
        Ok(Some(()))
    }
    
    async fn get_task_from_db(&self, task_id: &str) -> Result<Option<Task>> {
        let collection = self.mongo_db.collection::<Task>("tasks");
        let filter = mongodb::bson::doc! { "task_id": task_id };
        
        let task = collection
            .find_one(filter, None)
            .await
            .context("Failed to query task from MongoDB")?;
        
        Ok(task)
    }
    
    async fn update_task_in_db(&self, task: &Task) -> Result<()> {
        let collection = self.mongo_db.collection::<Task>("tasks");
        
        let filter = mongodb::bson::doc! { "task_id": &task.id };
        let update = mongodb::bson::doc! {
            "$set": mongodb::bson::to_document(task)?
        };
        
        collection
            .update_one(filter, update, None)
            .await
            .context("Failed to update task in MongoDB")?;
        
        Ok(())
    }
    
    async fn requeue_task(&self, task: &Task) -> Result<()> {
        let mut conn = self.redis_client.get_async_connection().await?;
        let serialized = serde_json::to_string(task)?;
        
        let _: () = redis::cmd("LPUSH")
            .arg(&self.queue_name)
            .arg(serialized)
            .query_async(&mut conn)
            .await?;
        
        Ok(())
    }
}

/// Fonction helper pour mettre à jour la progression dans MongoDB
async fn update_task_progress(db: &Database, task_id: &str, progress: f32) -> Result<()> {
    let collection = db.collection::<Task>("tasks");
    let filter = mongodb::bson::doc! { "task_id": task_id };
    let update = mongodb::bson::doc! {
        "$set": {
            "progress": progress,
            "updated_at": mongodb::bson::DateTime::from_millis(chrono::Utc::now().timestamp_millis())
        }
    };
    
    collection
        .update_one(filter, update, None)
        .await
        .context("Failed to update task progress")?;
    
    tracing::debug!(task_id = %task_id, progress = progress, "Progress updated in MongoDB");
    
    Ok(())
}
