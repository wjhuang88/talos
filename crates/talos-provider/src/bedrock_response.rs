//! Pull-based reading of Bedrock ConverseStream. Dropping the reader releases the HTTP body.
use crate::bedrock_frames::FrameReader;
use aws_smithy_types::event_stream::Message;
use serde_json::Value;
use talos_core::provider::{ProviderError, ProviderResult};

fn invalid(text: &'static str) -> ProviderError {
    ProviderError::InvalidResponse(text.into())
}

fn header<'a>(message: &'a Message, key: &str) -> ProviderResult<&'a str> {
    let mut values = message
        .headers()
        .iter()
        .filter(|h| h.name().as_str() == key);
    let value = values
        .next()
        .and_then(|h| h.value().as_string().ok())
        .ok_or_else(|| invalid("missing or invalid Bedrock event header"))?;
    if values.next().is_some() {
        return Err(invalid("duplicate Bedrock event header"));
    }
    Ok(value.as_str())
}

fn event(message: Message) -> ProviderResult<(String, Value)> {
    match header(&message, ":message-type")? {
        "event" => {
            let name = header(&message, ":event-type")?;
            if header(&message, ":content-type")? != "application/json" {
                return Err(invalid("unsupported Bedrock event content type"));
            }
            let value = serde_json::from_slice(message.payload())
                .map_err(|_| invalid("invalid Bedrock event JSON"))?;
            Ok((name.to_owned(), value))
        }
        "exception" | "error" => Err(invalid("Bedrock returned an event-stream service error")),
        _ => Err(invalid("unknown Bedrock event-stream message type")),
    }
}

pub(crate) struct ResponseReader {
    response: reqwest::Response,
    chunk: bytes::Bytes,
    frames: Option<FrameReader>,
}

impl ResponseReader {
    pub(crate) fn new(response: reqwest::Response) -> ProviderResult<Self> {
        if !response.status().is_success() {
            return Err(ProviderError::InvalidResponse(format!(
                "Bedrock HTTP {}",
                response.status().as_u16()
            )));
        }
        let mime = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(';').next())
            .map(str::trim);
        if !mime.is_some_and(|v| v.eq_ignore_ascii_case("application/vnd.amazon.eventstream")) {
            return Err(invalid("unexpected Bedrock response content type"));
        }
        Ok(Self {
            response,
            chunk: Default::default(),
            frames: Some(FrameReader::default()),
        })
    }

    pub(crate) async fn next(&mut self) -> ProviderResult<Option<(String, Value)>> {
        while let Some(frames) = self.frames.as_mut() {
            if self.chunk.is_empty() {
                match self
                    .response
                    .chunk()
                    .await
                    .map_err(|_| invalid("Bedrock response read failed"))?
                {
                    Some(chunk) => self.chunk = chunk,
                    None => {
                        self.frames.take().unwrap().finish()?;
                        return Ok(None);
                    }
                }
            }
            let mut input = self.chunk.as_ref();
            let result = frames.next(&mut input)?;
            self.chunk = self.chunk.slice(self.chunk.len() - input.len()..);
            if let Some(message) = result {
                return event(message).map(Some);
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_smithy_eventstream::frame::write_message_to;
    use aws_smithy_types::event_stream::{Header, HeaderValue};

    fn string_header(key: &'static str, value: &'static str) -> Header {
        Header::new(key, HeaderValue::String(value.into()))
    }

    #[tokio::test]
    async fn reads_events_and_surfaces_trailing_exception_over_http() {
        let stop = Message::new(r#"{"stopReason":"end_turn"}"#)
            .add_header(string_header(":message-type", "event"))
            .add_header(string_header(":event-type", "messageStop"))
            .add_header(string_header(":content-type", "application/json"));
        let exception = Message::new(r#"{"message":"private detail"}"#)
            .add_header(string_header(":message-type", "exception"));
        let mut wire = Vec::new();
        write_message_to(&stop, &mut wire).unwrap();
        write_message_to(&exception, &mut wire).unwrap();
        let mut server = mockito::Server::new_async().await;
        let fixture = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/vnd.amazon.eventstream")
            .with_body(wire)
            .create_async()
            .await;
        let response = reqwest::Client::new()
            .post(server.url())
            .send()
            .await
            .unwrap();
        let mut reader = ResponseReader::new(response).unwrap();
        let (name, value) = reader.next().await.unwrap().unwrap();
        assert_eq!(name, "messageStop");
        assert_eq!(value["stopReason"], "end_turn");
        let error = reader.next().await.unwrap_err().to_string();
        assert!(error.contains("service error"));
        assert!(!error.contains("private detail"));
        fixture.assert_async().await;
    }
}
