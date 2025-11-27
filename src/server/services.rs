use std::convert::Infallible;
use std::net::SocketAddr;

use http::{Request, Response};
use http_body_util::{BodyExt, Full};
use hyper::body;
use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    time::{self as TokioTime, Duration},
};

use super::utils::{DNS_RESOLVER, read_all_buffer};

#[tracing::instrument(level = "info", name = "HTTPSHandler")]
pub async fn https_handler(
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
    let first_line = *lines.get(0).unwrap_or(&"");
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

    // 2. Connect to destination server
    let (host, port_str) = _authority.split_once(':').ok_or("Invalid authority")?;
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
    let client_response = format!("{} 200 Connection Established\r\n\r\n", _version);
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

#[tracing::instrument(level = "info", name = "HTTPHandler")]
pub async fn http_handler(
    req: Request<body::Incoming>,
) -> Result<Response<Full<body::Bytes>>, Infallible> {
    let req_id = uuid::Uuid::new_v4();
    tracing::info!("Received request ID {}", req_id);

    let method = req.method().to_owned();
    let uri = req.uri().to_owned();
    let version = req.version();
    let headers = req.headers().to_owned();
    let body = req.collect().await.unwrap().to_bytes();
    static SCHEME: &str = "http";

    let client_builder = reqwest::ClientBuilder::new();
    let client_builder = match version {
        http::Version::HTTP_09 => client_builder.http09_responses(),
        http::Version::HTTP_2 => client_builder.http2_prior_knowledge(),
        // HTTP/1.0, HTTP/1.1, HTTP/3.0 (for the last one, I did not find the property methods)
        _ => client_builder.http1_only(),
    };

    let client = client_builder
        .build()
        .expect("Error creating the http client");

    // This should never failed if the server is acting as a proxy
    if let None = uri.authority() {
        tracing::error!("No authority found in the URI");
        return Ok(Response::new(Full::new(body::Bytes::from(
            "Error: No authority found in the URI",
        ))));
    }

    let url = format!(
        "{}://{}{}",
        SCHEME,
        uri.authority().unwrap(),
        uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/")
    );

    let reqwest_method = match method.as_str() {
        "GET" => reqwest::Method::GET,
        "POST" => reqwest::Method::POST,
        "PUT" => reqwest::Method::PUT,
        // "OPTIONS" => reqwest::Method::OPTIONS,
        "DELETE" => reqwest::Method::DELETE,
        "HEAD" => reqwest::Method::HEAD,
        // "CONNECT" => reqwest::Method::CONNECT,
        "PATCH" => reqwest::Method::PATCH,
        // "TRACE" => reqwest::Method::TRACE,
        _ => {
            tracing::warn!("Unsupported HTTP method: {}", method);
            reqwest::Method::GET
        }
    };

    tracing::info!(
        "Forwarding request ID {}: {} {}",
        req_id,
        reqwest_method,
        url
    );
    let response = client
        .request(reqwest_method, url)
        .headers(headers)
        .body(body)
        .send()
        .await;

    // Third step. Send the response back to the client.
    match response {
        Ok(resp) => {
            let status = resp.status();
            let resp_headers = resp.headers().clone();
            let resp_body = resp.bytes().await;

            match resp_body {
                Ok(bytes) => {
                    let mut builder = Response::builder().status(status);
                    for (key, value) in resp_headers.iter() {
                        builder = builder.header(key, value);
                    }

                    return Ok(builder.body(Full::new(body::Bytes::from(bytes))).unwrap());
                }
                Err(e) => {
                    let error_response = format!("Error reading response body: {}", e);
                    tracing::error!(error_response);
                    return Ok(Response::new(Full::new(body::Bytes::from(error_response))));
                }
            }
        }
        Err(err) => {
            tracing::error!("Error making request to destination server: {}", err);
            Ok(Response::new(Full::new(body::Bytes::from(format!(
                "Error: {}",
                err
            )))))
        }
    }
}
