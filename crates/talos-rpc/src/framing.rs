//! NDJSON framing for JSON-RPC messages over stdio.

use anyhow::{Context, Result};
use serde::Serialize;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// Reads one NDJSON line from an async reader.
pub async fn read_line<R: tokio::io::AsyncRead + Unpin>(
    reader: &mut BufReader<R>,
) -> Result<Option<String>> {
    let mut line = String::new();
    let bytes = reader
        .read_line(&mut line)
        .await
        .context("failed to read from RPC input")?;
    if bytes == 0 {
        return Ok(None);
    }
    while line.ends_with('\n') || line.ends_with('\r') {
        line.pop();
    }
    Ok(Some(line))
}

/// Writes one NDJSON-encoded JSON value to an async writer.
pub async fn write_json_line<W: tokio::io::AsyncWrite + Unpin, T: Serialize>(
    writer: &mut W,
    message: &T,
) -> Result<()> {
    let encoded = serde_json::to_string(message).context("failed to encode RPC output")?;
    writer
        .write_all(encoded.as_bytes())
        .await
        .context("failed to write RPC output")?;
    writer
        .write_all(b"\n")
        .await
        .context("failed to write RPC newline")?;
    writer.flush().await.context("failed to flush RPC output")?;
    Ok(())
}
