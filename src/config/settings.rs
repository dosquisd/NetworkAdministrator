use std::sync::{Arc, LazyLock, RwLock};

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub intercept_tls: bool,
    pub block_ads: bool,
    pub cache_enabled: bool,
}

impl ProxyConfig {
    pub fn from_cli(cli: &crate::cli::ProxyCommand) -> Self {
        Self {
            intercept_tls: cli.intercept_tls,
            block_ads: cli.block_ads,
            cache_enabled: cli.cache_enabled,
        }
    }
}

pub static GLOBAL_CONFIG: LazyLock<Arc<RwLock<Option<ProxyConfig>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

pub fn set_global_config(config: ProxyConfig) {
    let mut global = GLOBAL_CONFIG.write().unwrap();
    *global = Some(config);
}

pub fn get_global_config() -> ProxyConfig {
    let global = GLOBAL_CONFIG.read().unwrap();
    global.as_ref().unwrap().clone()
}
