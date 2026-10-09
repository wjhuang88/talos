//! Original argument extraction for protected compatibility calls.
//! Envelope deserialization rejects duplicate routing keys before they can change tool identity.

use serde::Deserialize;
use talos_core::tool::BrowserRawArguments;

/// Release only prose after the complete compatibility batch has passed validation.
pub(crate) fn outside_tool_blocks(text: &str) -> Result<String, &'static str> {
    let mut remaining = text;
    let mut output = String::new();
    while let Some((start, open, close)) = [
        ("```json-tool", "```"),
        ("<tool_call>", "</tool_call>"),
        ("<toolcall>", "</toolcall>"),
    ]
    .into_iter()
    .filter_map(|(open, close)| remaining.find(open).map(|start| (start, open, close)))
    .min_by_key(|(start, _, _)| *start)
    {
        output.push_str(&remaining[..start]);
        let inner = &remaining[start + open.len()..];
        let end = inner
            .find(close)
            .ok_or("unterminated compatibility tool block")?;
        remaining = &inner[end + close.len()..];
    }
    output.push_str(remaining);
    Ok(output)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope<'a> {
    name: &'a str,
    #[serde(borrow)]
    args: &'a serde_json::value::RawValue,
    #[serde(default)]
    id: Option<&'a str>,
}

pub(crate) fn extract(
    content: &str,
    expected_name: &str,
) -> Result<BrowserRawArguments, &'static str> {
    let content = content.trim();
    let (name, raw) = if content.starts_with('{') {
        let envelope: Envelope<'_> = serde_json::from_str(content)
            .map_err(|_| "invalid protected compatibility envelope")?;
        // Parsing id explicitly rejects duplicate, non-string routing metadata as well.
        let _ = envelope.id;
        (envelope.name, envelope.args.get())
    } else {
        let split = content
            .find(char::is_whitespace)
            .ok_or("invalid protected compatibility envelope")?;
        (&content[..split], content[split..].trim())
    };
    if name != expected_name {
        return Err("ambiguous protected compatibility tool identity");
    }
    BrowserRawArguments::parse_original(raw).map_err(|_| "invalid original browser arguments")
}

#[cfg(test)]
mod tests {
    use super::extract;

    #[tokio::test]
    async fn protected_text_sse_preserves_evidence_without_leaking_argument_text() {
        use crate::openai_sse::InvocationSender;
        use std::time::Duration;
        use talos_core::{
            message::AgentEvent, provider::ProviderInvocationEvent, tool::ToolProtocol,
        };

        for anthropic in [false, true] {
            for protocol in [ToolProtocol::Compat, ToolProtocol::TalosStrict] {
                for malformed in [false, true] {
                    let args = if malformed {
                        r#"{"text":"private-sentinel","te\u0078t":"duplicate"}"#
                    } else {
                        r#"{"text":"private-sentinel"}"#
                    };
                    let text = format!(
                        "<tool_call>{{\"name\":\"browser\",\"id\":\"b\",\"args\":{args}}}</tool_call>safe prose"
                    );
                    let mut body = String::new();
                    // Split every character, including tag names and escaped JSON keys.
                    for character in text.chars() {
                        let chunk = if anthropic {
                            serde_json::json!({"index":0,"delta":{"type":"text_delta","text":character.to_string()}})
                        } else {
                            serde_json::json!({"choices":[{"index":0,"delta":{"content":character.to_string()},"finish_reason":null}]})
                        };
                        if anthropic {
                            body.push_str("event: content_block_delta\n");
                        }
                        body.push_str(&format!("data: {chunk}\n\n"));
                    }
                    if anthropic {
                        body.push_str("event: message_delta\ndata: {\"delta\":{\"stop_reason\":\"tool_use\"},\"usage\":{\"output_tokens\":1}}\n\n");
                    } else {
                        body.push_str("data: [DONE]\n\n");
                    }
                    let mut server = mockito::Server::new_async().await;
                    let mock = server
                        .mock("GET", "/stream")
                        .with_status(200)
                        .with_header("content-type", "text/event-stream")
                        .with_body(body)
                        .create_async()
                        .await;
                    let response = reqwest::get(format!("{}/stream", server.url()))
                        .await
                        .expect("response");
                    let (tx, mut rx) = tokio::sync::mpsc::channel(16);
                    let sender = InvocationSender::Original {
                        tx,
                        browser_tools: vec!["browser".into()],
                    };
                    if anthropic {
                        crate::anthropic_stream::parse_sse_stream_with_integrity(
                            response,
                            sender,
                            Duration::from_secs(2),
                            Duration::from_secs(2),
                            protocol,
                        )
                        .await;
                    } else {
                        crate::openai_sse::parse_sse_stream_with_integrity(
                            response,
                            sender,
                            Duration::from_secs(2),
                            Duration::from_secs(2),
                            protocol,
                        )
                        .await;
                    }
                    let (mut calls, mut errors, mut endings) = (0, 0, 0);
                    let mut prose = String::new();
                    while let Some(event) = rx.recv().await {
                        match event {
                            ProviderInvocationEvent::BrowserToolCall { arguments, .. } => {
                                calls += 1;
                                assert_eq!(arguments.fields()["text"], "private-sentinel");
                            }
                            ProviderInvocationEvent::Legacy(event) => {
                                assert!(!format!("{event:?}").contains("private-sentinel"));
                                match event {
                                    AgentEvent::TextDelta { delta } => prose.push_str(&delta),
                                    AgentEvent::Error { .. } => errors += 1,
                                    AgentEvent::TurnEnd { .. } => endings += 1,
                                    AgentEvent::ToolCall { .. } => panic!("lost raw evidence"),
                                    _ => {}
                                }
                            }
                        }
                    }
                    assert_eq!(calls, usize::from(!malformed));
                    assert_eq!(errors, usize::from(malformed));
                    assert_eq!(endings, usize::from(!malformed));
                    assert_eq!(prose, if malformed { "" } else { "safe prose" });
                    mock.assert_async().await;
                }
            }
        }
    }

