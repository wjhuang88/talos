//! Recorded wire contracts exercised against the production transport.

use crate::openai_responses_reasoning;

#[test]
fn synthetic_parallel_wire_preserves_each_call_across_byte_splits() {
    use crate::openai_responses_sse::ResponsesStreamOutput;
    use talos_core::message::{AgentEvent, StopReason};
    let wire = include_bytes!("../../tests/fixtures/responses_parallel.synthetic.sse");
    for size in [1, 7, wire.len()] {
        let mut decoder = super::ResponsesSseDecoder::new();
        let mut outputs = Vec::new();
        for chunk in wire.chunks(size) {
            outputs.extend(decoder.push(chunk));
        }
        outputs.extend(decoder.finish());
        let calls: Vec<_> = outputs
            .iter()
            .filter_map(|o| match o {
                ResponsesStreamOutput::FunctionCall {
                    id,
                    name,
                    arguments,
                } => Some((id.as_str(), name.as_str(), arguments.as_str())),
                _ => None,
            })
            .collect();
        assert_eq!(
            calls,
            vec![
                ("call_1", "probe_1", "{ \"q\" : \"second\" }"),
                ("call_0", "probe_0", "{ \"q\" : \"汉字\" }")
            ]
        );
        assert!(
            matches!(outputs.last(), Some(ResponsesStreamOutput::Agent(AgentEvent::TurnEnd { stop_reason: StopReason::ToolUse, usage })) if usage.input_tokens == 12 && usage.output_tokens == 7 && usage.cache_read_tokens == 3 && usage.reasoning_tokens == 2)
        );
        assert!(
            !outputs
                .iter()
                .any(|o| matches!(o, ResponsesStreamOutput::Agent(AgentEvent::Error { .. })))
        );
    }
}

#[test]
fn upstream_sdk_recorded_turns_preserve_tools_reasoning_and_usage() {
    use crate::openai_responses_sse::ResponsesStreamOutput;
    use talos_core::message::{AgentEvent, ReasoningBlock, StopReason};
    let fixture = include_str!(
        "../../tests/fixtures/vercel-ai/openai-reasoning-encrypted-content.1.chunks.txt"
    );
    let mut wire = String::new();
    let mut expected_summary = String::new();
    let mut expected_text = String::new();
    let mut visible_bytes = 0;
    let mut turns = 0;
    let mut calls = 0;
    let mut reasoning = 0;
    for line in fixture.lines().filter(|line| !line.trim().is_empty()) {
        let event: serde_json::Value = serde_json::from_str(line).expect("upstream JSON event");
        match event["type"].as_str() {
            Some("response.reasoning_summary_text.delta") => {
                expected_summary.push_str(event["delta"].as_str().expect("summary delta"))
            }
            Some("response.output_text.delta") => {
                expected_text.push_str(event["delta"].as_str().expect("text delta"))
            }
            _ => {}
        }
        wire.push_str(&format!("data: {line}\n\n"));
        if event["type"] != "response.completed" {
            continue;
        }
        let mut decoder = super::ResponsesSseDecoder::new();
        let mut outputs = Vec::new();
        for chunk in wire.as_bytes().chunks(7) {
            outputs.extend(decoder.push(chunk));
        }
        outputs.extend(decoder.finish());
        assert!(!outputs.iter().any(|output| matches!(
            output,
            ResponsesStreamOutput::Agent(AgentEvent::Error { .. })
        )));
        let actual_summary: String = outputs
            .iter()
            .filter_map(|output| match output {
                ResponsesStreamOutput::Agent(AgentEvent::ThinkingDelta { delta }) => {
                    Some(delta.as_str())
                }
                _ => None,
            })
            .collect();
        let actual_text: String = outputs
            .iter()
            .filter_map(|output| match output {
                ResponsesStreamOutput::Agent(AgentEvent::TextDelta { delta }) => {
                    Some(delta.as_str())
                }
                _ => None,
            })
            .collect();
        assert!(
            actual_summary == expected_summary,
            "recorded summary projection mismatch"
        );
        assert!(
            actual_text == expected_text,
            "recorded text projection mismatch"
        );
        visible_bytes += actual_summary.len() + actual_text.len();
        expected_summary.clear();
        expected_text.clear();
        let turn_calls: Vec<_> = outputs
            .iter()
            .filter_map(|output| match output {
                ResponsesStreamOutput::FunctionCall {
                    id,
                    name,
                    arguments,
                } => Some((id, name, arguments)),
                _ => None,
            })
            .collect();
        assert_eq!(turn_calls.len(), usize::from(turns < 3));
        for (id, name, arguments) in turn_calls {
            assert!(!id.is_empty());
            assert_eq!(name, "calculator");
            assert!(serde_json::from_str::<serde_json::Value>(arguments).is_ok());
            calls += 1;
        }
        for output in &outputs {
            if let ResponsesStreamOutput::Agent(AgentEvent::ReasoningComplete { blocks }) = output {
                for block in blocks {
                    if let ReasoningBlock::Redacted { data } = block {
                        let item = openai_responses_reasoning::decode_reasoning_item(data)
                            .expect("opaque replay envelope");
                        assert!(item["encrypted_content"].is_string());
                        reasoning += 1;
                    }
                }
            }
        }
        assert!(
            matches!(outputs.last(), Some(ResponsesStreamOutput::Agent(AgentEvent::TurnEnd { stop_reason, usage })) if *stop_reason == if turns < 3 { StopReason::ToolUse } else { StopReason::EndTurn } && usage.input_tokens == [134,221,260,299][turns] && usage.output_tokens == [28,26,26,12][turns])
        );
        turns += 1;
        wire.clear();
    }
    assert!(wire.is_empty());
    assert_eq!(turns, 4);
    assert_eq!(calls, 3);
    assert!(reasoning > 0);
    assert!(visible_bytes > 0);
}

#[test]
fn upstream_sdk_recorded_error_never_becomes_success() {
    use crate::openai_responses_sse::ResponsesStreamOutput;
    use talos_core::message::AgentEvent;
    let fixture = include_str!("../../tests/fixtures/vercel-ai/openai-error.1.chunks.txt");
    let wire: String = fixture
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| format!("data: {line}\n\n"))
        .collect();
    let mut decoder = super::ResponsesSseDecoder::new();
    let mut outputs = Vec::new();
    for chunk in wire.as_bytes().chunks(1) {
        outputs.extend(decoder.push(chunk));
    }
    outputs.extend(decoder.finish());
    assert_eq!(
        outputs
            .iter()
            .filter(|output| matches!(
                output,
                ResponsesStreamOutput::Agent(AgentEvent::Error { .. })
            ))
            .count(),
        1
    );
    assert!(!outputs.iter().any(|output| matches!(
        output,
        ResponsesStreamOutput::Agent(AgentEvent::TurnEnd { .. })
    )));
}
