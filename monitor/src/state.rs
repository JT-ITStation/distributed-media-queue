use mongodb::Client as MongoClient;
use redis::Client as RedisClient;

#[derive(Clone)]
pub struct AppState {
    pub mongo_client: MongoClient,
    pub redis_client: RedisClient,
    pub database_name: String,
}

impl AppState {
    pub async fn new(
        mongo_uri: &str,
        redis_uri: &str,
        database_name: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mongo_client = MongoClient::with_uri_str(mongo_uri).await?;
        
        mongo_client
            .database("admin")
            .run_command(mongodb::bson::doc! { "ping": 1 }, None)
            .await?;
        
        tracing::info!("Connected to MongoDB");
        
        let redis_client = RedisClient::open(redis_uri)?;
        
        let mut conn = redis_client.get_async_connection().await?;
        let _: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await?;
        
        tracing::info!("Connected to Redis");
        
        Ok(Self {
            mongo_client,
            redis_client,
            database_name,
        })
    }

    pub fn get_database(&self) -> mongodb::Database {
        self.mongo_client.database(&self.database_name)
    }
}
