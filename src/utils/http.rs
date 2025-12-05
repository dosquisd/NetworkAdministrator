use std::collections::HashMap;

use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};
use tokio_native_tls::TlsStream;

use super::buffer::read_headers_buffer;

#[derive(Clone, Debug)]
pub struct HttpRequest {
    pub method: String,
    pub version: String,
    pub uri: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

#[derive(Clone, Debug)]
pub struct HttpResponse {
    pub version: String,
    pub status_code: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

fn parse_headers(lines: &[&str]) -> HashMap<String, String> {
    let mut headers = HashMap::new();

    // Maybe there's a more efficient way to do this, idk
    for line in lines {
        if let Some((key, value)) = line.split_once(": ") {
            headers.insert(key.to_string(), value.to_string());
        }
    }

    headers
}

pub async fn read_http_stream(
    tls_stream: &mut TlsStream<&mut TcpStream>,
) -> Result<HttpRequest, Box<dyn std::error::Error + Send + Sync>> {
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

    // Parse body as String
    // Check if there's a body based on Content-Length
    let body = if let Some(content_length) = headers.get("Content-Length") {
        let length: usize = content_length.parse().unwrap_or(0);

        if length > 0 {
            let mut body_buffer = vec![0u8; length];
            tls_stream.read_exact(&mut body_buffer).await?;
            Some(String::from_utf8_lossy(&body_buffer).to_string())
        } else {
            None
        }
    } else {
        None
    };

    Ok(HttpRequest {
        method: method.to_string(),
        version: version.to_string(),
        uri: authority.to_string(),
        headers,
        body: body,
    })
}

pub async fn read_stream_response(
    tls_stream: &mut TlsStream<&mut TcpStream>,
) -> Result<HttpResponse, Box<dyn std::error::Error + Send + Sync>> {
    let mut reader = BufReader::new(tls_stream);

    // Read status line
    let mut status_line = String::new();
    reader.read_line(&mut status_line).await?;

    let parts: Vec<&str> = status_line.trim().splitn(3, ' ').collect();
    if parts.len() < 3 {
        return Err("Malformed HTTP response line".into());
    }
    let (version, status_code, status_text) = (parts[0], parts[1], parts[2]);

    // Read headers
    let mut headers = HashMap::new();
    loop {
        let mut header_line = String::new();
        reader.read_line(&mut header_line).await?;

        if header_line == "\r\n" || header_line == "\n" {
            break;
        }

        if let Some((key, value)) = header_line.trim().split_once(": ") {
            headers.insert(key.to_string(), value.to_string());
        }
    }

    // Read body based on Content-Length or Transfer-Encoding
    let body = if let Some(content_length) = headers.get("Content-Length") {
        let length: usize = content_length.parse()?;
        if length > 0 {
            let mut body_buffer = vec![0u8; length];
            reader.read_exact(&mut body_buffer).await?;
            Some(body_buffer)
        } else {
            None
        }
    } else if headers
        .get("Transfer-Encoding")
        .map(|e| e.to_lowercase().contains("chunked"))
        .unwrap_or(false)
    {
        let mut body_data = Vec::new();

        loop {
            // Read chunk size line usando read_until
            let mut chunk_size_line = Vec::new();
            reader.read_until(b'\n', &mut chunk_size_line).await?;

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
                    let mut trailer_line = Vec::new();
                    reader.read_until(b'\n', &mut trailer_line).await?;
                    if trailer_line == b"\r\n" || trailer_line == b"\n" {
                        break;
                    }
                }
                break;
            }

            // Read chunk data
            let mut chunk_data = vec![0u8; chunk_size];
            reader.read_exact(&mut chunk_data).await?;
            body_data.extend_from_slice(&chunk_data);

            // Read trailing CRLF after chunk
            let mut trailing = [0u8; 2];
            reader.read_exact(&mut trailing).await?;

            tracing::trace!("Read chunk of {} bytes", chunk_size);
        }

        tracing::debug!("Finished chunked body: {} bytes", body_data.len());
        Some(body_data)
    } else {
        None
    };

    Ok(HttpResponse {
        version: version.to_string(),
        status_code: status_code.parse()?,
        status_text: status_text.to_string(),
        headers,
        body,
    })
}

pub async fn write_request(
    tls_stream: &mut TlsStream<&mut TcpStream>,
    request: &HttpRequest,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut request_string = format!("{} {} {}\r\n", request.method, request.uri, request.version);

    for (key, value) in &request.headers {
        request_string.push_str(&format!("{}: {}\r\n", key, value));
    }

    request_string.push_str("\r\n");

    if let Some(body) = &request.body {
        request_string.push_str(body);
    }

    tls_stream.write_all(request_string.as_bytes()).await?;

    Ok(())
}

pub async fn write_response(
    tls_stream: &mut TlsStream<&mut TcpStream>,
    response: &HttpResponse,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut response_string = format!(
        "{} {} {}\r\n",
        response.version, response.status_code, response.status_text
    );

    // Clone headers and modify for proper Content-Length
    let mut modified_headers = response.headers.clone();

    if let Some(body) = &response.body {
        // Remove Transfer-Encoding if present (we already decoded chunks)
        modified_headers.remove("Transfer-Encoding");
        modified_headers.remove("Content-Encoding");

        // Set correct Content-Length
        modified_headers.insert("Content-Length".to_string(), body.len().to_string());
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
