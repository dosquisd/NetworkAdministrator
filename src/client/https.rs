use std::net::SocketAddr;

use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    time::{self as TokioTime, Duration},
};
use tokio_native_tls::TlsStream;
use uuid::Uuid;

use crate::ads::{analyze_and_modify_request, analyze_and_modify_response, inject_script};
use crate::config::get_global_config;
use crate::filters::is_domain_blacklisted;
use crate::schemas::{HttpsRequest, HttpsResponse};
use crate::utils::{
    DNS_RESOLVER,
    decoders::{decode_brotli, decode_deflate, decode_gzip, decode_zstd},
    http::{read_http_stream, read_stream_response, write_request, write_response},
};

#[tracing::instrument(
    level = "info",
    name = "ForwardHTTPSRequest",
    skip(req_params, client_stream)
)]
pub async fn forward_https_request_tunnel(
    req_id: Uuid,
    client_stream: &mut TcpStream,
    req_params: HttpsRequest,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 2. Connect to destination server
    let (host, port_str) = req_params.uri.split_once(':').ok_or("Invalid authority")?;
    let port: u16 = port_str.parse()?;

    tracing::info!("Resolving DNS for {}", host);

    // Resolve the host using the async DNS resolver
    let lookup = DNS_RESOLVER.lookup_ip(host).await?;
    let ip = lookup.iter().next().ok_or("No IP address found")?;

    tracing::info!("Resolved {} to {}", host, ip);

    // Connect directly to the resolved IP address
    let addr = SocketAddr::new(ip, port);
    let mut dest_stream =
        TokioTime::timeout(Duration::from_secs(5), TcpStream::connect(addr)).await??;

    tracing::info!("Connected to {}", addr);

    // 3. Send back 200 Connection Established to the client
    let client_response = format!("{} 200 Connection Established\r\n\r\n", req_params.version);
    client_stream.write_all(client_response.as_bytes()).await?;

    // 4. Tunnel data between client and destination server
    tracing::info!("Establishing HTTPS tunnel for request ID {}", req_id);

    match tokio::io::copy_bidirectional(client_stream, &mut dest_stream).await {
        Ok((client_to_server, server_to_client)) => {
            tracing::info!(
                bytes_up = client_to_server,
                bytes_down = server_to_client,
                "Closed HTTPS tunnel successfully for request ID {}",
                req_id
            );
        }
        Err(e) => {
            tracing::error!(error = %e, error_kind = ?e.kind(), "Tunnel error for request ID {}", req_id);
        }
    }

    Ok(())
}

pub async fn forward_https_request_no_tunnel(
    req_id: Uuid,
    client_tls_stream: &mut TlsStream<&mut TcpStream>,
    dest_tls_stream: &mut TlsStream<&mut TcpStream>,
    version: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    loop {
        tokio::select! {
                http_request = read_http_stream(client_tls_stream) => {
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
                            let host = request.uri.split(':').next().unwrap_or_default();
                            if is_domain_blacklisted(host) {
                                tracing::info!("Blocking ad request for request ID {}", req_id);

                                let response = HttpsResponse {
                                    version: version.to_string(),
                                    status_code: 204,  // No Content
                                    status_text: "No Content".to_string(),
                                    headers: Default::default(),
                                    body: Some("Blocked by Network Administrator".as_bytes().to_vec()),
                                };

                                write_response(client_tls_stream, &response).await?;
                                continue;
                            }

                            request
                        }
                        false => http_request,
                };

                write_request(dest_tls_stream, &modified_request).await?;
            }

            http_response = read_stream_response(dest_tls_stream) => {
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

                if let Some(encoding) = http_response.headers.get("Content-Encoding") && let Some(body) = http_response.body.as_ref() {
                    let encodings: Vec<&str> = encoding.split(',')
                        .map(|e| e.trim())
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .collect();

                    tracing::debug!(
                        "Decoding chain for request ID {}: {:?}",
                        req_id,
                        encodings
                    );

                    let mut body = body.clone();
                    for enc in encodings {
                        let original_size = body.len();

                        body = match enc {
                            "br" => {
                                let decompressed = decode_brotli(&body[..])?;
                                tracing::debug!(
                                    "Decompressed Brotli: {} → {} bytes for request ID {}",
                                    original_size,
                                    decompressed.len(),
                                    req_id
                                );
                                decompressed
                            }
                            "gzip" => {
                                let decompressed = decode_gzip(&body[..])?;
                                tracing::debug!(
                                    "Decompressed gzip: {} → {} bytes for request ID {}",
                                    original_size,
                                    decompressed.len(),
                                    req_id
                                );
                                decompressed
                            }
                            "deflate" => {
                                let decompressed = decode_deflate(&body[..])?;
                                tracing::debug!(
                                    "Decompressed deflate: {} → {} bytes for request ID {}",
                                    original_size,
                                    decompressed.len(),
                                    req_id
                                );
                                decompressed
                            }
                            "zstd" => {
                                let decompressed = decode_zstd(&body)?;
                                tracing::debug!(
                                    "Decompressed zstd: {} → {} bytes for request ID {}",
                                    original_size,
                                    decompressed.len(),
                                    req_id
                                );
                                decompressed
                            }
                            "identity" | "" => {
                                // No encoding or identity (no-op)
                                body
                            }
                            unknown => {
                                tracing::warn!(
                                    "Unknown encoding '{}' for request ID {}, skipping",
                                    unknown,
                                    req_id
                                );
                                body
                            }
                        };
                    }

                    http_response.body = Some(body);
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

                write_response(client_tls_stream, &modified_response).await?;
            }
        }
    }

    Ok(())
}
