use std::convert::Infallible;

use bytes::Bytes;
use http::{Request, Response};
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use tokio::net::TcpStream;

use crate::client::{forward_http_request, forward_https_request_tunnel};
use crate::schemas::{HTTPRequestSchema, HTTPSRequestSchema};
use crate::utils::read_all_buffer;

#[tracing::instrument(level = "info", name = "ProcessHTTPRequest")]
pub async fn process_http_request(
    req: Request<Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let req_id = uuid::Uuid::new_v4();
    tracing::info!("Received request ID {}", req_id);

    let method = req.method().to_owned().to_string();
    let uri = req.uri().to_owned();
    let version = req.version();
    let headers = req.headers().to_owned();
    let body = match req.collect().await.ok() {
        Some(b) => b.to_bytes(),
        None => Bytes::new(),
    };

    // Here should implement the logic to process the HTTP request,
    // such as validate headers, methods, block ads, all that stuff that could be interesting!

    let http_request_schema = HTTPRequestSchema::new(method, uri, version, headers, Some(body));
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

#[tracing::instrument(level = "info", name = "ProcessHTTPSRequest")]
pub async fn process_https_request(
    client_stream: &mut TcpStream,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let req_id = uuid::Uuid::new_v4();

    tracing::info!("Received request ID {}", req_id);

    // 1. Parse request
    // read all buffer until double CRLF
    let buffer = read_all_buffer(&mut *client_stream).await?;

    tracing::debug!(
        "Full received data for HTTPS request ID {}: {:?}",
        req_id,
        buffer.as_str()
    );

    let lines = buffer.split("\r\n").collect::<Vec<&str>>();
    let first_line = *lines.first().unwrap_or(&"");
    let _header_lines = lines // Just for logging purposes
        .iter()
        .skip(1)
        .take_while(|&&line| !line.is_empty())
        .cloned()
        .collect::<Vec<&str>>();

    let [_method, _authority, _version] = first_line.split(' ').collect::<Vec<&str>>()[..] else {
        tracing::error!("Malformed HTTPS request line: {}", first_line);
        return Err("Malformed HTTPS request line".into());
    };

    tracing::debug!(
        "Parsed HTTPS request ID {id}: method={method}, authority={authority}, version={version}, headers={header_lines:?}",
        id = req_id,
        method = _method,
        authority = _authority,
        version = _version,
        header_lines = _header_lines,
    );

    // Here should implement the logic to process the HTTP request,
    // such as validate headers, methods, block ads, all that stuff that could be interesting!

    let https_request_schema = HTTPSRequestSchema::new(
        _method.to_string(),
        _authority.to_string(),
        _version.to_string(),
        _header_lines.iter().map(|s| s.to_string()).collect(),
        None,
    );
    match forward_https_request_tunnel(req_id, client_stream, https_request_schema).await {
        Ok(_) => Ok(()),
        Err(e) => {
            tracing::error!(error = %e, "Error forwarding HTTPS request for request ID {}", req_id);
            Err(e)
        }
    }
}
