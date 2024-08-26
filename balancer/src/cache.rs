use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::interval;

struct CacheEntry {
    key: String,
    value: Vec<u8>,
    expires_at: Instant,
}

pub struct SimpleCache {
    store: Arc<Mutex<VecDeque<CacheEntry>>>,
    capacity: usize,
}

impl SimpleCache {
    pub fn new(capacity: usize) -> Self {
        let cache = SimpleCache {
            store: Arc::new(Mutex::new(VecDeque::with_capacity(capacity))),
            capacity,
        };

        // Start background task for proactive garbage collection
        let store_clone = cache.store.clone();
        tokio::spawn(async move {
            Self::garbage_collector(store_clone, Duration::from_secs(60)).await;
        });

        cache
    }

    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        let mut store = self.store.lock().await;
        if let Some(index) = store.iter().position(|entry| entry.key == key) {
            let entry = &store[index];
            if entry.expires_at > Instant::now() {
                return Some(entry.value.clone());
            } else {
                store.remove(index);
            }
        }
        None
    }

    pub async fn set(&self, key: String, value: Vec<u8>, ttl: Duration) {
        let mut store = self.store.lock().await;
        let new_entry = CacheEntry {
            key,
            value,
            expires_at: Instant::now() + ttl,
        };

        if store.len() >= self.capacity {
            store.pop_front(); // Remove oldest entry if at capacity
        }
        store.push_back(new_entry);
    }

    async fn garbage_collector(store: Arc<Mutex<VecDeque<CacheEntry>>>, interval_duration: Duration) {
        let mut interval_timer = interval(interval_duration);
        loop {
            interval_timer.tick().await;
            let mut store = store.lock().await;
            let now = Instant::now();
            store.retain(|entry| entry.expires_at > now);
        }
    }
}