    #[test]
    fn protected_compat_batches_preserve_order_and_fail_atomically() {
        use crate::{anthropic_stream::parse_invocation_text_calls, openai_sse::InvocationSender};
        use talos_core::{provider::ProviderInvocationEvent, tool::ToolProtocol};
        let (tx, _rx) = tokio::sync::mpsc::channel(4);
        let sender = InvocationSender::Original {
            tx,
            browser_tools: vec!["browser".into()],
        };
        let browser =
            r#"<tool_call>{"name":"browser","id":"b","args":{"text":"original"}}</tool_call>"#;
        let ordinary = r#"<toolcall>{"name":"read","args":{"path":"note"}}</toolcall>"#;
        let events = parse_invocation_text_calls(
            &format!("{ordinary}{browser}"),
            ToolProtocol::Compat,
            &sender,
        )
        .expect("mixed batch");
        assert!(matches!(&events[0], ProviderInvocationEvent::Legacy(_)));
        assert!(
            matches!(&events[1], ProviderInvocationEvent::BrowserToolCall { id, arguments, .. }
            if id == "b" && arguments.fields()["text"] == "original")
        );
        assert!(matches!(
            parse_invocation_text_calls(browser, ToolProtocol::TalosStrict, &sender)
                .expect("strict")
                .first(),
            Some(ProviderInvocationEvent::BrowserToolCall { .. })
        ));
        for malformed in [
            r#"<tool_call>browser {"text":"a","te\u0078t":"b"}</tool_call>"#,
            r#"<tool_call>{"name":"browser","args":{},"args":{}}</tool_call>"#,
        ] {
            assert!(
                parse_invocation_text_calls(
                    &format!("{ordinary}{malformed}"),
                    ToolProtocol::Compat,
                    &sender
                )
                .is_err()
            );
        }
    }

    #[test]
    fn original_arguments_preserve_duplicates_in_both_compatibility_forms() {
        for raw in [
            r#"browser {"text":"first","te\u0078t":"second"}"#,
            r#"{"name":"browser","args":{"text":"first","text":"second"}}"#,
            r#"{"name":"other","name":"browser","args":{}}"#,
            r#"{"name":"browser","args":{},"args":{}}"#,
            r#"{"name":"browser","args":{},"id":"a","id":"b"}"#,
            r#"{"name":"browser","args":{},"extra":true}"#,
            r#"other {}"#,
        ] {
            assert!(extract(raw, "browser").is_err(), "{raw}");
        }
        for raw in [
            r#"browser {"text":"original"}"#,
            r#"{"name":"browser","args":{"text":"original"},"id":"call"}"#,
        ] {
            let arguments = extract(raw, "browser").expect("original carrier");
            assert_eq!(arguments.fields()["text"], "original");
        }
    }
}
