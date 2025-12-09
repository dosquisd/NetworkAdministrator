use clap::Parser;

use crate::{
    admin::start_admin_server,
    cli::types::{LogFormat, LogLevel},
    config::{ProxyConfig, set_global_config},
    logging::{LogConfig, configure_global_tracing},
    server::start_proxy_server,
};

#[derive(Parser, Debug)]
#[command(
    about = "Start the HTTP/HTTPS proxy server",
)]
pub struct ProxyCommand {
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
