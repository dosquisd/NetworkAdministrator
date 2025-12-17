#[derive(Clone)]
pub struct CacheConfig {
    pub capacity_mb: usize,
    pub cleanup_interval_secs: u64,
}
