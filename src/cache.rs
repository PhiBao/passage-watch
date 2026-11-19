//! A small in-process TTL cache. Used for CMC catalog / contract / multiplier,
//! Jupiter sell quotes, and market-price snapshots so repeated reads during a
//! judge's session do not hammer upstream APIs. The cache is for latency and
//! display consistency; correctness for a sell quote comes from computing
//! transfer amounts from verified unit data, not from this cache.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Clone)]
struct Entry {
    value: String,
    expires: Instant,
}

pub struct Cache {
    ttl: Duration,
    store: Mutex<HashMap<String, Entry>>,
}

impl Cache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            store: Mutex::new(HashMap::new()),
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let mut map = self.store.lock().unwrap();
        match map.get(key) {
            Some(e) if e.expires > Instant::now() => Some(e.value.clone()),
            Some(_) => {
                map.remove(key);
                None
            }
            None => None,
        }
    }

    pub fn put(&self, key: &str, value: impl Into<String>) {
        self.store.lock().unwrap().insert(
            key.to_string(),
            Entry {
                value: value.into(),
                expires: Instant::now() + self.ttl,
            },
        );
    }

    pub fn invalidate(&self, key: &str) {
        self.store.lock().unwrap().remove(key);
    }
}

/// Returns cached value or runs loader and stores it. The loader is skipped when fresh.
pub async fn get_or_load<F, Fut, E>(cache: &Cache, key: &str, loader: F) -> Result<String, E>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<String, E>>,
{
    if let Some(v) = cache.get(key) {
        return Ok(v);
    }
    let v = loader().await?;
    cache.put(key, v.clone());
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expires_after_ttl() {
        let c = Cache::new(Duration::from_millis(50));
        c.put("k", "v");
        assert_eq!(c.get("k").as_deref(), Some("v"));
        std::thread::sleep(Duration::from_millis(70));
        assert!(c.get("k").is_none());
    }

    #[test]
    fn invalidate_removes() {
        let c = Cache::new(Duration::from_secs(5));
        c.put("k", "v");
        c.invalidate("k");
        assert!(c.get("k").is_none());
    }

    #[tokio::test]
    async fn get_or_load_caches() {
        let c = Cache::new(Duration::from_secs(5));
        let v = get_or_load(&c, "k", || async { Ok::<_, ()>("v".into()) }).await;
        assert_eq!(v, Ok("v".to_string()));
        assert_eq!(c.get("k").as_deref(), Some("v"));
    }
}
