use clap::Parser;

use network_administrator::cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Proxy(proxy_cmd) => {
            proxy_cmd.execute().await?;
        }
        Commands::Scan(scan_cmd) => {
            scan_cmd.execute().await?;
        }
    }

    Ok(())
}
