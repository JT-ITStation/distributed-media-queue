use anyhow::{Context, Result};
use redis::Client as RedisClient;
use serde::{Deserialize, Serialize};

/// Client pour gérer les publications/souscriptions Redis
#[derive(Clone)]
pub struct PubSubClient {
    redis_client: RedisClient,
}

impl PubSubClient {
    /// Crée un nouveau client pub/sub
    pub fn new(redis_client: RedisClient) -> Self {
        Self { redis_client }
    }
    
    /// Publie un message sur un canal
    pub async fn publish(&self, channel: &str, message: &str) -> Result<()> {
        let mut conn = self.redis_client
            .get_async_connection()
            .await
            .context("Failed to get Redis connection for publish")?;
        
        redis::cmd("PUBLISH")
            .arg(channel)
            .arg(message)
            .query_async(&mut conn)
            .await
            .context("Failed to publish message to Redis")?;
        
        tracing::debug!(channel = %channel, "Published message to Redis");
        Ok(())
    }
    
    /// Publie une commande sérialisée en JSON
    pub async fn publish_command(&self, channel: &str, command: &TaskCommand) -> Result<()> {
        let message = serde_json::to_string(command)
            .context("Failed to serialize command")?;
        self.publish(channel, &message).await
    }
    
    /// Publie une commande d'annulation pour une tâche spécifique
    pub async fn cancel_task(&self, task_id: &str) -> Result<()> {
        let channel = format!("task:cancel:{}", task_id);
        self.publish(&channel, "cancel").await
    }
    
    /// Souscrit à un pattern de canaux (psubscribe)
    pub async fn psubscribe(&self, patterns: Vec<String>) -> Result<redis::aio::PubSub> {
        let pubsub = self.redis_client
            .get_async_connection()
            .await
            .context("Failed to get Redis connection for psubscribe")?
            .into_pubsub();
        
        for pattern in &patterns {
            tracing::debug!(pattern = %pattern, "Pattern subscribed to Redis");
        }
        
        Ok(pubsub)
    }
    
    /// Souscrit à un ou plusieurs canaux exacts
    pub async fn subscribe(&self, channels: Vec<String>) -> Result<redis::aio::PubSub> {
        let pubsub = self.redis_client
            .get_async_connection()
            .await
            .context("Failed to get Redis connection for subscribe")?
            .into_pubsub();
        
        for channel in &channels {
            tracing::debug!(channel = %channel, "Subscribed to Redis channel");
        }
        
        Ok(pubsub)
    }
}

/// Commandes possibles pour les tâches
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TaskCommand {
    /// Annuler une tâche
    Cancel { task_id: String },
    
    /// Mettre en pause une tâche (future implémentation)
    #[allow(dead_code)]
    Pause { task_id: String },
    
    /// Reprendre une tâche (future implémentation)
    #[allow(dead_code)]
    Resume { task_id: String },
}

impl TaskCommand {
    /// Obtient l'ID de la tâche associée à la commande
    pub fn task_id(&self) -> &str {
        match self {
            TaskCommand::Cancel { task_id } => task_id,
            TaskCommand::Pause { task_id } => task_id,
            TaskCommand::Resume { task_id } => task_id,
        }
    }
}
