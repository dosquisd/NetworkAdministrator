use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, BufReader};

pub async fn read_first_line_buffer(
    buffer: &[u8],
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut reader = BufReader::new(buffer);
    let mut first_line = String::new();

    let bytes_read = reader.read_line(&mut first_line).await?;

    if bytes_read == 0 {
        return Err("No data received to read the first line".into());
    }

    Ok(first_line.trim_end_matches(&['\r', '\n'][..]).to_string())
}

pub fn parse_first_line_buffer(
    buffer: String,
) -> Result<(String, String, String), Box<dyn std::error::Error + Send + Sync>> {
    let [method, authority, version] = buffer.split(' ').collect::<Vec<&str>>()[..] else {
        return Err("Malformed HTTPS request line".into());
    };

    Ok((
        method.to_string(),
        authority.to_string(),
        version.to_string(),
    ))
}

pub async fn read_headers_buffer<S>(
    stream: &mut S,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>>
where
    S: AsyncRead + Unpin,
{
    let mut raw = Vec::with_capacity(1024);
    let mut byte = [0u8; 1];

    loop {
        let n = stream.read(&mut byte).await?;
        if n == 0 {
            if raw.is_empty() {
                return Err("Connection closed before any data received".into());
            }
            return Err("Connection closed before complete headers".into());
        }

        raw.push(byte[0]);

        let len = raw.len();
        let found_crlf = len >= 4 && &raw[len - 4..] == b"\r\n\r\n";
        let found_lf = len >= 2 && &raw[len - 2..] == b"\n\n";
        if found_crlf || found_lf {
            tracing::trace!("Found end of headers");
            break;
        }

        // Header size guard to avoid abuse.
        if raw.len() > 64 * 1024 {
            return Err("Headers too large (possible attack)".into());
        }
    }

    let buffer = String::from_utf8_lossy(&raw).to_string();
    let line_count = buffer.lines().count();
    tracing::trace!("Read {} lines, {} bytes total", line_count, buffer.len());
    Ok(buffer)
}
