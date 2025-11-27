use std::net::IpAddr;
use std::net::SocketAddr;

use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto;
use tokio::net::TcpListener;

use crate::server::services::{http_handler, https_handler};

#[tracing::instrument(level = "info", name = "Server")]
pub async fn start_server(
    host: String,
    port: u16,
    is_v4: Option<bool>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ip = match is_v4 {
        Some(false) => IpAddr::V6(host.parse()?),
        _ => IpAddr::V4(host.parse()?),
    };

    let addr = SocketAddr::new(ip, port);
    tracing::info!("Starting server at http://{}", addr);

    let listener = TcpListener::bind(addr).await?;

    loop {
        let (mut stream, peer_addr) = listener.accept().await?;

        tracing::info!("Accepted connection from {}", peer_addr);

        tokio::task::spawn(async move {
            let mut buffer = vec![0u8; 1024];

            match stream.peek(&mut buffer).await {
                Ok(n) if n > 0 => {
                    if buffer.starts_with(b"CONNECT") {
                        tracing::info!("Detected HTTPS connection from {}", peer_addr);

                        if let Err(err) = https_handler(&mut stream).await {
                            tracing::error!("Error serving connection: {}", err);
                        }
                    } else {
                        tracing::info!("Detected HTTP connection from {}", peer_addr);

                        let io = TokioIo::new(stream);
                        if let Err(err) = auto::Builder::new(TokioExecutor::new())
                            .serve_connection(io, service_fn(http_handler))
                            .await
                        {
                            tracing::error!("Error serving connection: {}", err);
                        }
                    }
                }
                Ok(_) => {
                    tracing::warn!("No data received from {}", peer_addr);
                }
                Err(e) => {
                    tracing::error!("Error peeking into stream from {}: {}", peer_addr, e);
                }
            };
        });
    }
}
