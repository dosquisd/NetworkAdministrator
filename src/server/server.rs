use std::net::IpAddr;
use std::net::SocketAddr;

use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto;
use tokio::net::TcpListener;

use crate::server::services::handler_request;

pub async fn start_server(
    host: String,
    port: u16,
    is_v4: Option<bool>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ip = match is_v4 {
        Some(false) => IpAddr::V6(host.parse().unwrap()),
        _ => IpAddr::V4(host.parse().unwrap()),
    };

    let addr = SocketAddr::new(ip, port);
    println!("Starting server at http://{}", addr);

    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = auto::Builder::new(TokioExecutor::new())
                .serve_connection(io, service_fn(handler_request))
                .await
            {
                eprintln!("Error serving connection: {}", err);
            }
        });
    }
}
