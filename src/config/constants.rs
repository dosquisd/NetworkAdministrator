use std::path::Path;
use std::sync::LazyLock;

// ARP configuration
pub const ARP_TIMEOUT_SECS: f32 = 1.0;
pub const ARP_RETRIES: usize = 4;
pub const ARP_REQUEST_INTERVAL_MSECS: u64 = 50;

// Configuration paths
pub const CONFIG_PATH: LazyLock<&'static Path> = LazyLock::new(|| Path::new("./.config"));

// Certificate configuration
pub const CERT_DAYS_VALID: usize = 365;
pub const CERT_PATH: LazyLock<&'static Path> = LazyLock::new(|| Path::new("./certs"));
