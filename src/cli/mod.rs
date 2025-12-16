mod commands;
pub mod types;

pub use commands::{ProxyCommand, ScanCommand};

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "network-administrator",
    version = env!("CARGO_PKG_VERSION"),
    author = "dosquisd",
    about = "A powerful HTTP/HTTPS proxy for network administration and analysis",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Proxy(ProxyCommand),
    Scan(ScanCommand),
}
