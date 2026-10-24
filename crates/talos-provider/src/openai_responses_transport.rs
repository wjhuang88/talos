//! HTTP/SSE transport for the OpenAI Responses wire family.
//!
//! The semantic event mapping lives in `openai_responses_sse`; this module is responsible only for
//! byte-stream timeouts, incremental UTF-8 decoding, SSE record framing, JSON decoding, and clean
//! EOF handoff. Keeping these layers separate makes transport corruption distinguishable from a
//! valid Responses terminal event.

use std::time::Duration;

use futures_util::StreamExt;
use serde_json::Value;
use talos_core::message::AgentEvent;
use tokio::sync::mpsc;

use crate::openai_responses_sse::{ResponsesStreamOutput, ResponsesStreamState};
use crate::stream_utf8::Utf8StreamDecoder;

#[derive(Debug, Default)]
pub(crate) struct ResponsesSseDecoder {
    utf8: Utf8StreamDecoder,
    buffer: String,
    state: ResponsesStreamState,
    failed: bool,
}

impl ResponsesSseDecoder {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            state: ResponsesStreamState::new(),
            ..Self::default()
        }
    }

    /// Push one arbitrary transport chunk.
    ///
    /// Chunks may split UTF-8 code points, SSE lines, blank-line record delimiters, or JSON tokens.
    /// Complete SSE records are decoded immediately; an incomplete suffix is retained.
    pub(crate) fn push(&mut self, bytes: &[u8]) -> Vec<ResponsesStreamOutput> {
        if self.failed {
            return Vec::new();
        }
        let decoded = match self.utf8.push(bytes) {
            Ok(decoded) => decoded,
            Err(_) => return self.fail("OpenAI Responses stream contained invalid UTF-8"),
        };
        self.buffer.push_str(&decoded);
        self.drain_complete_records()
    }

    /// Finish decoding after transport EOF.
    pub(crate) fn finish(&mut self) -> Vec<ResponsesStreamOutput> {
        if self.failed {
            return Vec::new();
        }
        if self.utf8.finish().is_err() {
            return self.fail("OpenAI Responses stream ended inside a UTF-8 code point");
        }

        let mut outputs = self.drain_complete_records();
        if self.failed {
            return outputs;
        }

        if !self.buffer.trim().is_empty() {
            let tail = std::mem::take(&mut self.buffer);
            outputs.extend(self.decode_record(&tail));
        }
        if !self.failed {
            outputs.extend(self.state.finish_eof());
        }
        outputs
    }

    fn drain_complete_records(&mut self) -> Vec<ResponsesStreamOutput> {
        let mut outputs = Vec::new();
        while let Some((index, delimiter_len)) = record_delimiter(&self.buffer) {
            let record = self.buffer[..index].to_string();
            self.buffer.drain(..index + delimiter_len);
            outputs.extend(self.decode_record(&record));
            if self.failed {
                break;
            }
        }
        outputs
    }

    fn decode_record(&mut self, record: &str) -> Vec<ResponsesStreamOutput> {
        let mut data_lines = Vec::new();
        for line in record.lines() {
            if line.starts_with(':') {
                continue;
            }
            if let Some(data) = line.strip_prefix("data:") {
                data_lines.push(data.strip_prefix(' ').unwrap_or(data));
            }
        }

        if data_lines.is_empty() {
            return Vec::new();
        }
        let data = data_lines.join("\n");
        if data.trim() == "[DONE]" {
            // Responses success/failure is carried by semantic terminal events. `[DONE]` is only a
            // framing sentinel and must not manufacture a successful TurnEnd.
            return Vec::new();
        }

        let event: Value = match serde_json::from_str(&data) {
            Ok(event) => event,
            Err(_) => {
                return self.fail("OpenAI Responses stream contained invalid JSON event data");
            }
        };
        self.state.apply(&event)
    }

    fn fail(&mut self, message: impl Into<String>) -> Vec<ResponsesStreamOutput> {
        if self.failed {
            return Vec::new();
        }
        self.failed = true;
        vec![ResponsesStreamOutput::Agent(AgentEvent::Error {
            message: message.into(),
        })]
    }
}

/// Parse one live Responses HTTP body with first-packet and idle timeouts.
///
/// Semantic outputs stay typed until the provider adapter projects native calls through the shared
/// invocation-integrity path and forwards ordinary [`AgentEvent`] values.
pub(crate) async fn parse_response_stream(
    response: reqwest::Response,
    tx: mpsc::Sender<ResponsesStreamOutput>,
    first_packet_timeout: Duration,
    idle_timeout: Duration,
) {
    let mut bytes = response.bytes_stream();
    let mut decoder = ResponsesSseDecoder::new();
    let mut first_packet = true;

    loop {
        let timeout = if first_packet {
            first_packet_timeout
        } else {
            idle_timeout
        };
        let next = tokio::time::timeout(timeout, bytes.next()).await;
        match next {
            Err(_) => {
                let phase = if first_packet {
                    "first packet"
                } else {
                    "stream idle"
                };
                let _ = tx
                    .send(ResponsesStreamOutput::Agent(AgentEvent::Error {
                        message: format!("OpenAI Responses {phase} timeout"),
                    }))
                    .await;
                return;
            }
            Ok(Some(Err(_))) => {
                let _ = tx
                    .send(ResponsesStreamOutput::Agent(AgentEvent::Error {
                        message: "OpenAI Responses network stream error".into(),
                    }))
                    .await;
                return;
            }
            Ok(Some(Ok(chunk))) => {
                first_packet = false;
                for output in decoder.push(&chunk) {
                    if tx.send(output).await.is_err() {
                        return;
                    }
                }
            }
            Ok(None) => {
                for output in decoder.finish() {
                    if tx.send(output).await.is_err() {
                        return;
                    }
                }
                return;
            }
        }
    }
}

