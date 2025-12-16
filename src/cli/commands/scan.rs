use clap::Parser;

use crate::config::constants::ARP_TIMEOUT_SECS;
use crate::scan::scan_network;

#[derive(Parser, Debug)]
#[command(about = "Scan network devices and display summary information")]
pub struct ScanCommand {
    #[arg(help = "The IPv4 network to scan in the format xxx.xxx.xxx.xxx/x")]
    pub network_ip: String,

    #[arg(
        short = 'i',
        long = "iface",
        default_value = "eth0",
        help = "Network interface to use for scanning"
    )]
    pub interface_name: String,

    #[arg(short = 'o', long, help = "Output file to save raw captured data")]
    pub output_file: Option<String>,

    #[arg(short = 't', long = "timeout", help = format!("Timeout in seconds for each ARP request [default: {}]", ARP_TIMEOUT_SECS))]
    pub timeout_secs: Option<f32>,
}

impl ScanCommand {
    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        scan_network(&self.network_ip, &self.interface_name, self.timeout_secs)
    }
}
