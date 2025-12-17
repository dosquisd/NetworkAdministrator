use std::collections::HashMap;
use std::time::SystemTime;

use bytes::Bytes;


#[derive(Clone)]
pub struct CachedResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Bytes,
    pub cached_at: SystemTime,
    pub expires_at: SystemTime,
}

impl CachedResponse {
    pub fn is_expired(&self) -> bool {
        SystemTime::now() >= self.expires_at
    }
}
