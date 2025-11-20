mod dtos;
mod handlers;
mod routes;
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
                .unwrap_or_else(|_| "monitor=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    let host = std::env::var("MONITOR_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("MONITOR_PORT").unwrap_or_else(|_| "3001".to_string());
    let mongo_uri = std::env::var("MONGODB_URI")
        .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    let redis_uri = std::env::var("REDIS_URI")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let database_name = std::env::var("MONGODB_DATABASE")
        .unwrap_or_else(|_| "distributed_media_queue".to_string());
    
    tracing::info!("Starting Monitor Dashboard...");
    tracing::info!("MongoDB URI: {}", mongo_uri);
    tracing::info!("Redis URI: {}", redis_uri);
    
    let state = Arc::new(
        AppState::new(&mongo_uri, &redis_uri, database_name)
            .await
            .expect("Failed to initialize application state"),
    );
    
    let app = routes::create_router(state);
    
    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");
    
    tracing::info!("🖥️  Monitor Dashboard listening on http://{}", addr);
    tracing::info!("📊 Dashboard: http://{}/", addr);
    tracing::info!("📡 API Stats: http://{}/api/stats", addr);
    tracing::info!("📋 Recent Tasks: http://{}/api/tasks/recent", addr);
    
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
    
    Ok(())
}
