use std::convert::Infallible;

use bytes::Bytes;
use http::{Request, Response};
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use uuid::Uuid;

use crate::client::forward_http_request;
use crate::config::get_global_config;
use crate::filters::is_domain_blacklisted;
use crate::schemas::HttpRequest;

#[tracing::instrument(level = "info", name = "ProcessHTTPRequest")]
pub async fn process_http_request(
    req: Request<Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let config = get_global_config();

    let req_id = Uuid::new_v4();
    tracing::info!("Received request ID {}", req_id);

    let method = req.method().to_owned().to_string();
    let uri = req.uri().to_owned();
    let version = req.version();
    let headers = req.headers().to_owned();
    let body = match req.collect().await.ok() {
        Some(b) => b.to_bytes(),
        None => Bytes::new(),
    };

    if config.block_ads {
        let host = headers
            .get("host")
            .and_then(|h| h.to_str().ok())
            .unwrap_or_default();
        if is_domain_blacklisted(host) {
            tracing::info!("The host {} is blacklisted, returning 403 Forbidden", host);
            let mut forbidden_response = Response::new(Full::new(Bytes::from("403 Forbidden")));
            *forbidden_response.status_mut() = http::StatusCode::FORBIDDEN;
            return Ok(forbidden_response);
        }
    }

    let http_request_schema = HttpRequest {
        method,
        uri,
        version,
        headers,
        body: Some(body),
    };
    match forward_http_request(req_id, http_request_schema).await {
        Ok(resp) => Ok(resp),
        Err(e) => {
            tracing::error!(error = %e, "Error forwarding HTTP request for request ID {}", req_id);
            Ok(Response::new(Full::new(Bytes::from(
                "Error forwarding HTTP request",
            ))))
        }
    }
}
