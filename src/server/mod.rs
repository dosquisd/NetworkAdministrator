use std::net::SocketAddr;

use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto;
use tokio::net::TcpListener;

use crate::config::get_global_config;
use crate::filters::is_domain_whitelisted;
use crate::proxy::{
    process_http_request, process_https_request, process_https_request_with_interception,
};
use crate::utils::{
    DNS_RESOLVER,
    buffer::{parse_first_line_buffer, read_first_line_buffer},
};

#[tracing::instrument(level = "info", name = "Server")]
pub async fn start_proxy_server(
    host: String,
    port: u16,
    is_v4: Option<bool>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let lookup = DNS_RESOLVER.lookup_ip(host).await?;
    let ip = match is_v4 {
        Some(false) => lookup
            .iter()
            .find(|ip| ip.is_ipv6())
            .ok_or("No IPv6 address found for the specified host")?,
        _ => lookup
            .iter()
            .find(|ip| ip.is_ipv4())
            .ok_or("No IPv4 address found for the specified host")?,
    };

    let addr = SocketAddr::new(ip, port);
    tracing::info!("Starting proxy server at http://{}", addr);

    let listener = TcpListener::bind(addr).await?;

    loop {
        let (mut stream, peer_addr) = listener.accept().await?;

        tracing::info!("Accepted connection from {}", peer_addr);

        tokio::task::spawn(async move {
            let config = get_global_config();

            let mut buffer = vec![0u8; 1024];
            match stream.peek(&mut buffer).await {
                Ok(n) if n > 0 => {
                    if buffer.starts_with(b"CONNECT") {
                        tracing::info!("Detected HTTPS connection from {}", peer_addr);

                        let first_line = read_first_line_buffer(buffer.as_ref())
                            .await
                            .unwrap_or_default();
                        let (_, authority, _) =
                            parse_first_line_buffer(first_line).unwrap_or_default();
                        let host = authority.split(':').next().unwrap_or_default().to_string();
                        let is_whitelisted = is_domain_whitelisted(&host);

                        if is_whitelisted {
                            tracing::info!(
                                "The host {} is whitelisted, skipping interception",
                                host
                            );
                        }

                        if config.intercept_tls && !is_whitelisted {
                            if let Err(e) =
                                process_https_request_with_interception(&mut stream).await
                            {
                                tracing::error!(
                                    "Error processing HTTPS request (interception): {e}"
                                );
                            }
                        } else {
                            if let Err(e) = process_https_request(&mut stream).await {
                                tracing::error!(
                                    "Error processing HTTPS request (no interception): {e}"
                                );
                            }
                        }
                    } else {
                        tracing::info!("Detected HTTP connection from {}", peer_addr);

                        let io = TokioIo::new(stream);
                        if let Err(err) = auto::Builder::new(TokioExecutor::new())
                            .serve_connection(io, service_fn(process_http_request))
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
