use clap::Parser;

#[derive(Parser, Debug)]
#[command(about = "Sniff network traffic and display summaries")]
pub struct SniffCommand {
    // Network interface to sniff on
    #[arg(short, long, default_value = "eth0")]
    pub interface: String,

    // If it is not specified duration or count, the sniff will only take a snapshot of current traffic

    // Duration to sniff in seconds
    #[arg(short, long)]
    pub duration: Option<u64>,

    // Number of packets to capture
    #[arg(short, long)]
    pub count: Option<u32>,

    // The summary of all the devices will be displayed,
    // but it's possible to save to a file the raw captured data
    #[arg(long)]
    pub output_file: Option<String>,
}

impl SniffCommand {
    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!(
            "Sniffing on interface: {}, duration: {:?}, count: {:?}",
            self.interface, self.duration, self.count
        );
        todo!("Implement sniffing functionality here");
    }
}
