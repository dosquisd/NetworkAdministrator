use network_administrator::{logging::configure_global_tracing, server};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    configure_global_tracing(None);

    tracing::info!("Starting Network Administrator Server...");

    let host = String::from("127.0.0.1");
    let port: u16 = 8080;

    let _ = server::start_server(host, port, None).await?;

    Ok(())
}
