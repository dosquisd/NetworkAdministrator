pub mod constants;
pub mod settings;

pub use constants::{
    ARP_REQUEST_INTERVAL_MSECS, ARP_RETRIES, ARP_TIMEOUT_SECS, CERT_DAYS_VALID, CERT_PATH,
    CONFIG_PATH,
};
pub use settings::{ProxyConfig, get_global_config, set_global_config};
