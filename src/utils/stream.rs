use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

use crate::utils::buffer::read_headers_buffer;

#[derive(Clone, Debug)]
pub struct StreamParser {
    pub buffer: String,
    pub header_lines: Vec<String>,
    pub method: String,
    pub authority: String,
    pub version: String,
}

async fn read_and_consume_stream(
    stream: &mut TcpStream,
    all_buffer: bool,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    match all_buffer {
        true => {
            let mut buffer = vec![0u8];
            stream.read_to_end(&mut buffer).await?;
            let buffer_string = String::from_utf8_lossy(&buffer).to_string();
            Ok(buffer_string)
        }
        false => read_headers_buffer(stream).await,
    }
}

async fn read_without_consuming_stream(
    stream: &mut TcpStream,
    _all_buffer: bool,  // Unused for now
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut buffer = Vec::new();
    let mut peek_buffer = vec![0u8; 4096];

    loop {
        // Peek without consuming
        let n = stream.peek(&mut peek_buffer).await?;

        if n == 0 {
            return Err("Connection closed".into());
        }

        // Find \r\n\r\n in peeked data
        if let Some(pos) = peek_buffer[..n]
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
        {
            let headers_len = pos + 4;

            // Now actually read only what we need
            let mut actual_buffer = vec![0u8; headers_len];
            stream.read_exact(&mut actual_buffer).await?;

            return Ok(String::from_utf8_lossy(&actual_buffer).to_string());
        }

        // If not found, read what we peeked and continue
        let mut chunk = vec![0u8; n];
        stream.read_exact(&mut chunk).await?;
        buffer.extend_from_slice(&chunk);

        if buffer.len() > 16 * 1024 {
            return Err("Headers too large".into());
        }
    }
}

pub async fn read_stream(
    stream: &mut TcpStream,
    consume_stream: bool,
    all_buffer: bool,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    match consume_stream {
        true => read_and_consume_stream(stream, all_buffer).await,
        false => read_without_consuming_stream(stream, all_buffer).await,
    }
}

// TODO: Optimize this function to allow reading only the first line without consuming the entire stream
// returning the same output
pub async fn parse_stream(
    stream: &mut TcpStream,
    consume_stream: bool,
    all_buffer: bool
) -> Result<StreamParser, Box<dyn std::error::Error + Send + Sync>> {
    let buffer_string = read_stream(stream, consume_stream, all_buffer).await?;

    tracing::trace!("Full received data: {:?}", buffer_string);

    let lines = buffer_string.split("\r\n").collect::<Vec<&str>>();
    let first_line = *lines.first().unwrap_or(&"");
    let header_lines = lines
        .iter()
        .skip(1)
        .take_while(|&&line| !line.is_empty())
        .cloned()
        .collect::<Vec<&str>>();

    tracing::trace!("Parsed header lines: {:?}", header_lines);

    let [method, authority, version] = first_line.split(' ').collect::<Vec<&str>>()[..] else {
        tracing::error!("Malformed HTTPS request line: {}", first_line);
        return Err("Malformed HTTPS request line".into());
    };

    Ok(StreamParser {
        buffer: buffer_string.clone(),
        header_lines: header_lines.iter().map(|s| s.to_string()).collect(),
        method: method.to_string(),
        authority: authority.to_string(),
        version: version.to_string(),
    })
}
