use redis::aio::ConnectionManager;
use redis::Client;

use crate::config::RedisConfig;

pub type RedisPool = ConnectionManager;

pub async fn init_pool(config: &RedisConfig) -> Result<RedisPool, redis::RedisError> {
    let client = Client::open(config.url.as_str())?;
    let manager = ConnectionManager::new(client).await?;

    tracing::info!("Redis connection pool established");
    Ok(manager)
}
