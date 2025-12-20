pub mod handlers;
pub mod routes;

use std::net::SocketAddr;

use axum::Router;
use tower_http::cors::{CorsLayer, Any};
use tokio::net::TcpListener;

use crate::utils::DNS_RESOLVER;
use routes::{create_config_routes, create_health_routes, create_list_routes};

#[tracing::instrument(level = "info", name = "Admin Server")]
pub async fn start_admin_server(
    host: String,
    port: u16,
    is_v4: Option<bool>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(Any)
        .allow_headers(Any);

    let app = Router::new()
        .merge(create_config_routes())
        .merge(create_health_routes())
        .merge(create_list_routes())
        .layer(cors);

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
    tracing::info!("Starting admin server at http://{}", addr);
    let listener = TcpListener::bind(addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
