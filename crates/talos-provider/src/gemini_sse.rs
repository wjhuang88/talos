//! Bounded Gemini SSE framing; no tool invocation is released before validated EOF.

use crate::gemini_stream::{GeminiOutput, GeminiStream};
use talos_core::provider::{ProviderError, ProviderResult};

const MAX_STREAM: usize = 8 * 1024 * 1024;
const MAX_RECORD: usize = 1024 * 1024;

pub(crate) struct GeminiSse {
    state: GeminiStream,
    line: Vec<u8>,
    data: String,
    total: usize,
    skip_lf: bool,
    failed: bool,
    first_line: bool,
    pending_tools: Vec<GeminiOutput>,
    received_event: bool,
}

fn invalid(message: &'static str) -> ProviderError {
    ProviderError::InvalidResponse(message.into())
}

impl GeminiSse {
    pub(crate) fn new() -> Self {
        Self {
            state: GeminiStream::new(),
            line: Vec::new(),
            data: String::new(),
            total: 0,
            skip_lf: false,
            failed: false,
            first_line: true,
            pending_tools: Vec::new(),
            received_event: false,
        }
    }

    pub(crate) fn push(&mut self, bytes: &[u8]) -> ProviderResult<Vec<GeminiOutput>> {
        if self.failed {
            return Err(invalid("Gemini SSE already closed"));
        }
        let result = self.push_bytes(bytes);
        if result.is_err() {
            self.failed = true;
            self.pending_tools.clear();
        }
        result
    }

    fn push_bytes(&mut self, bytes: &[u8]) -> ProviderResult<Vec<GeminiOutput>> {
        self.total = self
            .total
            .checked_add(bytes.len())
            .filter(|n| *n <= MAX_STREAM)
            .ok_or_else(|| invalid("Gemini SSE stream exceeds limit"))?;
        let mut output = Vec::new();
        for byte in bytes {
            if self.skip_lf {
                self.skip_lf = false;
                if *byte == b'\n' {
                    continue;
                }
            }
            if *byte == b'\r' || *byte == b'\n' {
                self.skip_lf = *byte == b'\r';
                output.extend(self.complete_line()?);
            } else {
                if self.line.len() >= MAX_RECORD {
                    return Err(invalid("Gemini SSE line exceeds limit"));
                }
                self.line.push(*byte);
            }
        }
        Ok(output)
    }

    fn complete_line(&mut self) -> ProviderResult<Vec<GeminiOutput>> {
        let raw = std::mem::take(&mut self.line);
        let mut line =
            std::str::from_utf8(&raw).map_err(|_| invalid("invalid Gemini SSE UTF-8"))?;
        if self.first_line {
            self.first_line = false;
            line = line.strip_prefix('\u{feff}').unwrap_or(line);
        }
        if line.is_empty() {
            if self.data.is_empty() {
                return Ok(Vec::new());
            }
            let data = std::mem::take(&mut self.data);
            let value = serde_json::from_str(data.trim_end_matches('\n'))
                .map_err(|_| invalid("invalid Gemini SSE JSON"))?;
            let events = self.state.apply(&value)?;
            self.received_event = true;
            let mut visible = Vec::new();
            for event in events {
                if matches!(event, GeminiOutput::FunctionCall { .. }) {
                    self.pending_tools.push(event);
                } else {
                    visible.push(event);
                }
            }
            return Ok(visible);
        }
        if line.starts_with(':') {
            return Ok(Vec::new());
        }
        let (field, value) = line.split_once(':').unwrap_or((line, ""));
        if field == "data" {
            let value = value.strip_prefix(' ').unwrap_or(value);
            if self
                .data
                .len()
                .saturating_add(value.len())
                .saturating_add(1)
                > MAX_RECORD
            {
                return Err(invalid("Gemini SSE record exceeds limit"));
            }
            self.data.push_str(value);
            self.data.push('\n');
        }
        Ok(Vec::new())
    }

    pub(crate) fn finish(&mut self) -> ProviderResult<Vec<GeminiOutput>> {
        if self.failed {
            return Err(invalid("Gemini SSE already closed"));
        }
        self.failed = true;
        if !self.line.is_empty() || !self.data.is_empty() {
            self.pending_tools.clear();
            return Err(invalid("Gemini SSE ended inside a record"));
        }
        match self.state.finish_eof() {
            Ok(end) => {
                let mut output = std::mem::take(&mut self.pending_tools);
                output.push(GeminiOutput::Event(end));
                Ok(output)
            }
            Err(error) => {
                self.pending_tools.clear();
                Err(error)
            }
        }
    }
}

