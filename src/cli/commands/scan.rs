use clap::Parser;

use crate::cli::types::OutputFormat;
use crate::config::{ARP_RETRIES, ARP_TIMEOUT_SECS};
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

    #[arg(short = 'o', long = "output", default_value_t = OutputFormat::Txt, value_enum, help = "Output format to show raw captured data")]
    pub output_format: OutputFormat,

    #[arg(short = 't', long = "timeout", help = format!("Timeout in seconds for each ARP request [default: {}]", ARP_TIMEOUT_SECS))]
    pub timeout_secs: Option<f32>,

    #[arg(
        short = 'v',
        long = "verbose",
        help = "Enable verbose output for debugging"
    )]
    pub verbose: bool,

    #[arg(
        short = 'j',
        long = "jobs",
        help = "Number of threads to use for scanning [default: number of CPU cores]"
    )]
    pub num_threads: Option<usize>,

    #[arg(short = 'r', long = "retries", help = format!("Number of retries for each ARP request [default: {}]", ARP_RETRIES))]
    pub retries: Option<usize>,
}

impl ScanCommand {
    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        scan_network(
            &self.network_ip,
            &self.interface_name,
            self.timeout_secs,
            self.output_format,
            self.verbose,
            self.num_threads,
            self.retries,
        )
    }
}
