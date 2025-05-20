use actix_web::rt;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct CacheEntry<V> {
    value: V,
    expiry: Option<Instant>,
    last_accessed: Instant,
}

#[derive(Debug, Clone)]
pub struct MemoryCache<K: Eq + Hash + Clone, V> {
    data: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
}

#[derive(Debug)]
pub enum CacheError {
    Serialization,
    LockPoisoned,
}

pub trait Cache {
    type Key;
    type Value;
    type Error;

    fn get(&self, key: &Self::Key) -> Result<Option<Self::Value>, Self::Error>;
    fn set(
        &self,
        key: &Self::Key,
        value: Self::Value,
        ttl: Option<Duration>,
    ) -> Result<(), Self::Error>;
    fn delete(&self, key: &Self::Key) -> Result<bool, Self::Error>;
    fn clear(&self) -> Result<(), Self::Error>;
}

impl<K, V> Default for MemoryCache<K, V>
where
    K: Eq + Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<K, V> MemoryCache<K, V>
where
    K: Eq + Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn start_cleanup_task(&self, interval_secs: u64) {
        let cache_data = self.data.clone();

        rt::spawn(async move {
            let mut interval = actix_web::rt::time::interval(Duration::from_secs(interval_secs));

            loop {
                interval.tick().await;
                Self::cleanup_expired_entries(&cache_data);
            }
        });
    }

    fn cleanup_expired_entries(data: &Arc<RwLock<HashMap<K, CacheEntry<V>>>>) {
        let now = Instant::now();

        if let Ok(mut cache) = data.write() {
            cache.retain(|_, entry| {
                if let Some(expiry) = entry.expiry {
                    now < expiry
                } else {
                    true
                }
            });
        }
    }

    pub fn len(&self) -> usize {
        self.data.read().map(|cache| cache.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<K, V> Cache for MemoryCache<K, V>
where
    K: Eq + Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    type Key = K;
    type Value = V;
    type Error = CacheError;

    fn get(&self, key: &K) -> Result<Option<V>, CacheError> {
        let now = Instant::now();
        let mut entry_to_return = None;

        // First try with a read lock
        {
            let cache = self.data.read().map_err(|_| CacheError::LockPoisoned)?;

            if let Some(entry) = cache.get(key) {
                // Check if the entry is expired
                if let Some(expiry) = entry.expiry {
                    if now > expiry {
                        return Ok(None);
                    }
                }

                entry_to_return = Some(entry.value.clone());
            }
        }

        // If we found an entry, update the last accessed time
        if entry_to_return.is_some() {
            let mut cache = self.data.write().map_err(|_| CacheError::LockPoisoned)?;

            if let Some(entry) = cache.get_mut(key) {
                entry.last_accessed = now;
            }
        }

        Ok(entry_to_return)
    }

    fn set(&self, key: &K, value: V, ttl: Option<Duration>) -> Result<(), CacheError> {
        let now = Instant::now();
        let expiry = ttl.map(|duration| now + duration);

        let mut cache = self.data.write().map_err(|_| CacheError::LockPoisoned)?;

        cache.insert(
            key.clone(),
            CacheEntry {
                value,
                expiry,
                last_accessed: now,
            },
        );

        Ok(())
    }

    fn delete(&self, key: &K) -> Result<bool, CacheError> {
        let mut cache = self.data.write().map_err(|_| CacheError::LockPoisoned)?;
        Ok(cache.remove(key).is_some())
    }

    fn clear(&self) -> Result<(), CacheError> {
        let mut cache = self.data.write().map_err(|_| CacheError::LockPoisoned)?;
        cache.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_memory_cache() {
        let cache = MemoryCache::default();
        cache
            .set(&"key1", "value1", Some(Duration::from_secs(10)))
            .unwrap();
        assert_eq!(cache.get(&"key1").unwrap(), Some("value1"));

        cache
            .set(&"key1", "value2", Some(Duration::from_secs(1)))
            .unwrap();
        assert_eq!(cache.get(&"key1").unwrap(), Some("value2"));

        thread::sleep(Duration::from_secs(2));
        assert_eq!(cache.get(&"key1").unwrap(), None);
    }
}
