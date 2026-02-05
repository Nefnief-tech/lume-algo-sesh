use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;

/// Errors that can occur with cache operations
#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Cache miss: {0}")]
    CacheMiss(String),
}

/// Multi-tier cache manager
///
/// Implements L1 (in-memory) and L2 (Redis) caching strategy.
/// L1 is fastest but limited in size, L2 is shared across instances.
pub struct CacheManager {
    // Store ConnectionManager in a Mutex for interior mutability
    redis: Arc<tokio::sync::Mutex<ConnectionManager>>,
    l1_cache: moka::future::Cache<String, Vec<u8>>,
    ttl_secs: u64,
}

impl CacheManager {
    /// Create a new cache manager
    pub async fn new(redis_url: &str, l1_size: u64, ttl_secs: u64) -> Result<Self, CacheError> {
        let client = redis::Client::open(redis_url)?;
        let redis = redis::aio::ConnectionManager::new(client).await?;

        let l1_cache = moka::future::CacheBuilder::new(l1_size)
            .time_to_live(Duration::from_secs(ttl_secs))
            .build();

        Ok(Self {
            redis: Arc::new(tokio::sync::Mutex::new(redis)),
            l1_cache,
            ttl_secs,
        })
    }

    /// Get a value from cache (L1 first, then L2)
    pub async fn get<T>(&self, key: &str) -> Result<T, CacheError>
    where
        T: for<'de> Deserialize<'de>,
    {
        // Try L1 cache first
        if let Some(bytes) = self.l1_cache.get(key).await {
            tracing::trace!("L1 cache hit: {}", key);
            return Ok(serde_json::from_slice(&bytes)?);
        }

        // Try L2 cache (Redis)
        let mut conn = self.redis.lock().await;
        let value: Option<String> = redis::cmd("GET")
            .arg(key)
            .query_async(&mut *conn)
            .await?;
        drop(conn);

        if let Some(json) = value {
            tracing::trace!("L2 cache hit: {}", key);

            // Populate L1 cache
            let bytes = json.as_bytes().to_vec();
            self.l1_cache.insert(key.to_string(), bytes).await;

            return Ok(serde_json::from_str(&json)?);
        }

        tracing::trace!("Cache miss: {}", key);
        Err(CacheError::CacheMiss(key.to_string()))
    }

    /// Set a value in cache (both L1 and L2)
    pub async fn set<T>(&self, key: &str, value: &T) -> Result<(), CacheError>
    where
        T: Serialize,
    {
        let json = serde_json::to_string(value)?;

        // Set in L1 cache (uses configured TTL)
        let bytes = json.as_bytes().to_vec();
        self.l1_cache.insert(key.to_string(), bytes).await;

        // Set in L2 cache with explicit TTL
        let mut conn = self.redis.lock().await;
        redis::cmd("SETEX")
            .arg(key)
            .arg(self.ttl_secs)
            .arg(json)
            .query_async::<()>(&mut *conn)
            .await?;
        drop(conn);

        tracing::trace!("Cache set: {}", key);
        Ok(())
    }

    /// Delete a value from both cache tiers
    pub async fn delete(&self, key: &str) -> Result<(), CacheError> {
        self.l1_cache.invalidate(key).await;
        let mut conn = self.redis.lock().await;
        redis::cmd("DEL")
            .arg(key)
            .query_async::<()>(&mut *conn)
            .await?;
        Ok(())
    }

    /// Invalidate all cache entries matching a pattern
    pub async fn invalidate_pattern(&self, pattern: &str) -> Result<(), CacheError> {
        // For L1, we need to iterate (clear all for simplicity)
        self.l1_cache.invalidate_all();

        // For Redis, use KEYS to find matching keys
        let mut conn = self.redis.lock().await;
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(pattern)
            .query_async(&mut *conn)
            .await?;

        if !keys.is_empty() {
            redis::cmd("DEL")
                .arg(keys)
                .query_async::<()>(&mut *conn)
                .await?;
        }

        tracing::debug!("Invalidated cache pattern: {}", pattern);
        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            l1_size: self.l1_cache.entry_count(),
            l1_hit_count: 0,
            l1_miss_count: 0,
            l1_hit_rate: 0.0,
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub l1_size: u64,
    pub l1_hit_count: u64,
    pub l1_miss_count: u64,
    pub l1_hit_rate: f64,
}

/// Cache key builder
pub struct CacheKey;

impl CacheKey {
    /// Build a cache key for user preferences
    pub fn preferences(user_id: &str) -> String {
        format!("prefs:{}", user_id)
    }

    /// Build a cache key for candidate matches
    pub fn candidates(user_id: &str, page: u32) -> String {
        format!("candidates:{}:{}", user_id, page)
    }

    /// Build a cache key for user profile
    pub fn profile(user_id: &str) -> String {
        format!("profile:{}", user_id)
    }

    /// Build a cache key for match results
    pub fn matches(user_id: &str) -> String {
        format!("matches:{}", user_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "Requires Redis"]
    async fn test_cache_set_get() {
        let cache = CacheManager::new("redis://127.0.0.1:6379", 1000, 60)
            .await
            .expect("Failed to create cache");

        let key = "test_key";
        let value = "test_value";

        // Set and get
        cache.set(key, &value).await.unwrap();
        let result: String = cache.get(key).await.unwrap();
        assert_eq!(result, value);

        // Delete
        cache.delete(key).await.unwrap();
        assert!(cache.get::<String>(key).await.is_err());
    }

    #[test]
    fn test_cache_key_builder() {
        assert_eq!(CacheKey::preferences("user123"), "prefs:user123");
        assert_eq!(CacheKey::candidates("user123", 1), "candidates:user123:1");
        assert_eq!(CacheKey::profile("user123"), "profile:user123");
        assert_eq!(CacheKey::matches("user123"), "matches:user123");
    }
}
