use std::collections::HashMap;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tokio_native_tls::TlsStream;

use super::buffer::read_headers_buffer;
use crate::schemas::{HttpsRequest, HttpsResponse};

pub fn parse_headers(lines: &[&str]) -> HashMap<String, String> {
    let mut headers: HashMap<String, String> = HashMap::new();

    // Maybe there's a more efficient way to do this, idk
    for line in lines {
        if let Some((key, value)) = line.split_once(':') {
            let normalized_key = key.trim().to_ascii_lowercase();
            let normalized_value = value.trim().to_string();

            match headers.get_mut(&normalized_key) {
                Some(existing_value) if normalized_key == "set-cookie" => {
                    // Preserve multiple Set-Cookie lines in arrival order.
                    // This project writes headers from a flat map; embedding explicit CRLF
                    // keeps separate Set-Cookie header lines on re-serialization.
                    existing_value.push_str("\r\nset-cookie: ");
                    existing_value.push_str(&normalized_value);
                }
                Some(existing_value) => {
                    // Most repeated headers are list-compatible; keep both values.
                    existing_value.push_str(", ");
                    existing_value.push_str(&normalized_value);
                }
                None => {
                    headers.insert(normalized_key, normalized_value);
                }
            }
        }
    }

    headers
}

async fn read_line_bytes(
    tls_stream: &mut TlsStream<&mut TcpStream>,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let mut line = Vec::new();
    let mut byte = [0u8; 1];

    loop {
        let n = tls_stream.read(&mut byte).await?;
        if n == 0 {
            if line.is_empty() {
                return Err("Connection closed while reading line".into());
            }
            break;
        }

        line.push(byte[0]);
        if byte[0] == b'\n' {
            break;
        }

        if line.len() > 8 * 1024 {
            return Err("Line too large while parsing stream".into());
        }
    }

    Ok(line)
}

pub async fn read_http_stream(
    tls_stream: &mut TlsStream<&mut TcpStream>,
) -> Result<HttpsRequest, Box<dyn std::error::Error + Send + Sync>> {
    let buffer_string = read_headers_buffer(tls_stream).await?;

    let lines = buffer_string.split("\r\n").collect::<Vec<&str>>();

    // Parse request line
    let first_line = *lines.first().unwrap_or(&"");
    let [method, authority, version] = first_line.split(' ').collect::<Vec<&str>>()[..] else {
        tracing::error!("Malformed HTTPS request line: {}", first_line);
        return Err("Malformed HTTPS request line".into());
    };

    // Parse headers
    let header_lines = lines
        .iter()
        .skip(1)
        .take_while(|&&line| !line.is_empty())
        .cloned()
        .collect::<Vec<&str>>();

    let headers = parse_headers(header_lines.as_ref());

    // Parse body based on Content-Length or Transfer-Encoding.
    let body = if let Some(content_length) = headers.get("content-length") {
        let length: usize = content_length.parse().unwrap_or(0);

        if length > 0 {
            let mut body_buffer = vec![0u8; length];
            tls_stream.read_exact(&mut body_buffer).await?;
            Some(body_buffer)
        } else {
            None
        }
    } else if headers
        .get("transfer-encoding")
        .map(|e| e.to_lowercase().contains("chunked"))
        .unwrap_or(false)
    {
        let mut body_data = Vec::new();

        loop {
            let mut chunk_size_line = read_line_bytes(tls_stream).await?;

            while chunk_size_line.last() == Some(&b'\n') || chunk_size_line.last() == Some(&b'\r') {
                chunk_size_line.pop();
            }

            let size_str = String::from_utf8_lossy(&chunk_size_line);
            let size_hex = size_str.split(';').next().unwrap_or("").trim();
            let chunk_size = usize::from_str_radix(size_hex, 16)
                .map_err(|e| format!("Invalid request chunk size hex '{}': {}", size_hex, e))?;

            if chunk_size == 0 {
                loop {
                    let trailer_line = read_line_bytes(tls_stream).await?;
                    if trailer_line == b"\r\n" || trailer_line == b"\n" {
                        break;
                    }
                }
                break;
            }

            let mut chunk_data = vec![0u8; chunk_size];
            tls_stream.read_exact(&mut chunk_data).await?;
            body_data.extend_from_slice(&chunk_data);

            let mut trailing = [0u8; 2];
            tls_stream.read_exact(&mut trailing).await?;
        }

        Some(body_data)
    } else {
        None
    };

    Ok(HttpsRequest {
        method: method.to_string(),
        version: version.to_string(),
        uri: authority.to_string(),
        headers,
        body: body,
    })
}

