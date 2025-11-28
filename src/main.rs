use network_administrator::{logging::configure_global_tracing, server};

#[tokio::main(flavor = "multi_thread", worker_threads = 12)]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    configure_global_tracing(tracing::Level::INFO.into());

    tracing::info!("\nStarting Network Administrator Server...");

    let host = String::from("127.0.0.1");
    let port: u16 = 8080;

    server::start_server(host, port, None).await?;

    Ok(())
}
