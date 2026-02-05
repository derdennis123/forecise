use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use serde::{Serialize, de::DeserializeOwned};
use tracing::warn;

/// Try to get a cached JSON value from Redis.
pub async fn get<T: DeserializeOwned>(redis: &ConnectionManager, key: &str) -> Option<T> {
    let mut conn = redis.clone();
    match conn.get::<_, Option<String>>(key).await {
        Ok(Some(json)) => serde_json::from_str(&json).ok(),
        _ => None,
    }
}

/// Cache a serializable value in Redis with a TTL in seconds.
pub async fn set<T: Serialize>(redis: &ConnectionManager, key: &str, value: &T, ttl_secs: u64) {
    let mut conn = redis.clone();
    if let Ok(json) = serde_json::to_string(value) {
        if let Err(e) = conn.set_ex::<_, _, ()>(key, json, ttl_secs).await {
            warn!("Redis cache set failed for key {}: {}", key, e);
        }
    }
}
