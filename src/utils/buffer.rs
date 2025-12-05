use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};

pub async fn read_headers_buffer<S>(
    stream: &mut S,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>>
where
    S: AsyncRead + Unpin,
{
    let mut reader = BufReader::new(stream);
    let mut buffer = String::new();
    let mut line_count = 0u16;

    loop {
        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line).await?;

        if bytes_read == 0 {
            if line_count == 0 {
                return Err("Connection closed before any data received".into());
            }
            return Err("Connection closed before complete headers".into());
        }

        line_count += 1;
        buffer.push_str(&line);

        if line == "\r\n" {
            tracing::trace!("Found end of headers (CRLF)");
            break;
        }

        if line == "\n" {
            tracing::trace!("Found end of headers (LF only)");
            break;
        }

        // Don't allow too many header lines
        if line_count > 100 {
            return Err("Too many header lines (possible attack)".into());
        }
    }

    tracing::trace!("Read {} lines, {} bytes total", line_count, buffer.len());
    Ok(buffer)
}
