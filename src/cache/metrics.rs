use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Default)]
pub struct CacheMetrics {
    pub hits: AtomicU64,
    pub misses: AtomicU64,
    pub bytes_saved: AtomicU64,
}

impl CacheMetrics {
    pub fn record_hit(&self, size: usize) {
        self.hits.fetch_add(1, Ordering::Relaxed);
        self.bytes_saved.fetch_add(size as u64, Ordering::Relaxed);
    }

    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed) as f64;
        let misses = self.misses.load(Ordering::Relaxed) as f64;

        if hits + misses == 0.0 {
            return 0.0;
        }

        hits / (hits + misses)
    }
}
