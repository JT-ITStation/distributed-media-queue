use mongodb::Client as MongoClient;
use redis::Client as RedisClient;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Métriques de l'application
pub struct Metrics {
    pub tasks_created: AtomicU64,
    pub tasks_completed: AtomicU64,
    pub tasks_failed: AtomicU64,
    pub tasks_cancelled: AtomicU64,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            tasks_created: AtomicU64::new(0),
            tasks_completed: AtomicU64::new(0),
            tasks_failed: AtomicU64::new(0),
            tasks_cancelled: AtomicU64::new(0),
        }
    }
    
    // Increment methods
    pub fn increment_created(&self) {
        self.tasks_created.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn increment_completed(&self) {
        self.tasks_completed.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn increment_failed(&self) {
        self.tasks_failed.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn increment_cancelled(&self) {
        self.tasks_cancelled.fetch_add(1, Ordering::Relaxed);
    }
    
    // Get methods
    pub fn get_created(&self) -> u64 {
        self.tasks_created.load(Ordering::Relaxed)
    }
    
    pub fn get_completed(&self) -> u64 {
        self.tasks_completed.load(Ordering::Relaxed)
    }
    
    pub fn get_failed(&self) -> u64 {
        self.tasks_failed.load(Ordering::Relaxed)
    }
    
    pub fn get_cancelled(&self) -> u64 {
        self.tasks_cancelled.load(Ordering::Relaxed)
    }
    
    // Set methods (pour sync avec MongoDB)
    pub fn set_created(&self, value: u64) {
        self.tasks_created.store(value, Ordering::Relaxed);
    }
    
    pub fn set_completed(&self, value: u64) {
        self.tasks_completed.store(value, Ordering::Relaxed);
    }
    
    pub fn set_failed(&self, value: u64) {
        self.tasks_failed.store(value, Ordering::Relaxed);
    }
    
    pub fn set_cancelled(&self, value: u64) {
        self.tasks_cancelled.store(value, Ordering::Relaxed);
    }
    
    // Reset all counters to zero
    pub fn reset_all(&self) {
        self.tasks_created.store(0, Ordering::Relaxed);
        self.tasks_completed.store(0, Ordering::Relaxed);
        self.tasks_failed.store(0, Ordering::Relaxed);
        self.tasks_cancelled.store(0, Ordering::Relaxed);
    }
}

/// État partagé de l'application
#[derive(Clone)]
pub struct AppState {
    pub mongo_client: MongoClient,
    pub redis_client: RedisClient,
    pub database_name: String,
    pub metrics: Arc<Metrics>,
}

impl AppState {
    pub async fn new(
        mongo_uri: &str,
        redis_uri: &str,
        database_name: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Connexion MongoDB
        let mongo_client = MongoClient::with_uri_str(mongo_uri).await?;
        
        // Test MongoDB
        mongo_client
            .database("admin")
            .run_command(mongodb::bson::doc! { "ping": 1 }, None)
            .await?;
        
        tracing::info!("Connected to MongoDB");
        
        // Connexion Redis
        let redis_client = RedisClient::open(redis_uri)?;
        
        // Test Redis
        let mut conn = redis_client.get_async_connection().await?;
        let _: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await?;
        
        tracing::info!("Connected to Redis");
        
        Ok(Self {
            mongo_client,
            redis_client,
            database_name,
            metrics: Arc::new(Metrics::new()),
        })
    }

    pub fn get_database(&self) -> mongodb::Database {
        self.mongo_client.database(&self.database_name)
    }
}
