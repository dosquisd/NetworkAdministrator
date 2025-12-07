use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn as_tracing_level(&self) -> tracing::Level {
        match self {
            LogLevel::Trace => tracing::Level::TRACE,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Error => tracing::Level::ERROR,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum LogFormat {
    // Human-readable format with colors
    Pretty,

    // JSON format for machine parsing
    Json,

    // Compact single-line format
    Compact,
}

#[derive(Parser, Debug)]
#[command(
    name = "network-administrator",
    version = env!("CARGO_PKG_VERSION"),
    author = "dosquisd",
    about = "A powerful HTTP/HTTPS proxy for network administration and analysis",
    long_about = None
)]
pub struct Cli {
    // Host address to bind the proxy server
    #[arg(short = 'H', long, default_value = "127.0.0.1")]
    pub host: String,

    // Port number to bind the proxy server
    #[arg(short = 'p', long, default_value = "8080")]
    pub port: u16,

    // Force IPv6 usage
    #[arg(long, default_value = "false")]
    pub ipv6: bool,

    // Logging level
    #[arg(short, long, default_value = "info", value_enum)]
    pub log_level: LogLevel,

    // Path to log file (if not specified, logs go to stdout)
    #[arg(long)]
    pub log_file: Option<String>,

    // Log output format
    #[arg(long, default_value = "pretty", value_enum)]
    pub log_format: LogFormat,

    // Administrative interface port (if not specified, admin interface is disabled)
    #[arg(long, default_value = "8000")]
    pub admin_port: u16,

    // Enable TLS interception (requires CA certificate installed)
    #[arg(long, default_value = "false")]
    pub intercept_tls: bool,

    // Enable ad blocking (blocks known ad/tracker domains)
    #[arg(long, default_value = "false")]
    pub block_ads: bool,

    // Enable response caching
    #[arg(long, default_value = "false")]
    pub cache_enabled: bool,
}