/// Read within dispatch-owned total budget plus first-byte/idle limits. Dropping this future
/// cancels the read; no reader task or credential refresh is detached.
pub(crate) async fn read_response(
    response: reqwest::Response,
    tx: &tokio::sync::mpsc::Sender<GeminiOutput>,
    deadline: std::time::Instant,
    first_packet: std::time::Duration,
    idle: std::time::Duration,
) -> ProviderResult<()> {
    use futures_util::StreamExt;
    let mut decoder = GeminiSse::new();
    let mut stream = response.bytes_stream();
    let first_deadline = std::time::Instant::now()
        .checked_add(first_packet)
        .unwrap_or(deadline)
        .min(deadline);
    loop {
        let read_deadline = if decoder.received_event {
            std::time::Instant::now()
                .checked_add(idle)
                .unwrap_or(deadline)
                .min(deadline)
        } else {
            first_deadline
        };
        let next = tokio::time::timeout_at(read_deadline.into(), stream.next())
            .await
            .map_err(|_| invalid("Gemini SSE read timed out"))?;
        let events = match next {
            Some(Ok(bytes)) => decoder.push(&bytes)?,
            Some(Err(_)) => {
                return Err(ProviderError::NetworkError("Gemini SSE read failed".into()));
            }
            None => {
                for event in decoder.finish()? {
                    tokio::time::timeout_at(deadline.into(), tx.send(event))
                        .await
                        .map_err(|_| invalid("Gemini SSE delivery timed out"))?
                        .map_err(|_| invalid("Gemini SSE receiver closed"))?;
                }
                return Ok(());
            }
        };
        for event in events {
            tokio::time::timeout_at(deadline.into(), tx.send(event))
                .await
                .map_err(|_| invalid("Gemini SSE delivery timed out"))?
                .map_err(|_| invalid("Gemini SSE receiver closed"))?;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_core::message::AgentEvent;
    const TEXT: &str = "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"你🙂\"}]},\"finishReason\":\"STOP\"}]}\r\n\r\n";

    fn decode(chunks: &[&[u8]]) -> String {
        let mut decoder = GeminiSse::new();
        let mut text = String::new();
        for chunk in chunks {
            for event in decoder.push(chunk).expect("chunk") {
                if let GeminiOutput::Event(AgentEvent::TextDelta { delta }) = event {
                    text.push_str(&delta);
                }
            }
        }
        assert!(matches!(
            decoder.finish().expect("EOF").last(),
            Some(GeminiOutput::Event(AgentEvent::TurnEnd { .. }))
        ));
        text
    }

    #[tokio::test]
    async fn heartbeat_does_not_reset_first_event_deadline() {
        use std::time::{Duration, Instant};
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("listener");
        let address = listener.local_addr().expect("address");
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("accept");
            let mut header = Vec::new();
            let mut byte = [0];
            while !header.ends_with(b"\r\n\r\n") {
                socket.read_exact(&mut byte).await.expect("request");
                header.push(byte[0]);
            }
            socket.write_all(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nContent-Type: text/event-stream\r\n\r\n").await.expect("headers");
            for _ in 0..10 {
                if socket.write_all(b"3\r\n:\n\n\r\n").await.is_err() {
                    return;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });
        let response = reqwest::Client::builder()
            .no_proxy()
            .build()
            .expect("client")
            .get(format!("http://{address}/"))
            .send()
            .await
            .expect("response");
        let (tx, _rx) = tokio::sync::mpsc::channel(4);
        let error = read_response(
            response,
            &tx,
            Instant::now() + Duration::from_secs(2),
            Duration::from_millis(25),
            Duration::from_secs(1),
        )
        .await
        .expect_err("first event timeout");
        assert!(error.to_string().contains("timed out"));
        server.abort();
    }

    #[test]
    fn every_utf8_and_crlf_boundary_preserves_text() {
        for split in 0..=TEXT.len() {
            assert_eq!(
                decode(&[&TEXT.as_bytes()[..split], &TEXT.as_bytes()[split..]]),
                "你🙂"
            );
        }
        assert_eq!(decode(&[TEXT.replace("\r\n", "\r").as_bytes()]), "你🙂");
    }

    proptest::proptest! {
        #[test]
        fn arbitrary_transport_chunk_sizes_preserve_records(sizes in proptest::collection::vec(1usize..40, 1..50)) {
            let mut chunks = Vec::new(); let mut offset = 0; let mut i = 0;
            while offset < TEXT.len() {
                let end = (offset + sizes[i % sizes.len()]).min(TEXT.len());
                chunks.push(&TEXT.as_bytes()[offset..end]); offset = end; i += 1;
            }
            proptest::prop_assert_eq!(decode(&chunks), "你🙂");
        }
    }

    #[test]
    fn tools_are_held_until_successful_eof_and_dropped_on_late_error() {
        let tool = b"data: {\"candidates\":[{\"content\":{\"parts\":[{\"functionCall\":{\"name\":\"ping\"}}]}}]}\n\n";
        let mut decoder = GeminiSse::new();
        assert!(decoder.push(tool).expect("tool").is_empty());
        assert!(decoder.finish().is_err());
        let mut decoder = GeminiSse::new();
        assert!(decoder.push(tool).expect("tool").is_empty());
        assert!(
            decoder
                .push(b"data: {\"error\":{\"message\":\"private\"}}\n\n")
                .is_err()
        );
        assert!(decoder.finish().is_err());
        let mut decoder = GeminiSse::new();
        decoder.push(tool).expect("tool");
        decoder
            .push(b"data: {\"candidates\":[{\"finishReason\":\"STOP\"}]}\n\n")
            .expect("stop");
        assert!(matches!(
            decoder.finish().expect("end")[0],
            GeminiOutput::FunctionCall { .. }
        ));
    }

    #[test]
    fn malformed_truncated_and_oversized_records_are_terminal_errors() {
        for wire in [
            b"data: invalid\n\n".as_slice(),
            b"data: \xff\n\n",
            b"data: {}\n",
            b"data: {}",
        ] {
            let mut decoder = GeminiSse::new();
            if decoder.push(wire).is_ok() {
                assert!(decoder.finish().is_err());
            } else {
                assert!(decoder.finish().is_err());
            }
        }
        assert!(GeminiSse::new().push(&vec![b'x'; MAX_RECORD + 1]).is_err());
        assert!(GeminiSse::new().push(&vec![b'x'; MAX_STREAM + 1]).is_err());
    }
}
