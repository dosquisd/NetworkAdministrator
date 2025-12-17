pub struct CacheStats {
    pub entries: usize,
    pub capacity: usize,
    pub hit_rate: f64,
    pub hits: u64,
    pub misses: u64,
    pub bytes_saved: u64,
}