pub async fn read_stream_response(
    tls_stream: &mut TlsStream<&mut TcpStream>,
) -> Result<HttpsResponse, Box<dyn std::error::Error + Send + Sync>> {
    let headers_raw = read_headers_buffer(tls_stream).await?;
    let lines = headers_raw
        .split("\r\n")
        .filter(|line| !line.is_empty())
        .collect::<Vec<&str>>();

    let status_line = lines.first().copied().unwrap_or_default();
    let parts: Vec<&str> = status_line.trim().splitn(3, ' ').collect();
    if parts.len() < 3 {
        return Err("Malformed HTTP response line".into());
    }
    let (version, status_code, status_text) = (parts[0], parts[1], parts[2]);

    let header_lines = lines.iter().skip(1).copied().collect::<Vec<&str>>();
    let headers = parse_headers(header_lines.as_ref());

    // Read body based on Content-Length or Transfer-Encoding
    let body = if let Some(content_length) = headers.get("content-length") {
        let length: usize = content_length.parse()?;
        if length > 0 {
            let mut body_buffer = vec![0u8; length];
            tls_stream.read_exact(&mut body_buffer).await?;
            Some(body_buffer)
        } else {
            None
        }
    } else if headers
        .get("transfer-encoding")
        .map(|e| e.to_lowercase().contains("chunked"))
        .unwrap_or(false)
    {
        let mut body_data = Vec::new();

        loop {
            let mut chunk_size_line = read_line_bytes(tls_stream).await?;

            // Remove \r\n or \n
            while chunk_size_line.last() == Some(&b'\n') || chunk_size_line.last() == Some(&b'\r') {
                chunk_size_line.pop();
            }

            let size_str = String::from_utf8_lossy(&chunk_size_line);
            let size_hex = size_str.split(';').next().unwrap_or("").trim();

            tracing::trace!(
                "Chunk size line: {:?}, parsed hex: '{}'",
                size_str,
                size_hex
            );

            let chunk_size = usize::from_str_radix(size_hex, 16)
                .map_err(|e| format!("Invalid chunk size hex '{}': {}", size_hex, e))?;

            if chunk_size == 0 {
                // Read trailers until empty line
                loop {
                    let trailer_line = read_line_bytes(tls_stream).await?;
                    if trailer_line == b"\r\n" || trailer_line == b"\n" {
                        break;
                    }
                }
                break;
            }

            // Read chunk data
            let mut chunk_data = vec![0u8; chunk_size];
            tls_stream.read_exact(&mut chunk_data).await?;
            body_data.extend_from_slice(&chunk_data);

            // Read trailing CRLF after chunk
            let mut trailing = [0u8; 2];
            tls_stream.read_exact(&mut trailing).await?;

            tracing::trace!("Read chunk of {} bytes", chunk_size);
        }

        tracing::debug!("Finished chunked body: {} bytes", body_data.len());
        Some(body_data)
    } else {
        None
    };

    Ok(HttpsResponse {
        version: version.to_string(),
        status_code: status_code.parse()?,
        status_text: status_text.to_string(),
        headers,
        body,
    })
}

pub async fn write_request(
    tls_stream: &mut TlsStream<&mut TcpStream>,
    request: &HttpsRequest,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut modified_headers = request.headers.clone();
    if let Some(body) = &request.body {
        if modified_headers
            .get("transfer-encoding")
            .map(|v| v.to_lowercase().contains("chunked"))
            .unwrap_or(false)
        {
            modified_headers.remove("transfer-encoding");
            modified_headers.insert("content-length".to_string(), body.len().to_string());
        }
    }

    let mut request_string = format!("{} {} {}\r\n", request.method, request.uri, request.version);

    for (key, value) in &modified_headers {
        request_string.push_str(&format!("{}: {}\r\n", key, value));
    }

    request_string.push_str("\r\n");

    tls_stream.write_all(request_string.as_bytes()).await?;

    if let Some(body) = &request.body {
        tls_stream.write_all(body).await?;
    }

    tls_stream.flush().await?;

    Ok(())
}

pub async fn write_response(
    tls_stream: &mut TlsStream<&mut TcpStream>,
    response: &HttpsResponse,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut response_string = format!(
        "{} {} {}\r\n",
        response.version, response.status_code, response.status_text
    );

    // Clone headers and modify for proper Content-Length
    let mut modified_headers = response.headers.clone();

    if let Some(body) = &response.body {
        // Remove Transfer-Encoding if present (we already decoded chunks)
        tracing::trace!(
            "Modifying response headers for Content-Length. Original headers: {:?}",
            modified_headers
        );

        modified_headers.remove("transfer-encoding");

        // Set correct Content-Length
        modified_headers.insert("content-length".to_string(), body.len().to_string());
    }

    for (key, value) in &modified_headers {
        response_string.push_str(&format!("{}: {}\r\n", key, value));
    }

    response_string.push_str("\r\n");

    tls_stream.write_all(response_string.as_bytes()).await?;
    if let Some(body) = &response.body {
        tls_stream.write_all(body).await?;
    }

    tls_stream.flush().await?;

    Ok(())
}
