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

use crate::client::{forward_http_request, forward_https_request_tunnel};
use crate::config::get_global_config;
use crate::schemas::{HTTPRequestSchema, HTTPSRequestSchema};
use crate::utils::{
    DNS_RESOLVER,
    ads::{analyze_and_modify_request, analyze_and_modify_response, inject_script, is_ad_request},
    http::{HttpResponse, read_http_stream, read_stream_response, write_request, write_response},
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

    loop {
        tokio::select! {
                http_request = read_http_stream(&mut client_tls_stream) => {
                    if let Err(e) = http_request {
                        tracing::error!("Error reading HTTP request from client TLS stream for request ID {}: {}", req_id, e);
                        break;
                    }
                    let http_request = http_request.unwrap();

                    tracing::debug!("Intercepted HTTPS request ID {}: {:?}", req_id, http_request);

                    let config = get_global_config();
                    let modified_request = match config.block_ads {
                        true => {
                            let request = analyze_and_modify_request(&http_request);
                            if is_ad_request(&request) {
                                tracing::info!("Blocking ad request for request ID {}", req_id);

                                let response = HttpResponse {
                                    version: https_stream_parser.version.clone(),
                                    status_code: 204,  // No Content
                                    status_text: "No Content".to_string(),
                                    headers: Default::default(),
                                    body: Some("Blocked by Network Administrator".as_bytes().to_vec()),
                                };

                                write_response(&mut client_tls_stream, &response).await?;
                                continue;
                            }

                            request
                        }
                        false => http_request,
                };

                write_request(&mut dest_tls_stream, &modified_request).await?;
            }

            http_response = read_stream_response(&mut dest_tls_stream) => {
                if let Err(e) = http_response {
                    tracing::error!("Error reading HTTP response from destination TLS stream for request ID {}: {}", req_id, e);
                    break;
                }

                let mut http_response = http_response.unwrap();
                tracing::debug!("Intercepted HTTPS response ID {id}: version={version}, status_code={status_code}, status_text={status_text}, headers={headers:?}, body_size={body_size}",
                    id = req_id,
                    version = http_response.version,
                    status_code = http_response.status_code,
                    status_text = http_response.status_text,
                    headers = http_response.headers,
                    body_size = http_response.body.as_ref().map_or(0, |b| b.len())
                );

                if let Some(encoding) = http_response.headers.get("Content-Encoding") {
                    if encoding.contains("br") && http_response.body.is_some() {
                        let compressed = http_response.body.as_ref().unwrap();
                        let mut decompressed = Vec::new();

                        brotli::BrotliDecompress(
                            &mut &compressed[..],
                            &mut decompressed
                        )?;

                        tracing::debug!("Decompressed Brotli: {} → {} bytes",
                            compressed.len(), decompressed.len());

                        http_response.body = Some(decompressed);
                    }
                }

                let mut modified_response = analyze_and_modify_response(&http_response);

                let content_type = modified_response.headers.get("Content-Type");
                if let Some(ct) = content_type && ct.contains("text/html") {
                    // Just for demonstration, we inject a simple script
                    let body = modified_response.body.clone().unwrap_or_default();

                    let charset = ct.split("charset=")
                    .nth(1)
                    .and_then(|c| c.split(';').next())
                    .unwrap_or("utf-8");

                    let body_text = if charset.to_lowercase() == "iso-8859-1" {
                        // Decodificar ISO-8859-1
                        body.iter().map(|&b| b as char).collect::<String>()
                    } else {
                            String::from_utf8_lossy(body.as_ref()).to_string()
                    };

                    let modified_body = inject_script(
                        &body_text,
                        "console.log('Injected script by Network Administrator');",
                    );

                    modified_response.body = Some(modified_body.as_bytes().to_vec());
                }

                write_response(&mut client_tls_stream, &modified_response).await?;
            }
        }
    }

    tracing::info!("Finished TLS Interception request ID {}", req_id);
    Ok(())
}
