mod engine;
mod processors;

use engine::WorkerEngine;
use processors::{AudioProcessor, ImageProcessor, TaskProcessor, VideoProcessor};
use shared::TaskType;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment
    dotenv::dotenv().ok();
    
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "worker=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    // Parse CLI arguments
    let args: Vec<String> = std::env::args().collect();
    let worker_type_arg = if args.len() > 2 && args[1] == "--task-type" {
        Some(args[2].clone())
    } else {
        None
    };
    
    // Configuration - priorité aux args CLI, puis env, puis défaut
    let worker_type = worker_type_arg
        .or_else(|| std::env::var("WORKER_TYPE").ok())
        .unwrap_or_else(|| "video".to_string());
    
    let worker_id = std::env::var("WORKER_ID")
        .unwrap_or_else(|_| format!("worker-{}", uuid::Uuid::new_v4()));
    let mongo_uri = std::env::var("MONGODB_URI")
        .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    let redis_uri = std::env::var("REDIS_URI")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let database_name = std::env::var("MONGODB_DATABASE")
        .unwrap_or_else(|_| "distributed_media_queue".to_string());
    let output_dir = std::env::var("OUTPUT_DIR")
        .unwrap_or_else(|_| "/tmp/processed".to_string());
    
    tracing::info!("Starting Worker...");
    tracing::info!("Worker ID: {}", worker_id);
    tracing::info!("Worker Type: {}", worker_type);
    tracing::info!("MongoDB URI: {}", mongo_uri);
    tracing::info!("Redis URI: {}", redis_uri);
    tracing::info!("Output Dir: {}", output_dir);
    
    // Connect to MongoDB
    let mongo_client = mongodb::Client::with_uri_str(&mongo_uri)
        .await
        .expect("Failed to connect to MongoDB");
    
    mongo_client
        .database("admin")
        .run_command(mongodb::bson::doc! { "ping": 1 }, None)
        .await
        .expect("Failed to ping MongoDB");
    
    tracing::info!("Connected to MongoDB");
    
    let mongo_db = mongo_client.database(&database_name);
    
    // Connect to Redis
    let redis_client = redis::Client::open(redis_uri.as_str())
        .expect("Failed to create Redis client");
    
    let mut conn = redis_client
        .get_async_connection()
        .await
        .expect("Failed to connect to Redis");
    
    let _: String = redis::cmd("PING")
        .query_async(&mut conn)
        .await
        .expect("Failed to ping Redis");
    
    tracing::info!("Connected to Redis");
    
    // Create processor based on worker type
    let (processor, task_type): (Arc<dyn TaskProcessor>, TaskType) = match worker_type.as_str() {
        "video" => (
            Arc::new(VideoProcessor::new(output_dir)),
            TaskType::VideoCompression,
        ),
        "audio" => (
            Arc::new(AudioProcessor::new(output_dir)),
            TaskType::AudioProcessing,
        ),
        "image" => (
            Arc::new(ImageProcessor::new(output_dir)),
            TaskType::ImageOptimization,
        ),
        _ => panic!("Invalid WORKER_TYPE: {}. Must be 'video', 'audio', or 'image'", worker_type),
    };
    
    tracing::info!("Processor initialized: {}", worker_type);
    
    // Create and run worker engine
    let engine = WorkerEngine::new(
        redis_client,
        mongo_db,
        processor,
        task_type,
        worker_id,
    );
    
    tracing::info!("Worker engine starting...");
    
    engine.run().await?;
    
    Ok(())
}
