pub mod config;
pub use config::CacheConfig;

pub mod metrics;
pub use metrics::CacheMetrics;

pub mod response;
pub use response::CachedResponse;

pub mod stats;
pub use stats::CacheStats;

use std::num::NonZeroUsize;
use std::sync::{Arc, LazyLock, Mutex, RwLock, atomic::Ordering};
use std::time::SystemTime;

use lru::LruCache;

use crate::config::get_global_config;
use crate::schemas::Request;

#[allow(dead_code)]
#[derive(Clone)]
pub struct HttpCache {
    cache: Arc<Mutex<LruCache<String, CachedResponse>>>,
    config: CacheConfig,
    metrics: Arc<CacheMetrics>,
}

impl HttpCache {
    pub fn new(config: CacheConfig) -> Self {
        let cache = Arc::new(Mutex::new(LruCache::new(
            NonZeroUsize::new(config.capacity_mb).unwrap(),
        )));
        let metrics = Arc::new(CacheMetrics::default());

        // Background cleanup task
        let cache_clone = Arc::clone(&cache);
        tokio::spawn(async move {
            Self::cleanup_loop(cache_clone, config.cleanup_interval_secs).await;
        });

        Self {
            cache,
            config,
            metrics,
        }
    }

    #[tracing::instrument(skip(cache), level = "trace", name = "CacheCleanupLoop")]
    pub async fn cleanup_loop(
        cache: Arc<Mutex<LruCache<String, CachedResponse>>>,
        cleanup_interval_secs: u64,
    ) {
        let interval = tokio::time::Duration::from_secs(cleanup_interval_secs);
        let mut ticker = tokio::time::interval(interval);

        loop {
            ticker.tick().await;
            let mut cache_lock = cache.lock().unwrap();
            let now = SystemTime::now();

            // Collect keys of expired entries
            let expired_keys: Vec<String> = cache_lock
                .iter()
                .filter_map(|(key, resp)| {
                    if now >= resp.expires_at {
                        Some(key.clone())
                    } else {
                        None
                    }
                })
                .collect();

            tracing::trace!(
                "Cache cleanup: removing {} expired entries",
                expired_keys.len()
            );

            // Remove expired entries
            for key in expired_keys {
                cache_lock.pop(&key);
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<CachedResponse> {
        let mut cache = self.cache.lock().unwrap();

        if let Some(cached_response) = cache.get(key) {
            if cached_response.is_expired() {
                // Remove expired entry
                cache.pop(key);
                self.metrics.record_miss();
                return None;
            }

            self.metrics.record_hit(cached_response.body.len());
            return Some(cached_response.clone());
        }

        self.metrics.record_miss();
        return None;
    }

    pub fn set(&self, key: String, response: CachedResponse) {
        let mut cache = self.cache.lock().unwrap();
        cache.put(key, response);
    }

    pub fn should_bypass_http_request(&self, request: Request) -> bool {
        // 1. Check if caching is globally disabled
        let config = get_global_config();
        if !config.cache_enabled {
            return true;
        }

        // 2. Check client headers
        let headers = request.get_headers();

        if let Some(value) = headers.get("Cache-Control") {
            if value.contains("no-cache") || value.contains("no-store") {
                tracing::debug!("Cache-Control: no-cache or no-store detected");
                return true;
            }

            // Client requests revalidation (refresh)
            if value.contains("max-age=0") || value.contains("must-revalidate") {
                tracing::debug!("Client requested revalidation");
                return false;
            }
        }

        if let Some(pragma) = headers.get("Pragma") {
            if pragma.contains("no-cache") {
                tracing::debug!("Pragma: no-cache detected");
                return true;
            }
        }

        // 3. Check custom bypass header
        if headers.get("X-Proxy-Bypass-Cache").is_some() {
            tracing::debug!("Custom bypass header detected");
            return true;
        }

        // Maybe here could be implemented URL pattern checks, but I decided to keep it simple for now.

        false
    }

    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
        tracing::info!("Cache cleared");
    }

    pub fn stats(&self) -> CacheStats {
        let cache = self.cache.lock().unwrap();

        CacheStats {
            entries: cache.len(),
            capacity: cache.cap().get(),
            hit_rate: self.metrics.hit_rate(),
            hits: self.metrics.hits.load(Ordering::Relaxed),
            misses: self.metrics.misses.load(Ordering::Relaxed),
            bytes_saved: self.metrics.bytes_saved.load(Ordering::Relaxed),
        }
    }
}

pub static GLOBAL_CACHE: LazyLock<Arc<RwLock<Option<HttpCache>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

pub fn set_global_cache(cache_config: CacheConfig) {
    let mut global = GLOBAL_CACHE.write().unwrap();
    *global = Some(HttpCache::new(cache_config));
}

pub fn get_global_cache() -> HttpCache {
    let global = GLOBAL_CACHE.read().unwrap();
    global.as_ref().unwrap().clone()
}
