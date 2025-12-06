use std::convert::Infallible;
use std::net::SocketAddr;

use bytes::Bytes;
use http::{Request, Response};
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    time::{self as TokioTime, Duration},
};
use tokio_native_tls::TlsAcceptor;
use uuid::Uuid;

use crate::client::{forward_http_request, forward_https_request_no_tunnel, forward_https_request_tunnel};
use crate::schemas::{HTTPRequestSchema, HTTPSRequestSchema};
use crate::utils::{
    DNS_RESOLVER,
    read_headers_buffer,
    stream::parse_stream,
    tls::generate_cert_for_domain,
};

#[tracing::instrument(level = "info", name = "ProcessHTTPRequest")]
pub async fn process_http_request(
    req: Request<Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
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
    let req_id = Uuid::new_v4();

    tracing::info!("Received request ID {}", req_id);

    // 1. Parse request
    // read all buffer until double CRLF
    let buffer = read_headers_buffer(&mut *client_stream).await?;

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

#[tracing::instrument(level = "info", name = "ProcessHTTPSRequestWithInterception")]
pub async fn process_https_request_with_interception(
    client_stream: &mut TcpStream,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let req_id = Uuid::new_v4();
    tracing::info!("Received request ID {}", req_id);

    // This parse CONNECT request
    let https_stream_parser = parse_stream(&mut *client_stream, false, false).await?;
    tracing::debug!(
        "Parsed HTTPS CONNECT request ID {id}: {stream:?}",
        id = req_id,
        stream = https_stream_parser
    );

    // Send 200 Connection Established BEFORE TLS handshake
    let connect_response = format!(
        "{} 200 Connection Established\r\n\r\n",
        https_stream_parser.version
    );
    client_stream.write_all(connect_response.as_bytes()).await?;
    client_stream.flush().await?;

    tracing::info!("Sent 200 Connection Established for request ID {}", req_id);

    // 1. Perform TLS handshake with client using our CA (accept)
    let (host, port_str) = https_stream_parser
        .authority
        .split_once(':')
        .ok_or("Invalid authority")?;
    let port: u16 = port_str.parse()?;

    let (cert_pem, key_pem) = generate_cert_for_domain(&host)?;
    let identity = native_tls::Identity::from_pkcs8(&cert_pem.as_bytes(), &key_pem.as_bytes())?;
    let native_acceptor = native_tls::TlsAcceptor::new(identity)?;
    let tls_acceptor = TlsAcceptor::from(native_acceptor);

    tracing::info!(
        "Starting TLS handshake with client for request ID {}",
        req_id
    );

    // If the TLS handshake fails, could means that the client does not trust our CA
    // TODO: Implement a fallback to plain TCP tunnel if needed
    let mut client_tls_stream = tls_acceptor.accept(client_stream).await.map_err(|e| {
        tracing::error!(
            "TLS handshake with client failed for request ID {}: {}",
            req_id,
            e
        );
        e
    })?;

    tracing::info!(
        "TLS handshake with client succeeded for request ID {}",
        req_id
    );

    // 2. Now connect to the destination server and perform TLS handshake (connect)
    let lookup = DNS_RESOLVER.lookup_ip(host).await?;
    let ip = lookup.iter().next().ok_or("No IP address found")?;
    let dest_addr = SocketAddr::new(ip, port);

    tracing::info!(
        "Connecting to destination server {} for request ID {}",
        https_stream_parser.authority,
        req_id
    );

    let mut dest_tcp_stream =
        TokioTime::timeout(Duration::from_secs(5), TcpStream::connect(dest_addr)).await??;

    let native_connector = native_tls::TlsConnector::builder()
        .danger_accept_invalid_certs(false)
        .build()?;
    let tls_connector = tokio_native_tls::TlsConnector::from(native_connector);
    let mut dest_tls_stream = tls_connector.connect(host, &mut dest_tcp_stream).await?;

    tracing::info!(
        "TLS handshake with destination server {} succeeded for request ID {}",
        host,
        req_id
    );

    // 3. Now we have both TLS streams (client_tls_stream and dest_tls_stream)
    // Here should implement the logic to intercept and process the HTTPS traffic,
    // such as validate headers, methods, block ads, all that stuff that could be interesting
    forward_https_request_no_tunnel(req_id, &mut client_tls_stream, &mut dest_tls_stream, https_stream_parser.version.as_str()).await?;

    tracing::info!("Finished TLS Interception request ID {}", req_id);
    Ok(())
}
