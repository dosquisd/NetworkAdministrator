use std::path::Path;
use std::sync::LazyLock;

pub const CERT_DAYS_VALID: usize = 365;
pub const CERT_PATH: LazyLock<&'static Path> = LazyLock::new(|| Path::new("./certs"));
