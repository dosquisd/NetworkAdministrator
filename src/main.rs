use clap::Parser;

use network_administrator::cli::Cli;
use network_administrator::config::{ProxyConfig, set_global_config};
use network_administrator::logging::{LogConfig, configure_global_tracing};
use network_administrator::server::start_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Configure logging based on CLI options
    let log_config = LogConfig {
        level: cli.log_level,
        format: cli.log_format,
        file_path: cli.log_file.clone(),
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
    println!("  → Host: {}", cli.host);
    println!("  → Port: {}", cli.port);
    println!("  → IPv6: {}", cli.ipv6);
    println!("  → Log Level: {:?}", cli.log_level);
    println!("  → Log Format: {:?}", cli.log_format);

    if let Some(ref file) = cli.log_file {
        println!("  → Log File: {}", file);
    }

    println!("");
    println!("Features:");
    println!(
        "  → TLS Interception: {}",
        if cli.intercept_tls {
            "✓ Enabled"
        } else {
            "✗ Disabled"
        }
    );
    println!(
        "  → Ad Blocking: {}",
        if cli.block_ads {
            "✓ Enabled"
        } else {
            "✗ Disabled"
        }
    );
    println!(
        "  → Caching: {}",
        if cli.cache_enabled {
            "✓ Enabled"
        } else {
            "✗ Disabled"
        }
    );
    println!("");

    // Set global configuration
    let config = ProxyConfig::from_cli(&cli);
    set_global_config(config);

    // Start the server
    let is_v4 = if cli.ipv6 { Some(false) } else { None };
    start_server(cli.host, cli.port, is_v4).await
}
