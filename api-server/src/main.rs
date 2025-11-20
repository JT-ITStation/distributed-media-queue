mod dtos;
mod error;
mod handlers;
mod routes;
mod services;
mod state;

use state::AppState;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "api_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    let host = std::env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("API_PORT").unwrap_or_else(|_| "3000".to_string());
    let mongo_uri = std::env::var("MONGODB_URI")
        .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    let redis_uri = std::env::var("REDIS_URI")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let database_name = std::env::var("MONGODB_DATABASE")
        .unwrap_or_else(|_| "distributed_media_queue".to_string());
    
    tracing::info!("Starting API Server...");
    tracing::info!("MongoDB URI: {}", mongo_uri);
    tracing::info!("Redis URI: {}", redis_uri);
    tracing::info!("Database: {}", database_name);
    
    let state = Arc::new(
        AppState::new(&mongo_uri, &redis_uri, database_name)
            .await
            .expect("Failed to initialize application state"),
    );
    
    tracing::info!("Application state initialized");
    
    let app = routes::create_router(state);
    
    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");
    
    tracing::info!("API Server listening on {}", addr);
    tracing::info!("📡 Health check: http://{}/health", addr);
    tracing::info!("📋 API endpoints:");
    tracing::info!("  POST   /tasks       - Create task");
    tracing::info!("  GET    /tasks       - List tasks");
    tracing::info!("  GET    /tasks/:id   - Get task");
    tracing::info!("  DELETE /tasks/:id   - Cancel task");
    
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
    
    Ok(())
}