fn record_delimiter(buffer: &str) -> Option<(usize, usize)> {
    let lf = buffer.find("\n\n").map(|index| (index, 2));
    let crlf = buffer.find("\r\n\r\n").map(|index| (index, 4));
    match (lf, crlf) {
        (Some(left), Some(right)) => Some(if left.0 <= right.0 { left } else { right }),
        (Some(delimiter), None) | (None, Some(delimiter)) => Some(delimiter),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn agents(outputs: Vec<ResponsesStreamOutput>) -> Vec<AgentEvent> {
        outputs
            .into_iter()
            .filter_map(|output| match output {
                ResponsesStreamOutput::Agent(event) => Some(event),
                ResponsesStreamOutput::FunctionCall { .. } => None,
            })
            .collect()
    }

    #[test]
    fn reassembles_split_utf8_sse_and_semantic_completion() {
        let wire = concat!(
            "event: response.created\n",
            "data: {\"type\":\"response.created\"}\n\n",
            "event: response.output_text.delta\n",
            "data: {\"type\":\"response.output_text.delta\",\"delta\":\"你🙂\"}\n\n",
            "event: response.completed\n",
            "data: {\"type\":\"response.completed\",\"response\":{\"usage\":{\"input_tokens\":1,\"output_tokens\":2}}}\n\n"
        );
        let bytes = wire.as_bytes();
        let split = wire.find('你').expect("fixture contains multibyte text") + 1;
        let mut decoder = ResponsesSseDecoder::new();
        let mut outputs = decoder.push(&bytes[..split]);
        outputs.extend(decoder.push(&bytes[split..]));
        outputs.extend(decoder.finish());

        let events = agents(outputs);
        assert!(matches!(events.first(), Some(AgentEvent::TurnStart)));
        assert!(
            events
                .iter()
                .any(|event| matches!(event, AgentEvent::TextDelta { delta } if delta == "你🙂"))
        );
        assert!(events.iter().any(|event| matches!(
            event,
            AgentEvent::TurnEnd {
                stop_reason: talos_core::message::StopReason::EndTurn,
                ..
            }
        )));
    }

    #[test]
    fn accepts_crlf_and_multiline_data_records() {
        let mut decoder = ResponsesSseDecoder::new();
        let outputs = decoder.push(
            b"data: {\"type\":\"response.output_text.delta\",\r\ndata: \"delta\":\"hello\"}\r\n\r\n",
        );
        let events = agents(outputs);
        assert!(matches!(
            events.as_slice(),
            [AgentEvent::TextDelta { delta }] if delta == "hello"
        ));
    }

    #[test]
    fn invalid_json_is_terminal_and_does_not_turn_done_into_success() {
        let mut decoder = ResponsesSseDecoder::new();
        let first = agents(decoder.push(b"data: {not-json}\n\n"));
        assert!(matches!(first.as_slice(), [AgentEvent::Error { .. }]));
        assert!(
            decoder
                .push(b"data: {\"type\":\"response.completed\",\"response\":{}}\n\n")
                .is_empty()
        );
        assert!(decoder.finish().is_empty());
    }

    #[test]
    fn done_sentinel_without_semantic_terminal_still_fails_at_eof() {
        let mut decoder = ResponsesSseDecoder::new();
        assert!(decoder.push(b"data: [DONE]\n\n").is_empty());
        let events = agents(decoder.finish());
        assert!(matches!(
            events.as_slice(),
            [AgentEvent::Error { message }] if message.contains("before a terminal event")
        ));
    }

    #[test]
    fn final_unterminated_sse_record_is_processed_before_eof_check() {
        let mut decoder = ResponsesSseDecoder::new();
        let outputs = decoder.push(b"data: {\"type\":\"response.completed\",\"response\":{}}");
        assert!(outputs.is_empty());
        let events = agents(decoder.finish());
        assert!(matches!(events.as_slice(), [AgentEvent::TurnEnd { .. }]));
    }

    #[test]
    fn comments_and_event_lines_do_not_enter_json_payload() {
        let mut decoder = ResponsesSseDecoder::new();
        let outputs = decoder.push(
            b": keepalive\nevent: response.output_text.delta\ndata: {\"type\":\"response.output_text.delta\",\"delta\":\"ok\"}\n\n",
        );
        let events = agents(outputs);
        assert!(matches!(
            events.as_slice(),
            [AgentEvent::TextDelta { delta }] if delta == "ok"
        ));
    }
}
