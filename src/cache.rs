//! Tensor caching system.

use candle_core::Tensor;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::debug;

/// A cached tensor entry with expiration.
struct CacheEntry {
    tensor: Tensor,
    created_at: Instant,
    ttl: Duration,
    access_count: u64,
}

impl CacheEntry {
    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

/// The tensor cache provides fast access to recently computed tensors.
pub struct TensorCache {
    cache: RwLock<HashMap<String, CacheEntry>>,
    max_entries: usize,
    default_ttl: Duration,
}

impl TensorCache {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            max_entries: 100,
            default_ttl: Duration::from_secs(300),
        }
    }

    pub fn with_settings(max_entries: usize, ttl: Duration) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            max_entries,
            default_ttl: ttl,
        }
    }

    pub fn insert(&self, key: &str, tensor: Tensor) {
        let entry = CacheEntry {
            tensor,
            created_at: Instant::now(),
            ttl: self.default_ttl,
            access_count: 0,
        };

        let mut cache = self.cache.write();

        if cache.len() >= self.max_entries {
            cache.retain(|_, entry| !entry.is_expired());
        }

        if cache.len() >= self.max_entries {
            if let Some(least_key) = cache
                .iter()
                .min_by_key(|(_, e)| e.access_count)
                .map(|(k, _)| k.clone())
            {
                cache.remove(&least_key);
            }
        }

        cache.insert(key.to_string(), entry);
        debug!("cached tensor: {} (total: {})", key, cache.len());
    }

    pub fn get(&self, key: &str) -> Option<Tensor> {
        let mut cache = self.cache.write();
        if let Some(entry) = cache.get_mut(key) {
            if entry.is_expired() {
                cache.remove(key);
                return None;
            }
            entry.access_count += 1;
            Some(entry.tensor.clone())
        } else {
            None
        }
    }

    pub fn contains(&self, key: &str) -> bool {
        let cache = self.cache.read();
        cache
            .get(key)
            .map(|e| !e.is_expired())
            .unwrap_or(false)
    }

    pub fn remove(&self, key: &str) -> bool {
        self.cache.write().remove(key).is_some()
    }

    pub fn clear(&self) {
        self.cache.write().clear();
        debug!("tensor cache cleared");
    }

    pub fn evict_expired(&self) {
        let mut cache = self.cache.write();
        let before = cache.len();
        cache.retain(|_, entry| !entry.is_expired());
        let after = cache.len();
        if before != after {
            debug!("evicted {} expired cache entries", before - after);
        }
    }
}

impl Default for TensorCache {
    fn default() -> Self {
        Self::new()
    }
}
