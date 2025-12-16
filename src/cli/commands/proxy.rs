use clap::Parser;

use crate::{
    admin::start_admin_server,
    cli::types::{LogFormat, LogLevel},
    config::{ProxyConfig, set_global_config},
    logging::{LogConfig, configure_global_tracing},
    server::start_proxy_server,
};

#[derive(Parser, Debug)]
#[command(about = "Start the HTTP/HTTPS proxy server")]
pub struct ProxyCommand {
    #[arg(
        short = 'H',
        long,
        default_value = "127.0.0.1",
        help = "Host address to bind the proxy server"
    )]
    pub host: String,

    #[arg(
        short = 'p',
        long,
        default_value = "8080",
        help = "Port number to bind the proxy server"
    )]
    pub port: u16,

    #[arg(long, default_value = "false", help = "Force IPv6 usage")]
    pub ipv6: bool,

    #[arg(
        short,
        long,
        default_value = "info",
        value_enum,
        help = "Logging level"
    )]
    pub log_level: LogLevel,

    #[arg(long, help = "Path to log file (if not specified, logs go to stdout)")]
    pub log_file: Option<String>,

    #[arg(long, default_value = "pretty", value_enum, help = "Log output format")]
    pub log_format: LogFormat,

    #[arg(
        long,
        help = "Maximum number of log files to retain (only applies if log_file is set)"
    )]
    pub log_max_files: Option<usize>,

    #[arg(long, default_value = "8000", help = "Administrative interface port")]
    pub admin_port: u16,

    #[arg(
        long,
        default_value = "false",
        help = "Enable TLS interception (requires CA certificate installed)"
    )]
    pub intercept_tls: bool,

    #[arg(
        long,
        default_value = "false",
        help = "Enable ad blocking (blocks known ad/tracker domains)"
    )]
    pub block_ads: bool,

    #[arg(long, default_value = "false", help = "Enable response caching")]
    pub cache_enabled: bool,
}

impl ProxyCommand {
    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if (self.admin_port == 0 || self.port == 0) && self.admin_port == self.port {
            eprintln!("Error: Admin port cannot be the same as the proxy server port.");
            std::process::exit(1);
        }

        // Configure logging based on CLI options
        let log_config = LogConfig {
            level: self.log_level,
            format: self.log_format,
            file_path: self.log_file.clone(),
            max_log_files: self.log_max_files,
        };

        configure_global_tracing(log_config);

        // Display startup banner
        println!("");
        println!("╔═══════════════════════════════════════════════╗");
        println!(
            "║   Network Administrator Proxy Server v{}   ║",
            env!("CARGO_PKG_VERSION")
        );
        println!("╚═══════════════════════════════════════════════╝");
        println!("");
        println!("Configuration:");
        println!("  → Host: {}", self.host);
        println!("  → Port: {}", self.port);
        println!("  → IPv6: {}", self.ipv6);
        println!("  → Log Level: {:?}", self.log_level);
        println!("  → Log Format: {:?}", self.log_format);

        if let Some(ref file) = self.log_file {
            println!("  → Log File: {}", file);
        }

        println!("");
        println!("Features:");
        println!(
            "  → TLS Interception: {}",
            if self.intercept_tls {
                "✓ Enabled"
            } else {
                "✗ Disabled"
            }
        );
        println!(
            "  → Ad Blocking: {}",
            if self.block_ads {
                "✓ Enabled"
            } else {
                "✗ Disabled"
            }
        );
        println!(
            "  → Caching: {}",
            if self.cache_enabled {
                "✓ Enabled"
            } else {
                "✗ Disabled"
            }
        );
        println!("");

        // Set global configuration
        let config = ProxyConfig::from_cli(self);
        set_global_config(config);

        // Start servers
        let host = self.host.clone();
        let is_v4 = if self.ipv6 { Some(false) } else { None };

        let proxy_handle = tokio::spawn(start_proxy_server(host.clone(), self.port, is_v4));
        let admin_handle = tokio::spawn(start_admin_server(host, self.admin_port, is_v4));

        tokio::select! {
            result = proxy_handle => {
                if let Err(e) = result {
                    tracing::error!("Proxy server panicked: {:?}", e);
                }
            }
            result = admin_handle => {
                if let Err(e) = result {
                    tracing::error!("Admin server panicked: {:?}", e);
                }
            }
        }

        Ok(())
    }
}
