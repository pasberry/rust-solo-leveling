use crate::error::Result;
use bytes::Bytes;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Entry in the cache with optional TTL
#[derive(Clone, Debug)]
struct CacheEntry {
    value: Bytes,
    expires_at: Option<Instant>,
}

impl CacheEntry {
    fn is_expired(&self) -> bool {
        self.expires_at
            .map_or(false, |exp| Instant::now() >= exp)
    }
}

/// Configuration for a cache node
#[derive(Clone, Debug)]
pub struct CacheConfig {
    /// Maximum number of entries
    pub max_entries: usize,
    /// Default TTL for entries (None = no expiration)
    pub default_ttl: Option<Duration>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        CacheConfig {
            max_entries: 10000,
            default_ttl: None,
        }
    }
}

/// An individual cache node with LRU eviction
pub struct CacheNode {
    cache: Arc<RwLock<LruCache<String, CacheEntry>>>,
    config: CacheConfig,
}

impl CacheNode {
    /// Create a new cache node
    pub fn new(config: CacheConfig) -> Self {
        let capacity = NonZeroUsize::new(config.max_entries).unwrap_or(NonZeroUsize::new(1).unwrap());

        CacheNode {
            cache: Arc::new(RwLock::new(LruCache::new(capacity))),
            config,
        }
    }

    /// Create with default configuration
    pub fn with_capacity(max_entries: usize) -> Self {
        Self::new(CacheConfig {
            max_entries,
            ..Default::default()
        })
    }

    /// Get a value from the cache
    pub async fn get(&self, key: &str) -> Result<Option<Bytes>> {
        let mut cache = self.cache.write().await;

        match cache.get(key) {
            Some(entry) if !entry.is_expired() => Ok(Some(entry.value.clone())),
            Some(_) => {
                // Entry expired, remove it
                cache.pop(key);
                Ok(None)
            }
            None => Ok(None),
        }
    }

    /// Set a value in the cache
    pub async fn set(&self, key: String, value: Bytes) -> Result<()> {
        self.set_with_ttl(key, value, self.config.default_ttl).await
    }

    /// Set a value with specific TTL
    pub async fn set_with_ttl(
        &self,
        key: String,
        value: Bytes,
        ttl: Option<Duration>,
    ) -> Result<()> {
        let mut cache = self.cache.write().await;

        let entry = CacheEntry {
            value,
            expires_at: ttl.map(|d| Instant::now() + d),
        };

        cache.put(key, entry);
        Ok(())
    }

    /// Delete a value from the cache
    pub async fn delete(&self, key: &str) -> Result<bool> {
        let mut cache = self.cache.write().await;
        Ok(cache.pop(key).is_some())
    }

    /// Check if a key exists
    pub async fn exists(&self, key: &str) -> Result<bool> {
        let mut cache = self.cache.write().await;

        match cache.peek(key) {
            Some(entry) if !entry.is_expired() => Ok(true),
            Some(_) => {
                cache.pop(key);
                Ok(false)
            }
            None => Ok(false),
        }
    }

    /// Get current cache size
    pub async fn len(&self) -> usize {
        self.cache.read().await.len()
    }

    /// Check if cache is empty
    pub async fn is_empty(&self) -> bool {
        self.cache.read().await.is_empty()
    }

    /// Clear all entries
    pub async fn clear(&self) {
        self.cache.write().await.clear();
    }

    /// Clean up expired entries
    pub async fn cleanup_expired(&self) -> usize {
        let mut cache = self.cache.write().await;
        let mut expired_keys = Vec::new();

        // Find expired keys (can't remove during iteration)
        for (key, entry) in cache.iter() {
            if entry.is_expired() {
                expired_keys.push(key.clone());
            }
        }

        // Remove expired keys
        for key in &expired_keys {
            cache.pop(key);
        }

        expired_keys.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_set() {
        let cache = CacheNode::with_capacity(100);

        cache
            .set("key1".to_string(), Bytes::from("value1"))
            .await
            .unwrap();

        let value = cache.get("key1").await.unwrap();
        assert_eq!(value, Some(Bytes::from("value1")));
    }

    #[tokio::test]
    async fn test_get_nonexistent() {
        let cache = CacheNode::with_capacity(100);

        let value = cache.get("nonexistent").await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_delete() {
        let cache = CacheNode::with_capacity(100);

        cache
            .set("key1".to_string(), Bytes::from("value1"))
            .await
            .unwrap();

        let deleted = cache.delete("key1").await.unwrap();
        assert!(deleted);

        let value = cache.get("key1").await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_lru_eviction() {
        let cache = CacheNode::with_capacity(3);

        // Fill cache to capacity
        cache
            .set("key1".to_string(), Bytes::from("value1"))
            .await
            .unwrap();
        cache
            .set("key2".to_string(), Bytes::from("value2"))
            .await
            .unwrap();
        cache
            .set("key3".to_string(), Bytes::from("value3"))
            .await
            .unwrap();

        // Add one more - should evict key1
        cache
            .set("key4".to_string(), Bytes::from("value4"))
            .await
            .unwrap();

        assert_eq!(cache.get("key1").await.unwrap(), None);
        assert_eq!(cache.get("key2").await.unwrap(), Some(Bytes::from("value2")));
        assert_eq!(cache.get("key3").await.unwrap(), Some(Bytes::from("value3")));
        assert_eq!(cache.get("key4").await.unwrap(), Some(Bytes::from("value4")));
    }

    #[tokio::test]
    async fn test_ttl_expiration() {
        let cache = CacheNode::with_capacity(100);

        cache
            .set_with_ttl(
                "key1".to_string(),
                Bytes::from("value1"),
                Some(Duration::from_millis(100)),
            )
            .await
            .unwrap();

        // Value should exist initially
        assert_eq!(cache.get("key1").await.unwrap(), Some(Bytes::from("value1")));

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Value should be expired
        assert_eq!(cache.get("key1").await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_exists() {
        let cache = CacheNode::with_capacity(100);

        cache
            .set("key1".to_string(), Bytes::from("value1"))
            .await
            .unwrap();

        assert!(cache.exists("key1").await.unwrap());
        assert!(!cache.exists("key2").await.unwrap());
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let cache = CacheNode::with_capacity(100);

        // Add some entries with short TTL
        for i in 0..5 {
            cache
                .set_with_ttl(
                    format!("key{}", i),
                    Bytes::from(format!("value{}", i)),
                    Some(Duration::from_millis(50)),
                )
                .await
                .unwrap();
        }

        // Add some without TTL
        for i in 5..10 {
            cache
                .set(format!("key{}", i), Bytes::from(format!("value{}", i)))
                .await
                .unwrap();
        }

        assert_eq!(cache.len().await, 10);

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Cleanup
        let expired = cache.cleanup_expired().await;
        assert_eq!(expired, 5);
        assert_eq!(cache.len().await, 5);
    }
}
