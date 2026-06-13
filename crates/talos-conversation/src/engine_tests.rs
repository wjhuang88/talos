use futures::StreamExt;
use talos_core::message::{AgentEvent, StopReason, ToolCall, ToolResult, Usage};
use talos_core::tool::ToolProvenance;

use crate::engine::ConversationEngine;
use crate::types::{
    ChatMessage, MessageRole, MessageSource, MessageStatus, PluginObservation, TipKind, UiOutput,
};

fn new_engine() -> ConversationEngine {
    ConversationEngine::new("claude-sonnet-4".to_string())
}

fn make_tool_call(name: &str, _provenance: ToolProvenance) -> ToolCall {
    ToolCall {
        id: "tc-1".to_string(),
        name: name.to_string(),
        input: serde_json::json!({}),
    }
}

fn make_tool_result(content: &str, is_error: bool) -> ToolResult {
    ToolResult {
        tool_use_id: "tc-1".to_string(),
        content: content.to_string(),
        is_error,
    }
}

/// Extract the first `UiOutput::Stream` from outputs and collect its content.
async fn collect_stream(outputs: Vec<UiOutput>) -> Option<(MessageSource, String)> {
    for output in outputs {
        if let UiOutput::Stream(msg) = output {
            let source = msg.source.clone();
            let chunks: Vec<String> = msg.stream.collect().await;
            return Some((source, chunks.join("")));
        }
    }
    None
}

/// Extract the first `UiOutput::Status` from outputs.
fn find_status(outputs: &[UiOutput]) -> Option<&crate::types::StatusSnapshot> {
    outputs.iter().find_map(|o| match o {
        UiOutput::Status(s) => Some(s),
        _ => None,
    })
}

// ---------------------------------------------------------------------------
// handle_agent_event: TurnStart
// ---------------------------------------------------------------------------

#[test]
fn turn_start_creates_stream_and_status() {
    let mut engine = new_engine();
    engine.current_turn_text = "leftover".to_string();
    engine.is_processing = false;

    let outputs = engine.handle_agent_event(&AgentEvent::TurnStart);

    assert_eq!(outputs.len(), 2);
    assert!(matches!(&outputs[0], UiOutput::Stream(msg) if msg.source == MessageSource::Assistant));
    assert!(matches!(&outputs[1], UiOutput::Status(_)));

    assert!(engine.current_turn_text.is_empty());
    assert!(engine.is_processing);
}

// ---------------------------------------------------------------------------
// handle_agent_event: TextDelta
// ---------------------------------------------------------------------------

#[tokio::test]
async fn text_delta_sends_chunks_to_stream() {
    let mut engine = new_engine();
    let outputs = engine.handle_agent_event(&AgentEvent::TurnStart);

    let stream_msg = outputs.into_iter().find_map(|o| match o {
        UiOutput::Stream(m) => Some(m),
        _ => None,
    });
    assert!(stream_msg.is_some());
    let stream_msg = stream_msg.unwrap();

    let outputs = engine.handle_agent_event(&AgentEvent::TextDelta {
        delta: "Hello".to_string(),
    });
    assert_eq!(outputs.len(), 1);
    assert!(find_status(&outputs).is_some());
    assert_eq!(engine.current_turn_text, "Hello");

    let outputs = engine.handle_agent_event(&AgentEvent::TextDelta {
        delta: " World".to_string(),
    });
    assert_eq!(outputs.len(), 1);
    assert_eq!(engine.current_turn_text, "Hello World");

    engine.handle_agent_event(&AgentEvent::TurnEnd {
        stop_reason: StopReason::EndTurn,
        usage: Usage::default(),
    });

    let chunks: Vec<String> = stream_msg.stream.collect().await;
    let text: String = chunks.join("");
    assert_eq!(text, "Hello World");
}

#[tokio::test]
async fn text_delta_accumulates_multiline() {
    let mut engine = new_engine();
    let outputs = engine.handle_agent_event(&AgentEvent::TurnStart);
    let stream_msg = outputs
        .into_iter()
        .find_map(|o| match o {
            UiOutput::Stream(m) => Some(m),
            _ => None,
        })
        .unwrap();

    engine.handle_agent_event(&AgentEvent::TextDelta {
        delta: "line1\nline2\n".to_string(),
    });
    assert_eq!(engine.current_turn_text, "line1\nline2\n");

    engine.handle_agent_event(&AgentEvent::TextDelta {
        delta: "line3".to_string(),
    });
    assert_eq!(engine.current_turn_text, "line1\nline2\nline3");

    engine.handle_agent_event(&AgentEvent::TurnEnd {
        stop_reason: StopReason::EndTurn,
        usage: Usage::default(),
    });

    let chunks: Vec<String> = stream_msg.stream.collect().await;
    let text: String = chunks.join("");
    assert_eq!(text, "line1\nline2\nline3");
}

#[tokio::test]
async fn text_delta_empty_delta_noop() {
    let mut engine = new_engine();
    engine.handle_agent_event(&AgentEvent::TurnStart);

    let outputs = engine.handle_agent_event(&AgentEvent::TextDelta {
        delta: "".to_string(),
    });
    assert_eq!(outputs.len(), 1);
    assert!(find_status(&outputs).is_some());
    assert!(engine.current_turn_text.is_empty());
}

// ---------------------------------------------------------------------------
// handle_agent_event: ToolCall
// ---------------------------------------------------------------------------

#[tokio::test]
async fn tool_call_produces_stream_and_message() {
    let mut engine = new_engine();

    let outputs = engine.handle_agent_event(&AgentEvent::ToolCall {
        call: make_tool_call("bash", ToolProvenance::Native),
        provenance: ToolProvenance::Native,
    });

    assert_eq!(outputs.len(), 1);
    let (source, text) = collect_stream(outputs).await.unwrap();
    assert!(matches!(source, MessageSource::Tool { name } if name == "bash"));
    assert!(text.contains("bash"));
    assert!(text.contains("[native]"));

    assert_eq!(engine.messages.len(), 1);
    let msg = &engine.messages[0];
    assert_eq!(msg.role, MessageRole::Assistant);
    assert_eq!(msg.status, MessageStatus::Completed);
    assert!(msg.content.is_empty());
    let tc = msg.tool_call.as_ref().unwrap();
    assert_eq!(tc.tool_name, "bash");
    assert_eq!(tc.provenance, ToolProvenance::Native);
    assert!(tc.result.is_none());
}

#[tokio::test]
async fn tool_call_closes_previous_stream() {
    let mut engine = new_engine();

    let outputs = engine.handle_agent_event(&AgentEvent::TurnStart);
    let stream_msg = outputs
        .into_iter()
        .find_map(|o| match o {
            UiOutput::Stream(m) => Some(m),
            _ => None,
        })
        .unwrap();

    engine.handle_agent_event(&AgentEvent::TextDelta {
        delta: "partial".to_string(),
    });

    let outputs = engine.handle_agent_event(&AgentEvent::ToolCall {
        call: make_tool_call("read", ToolProvenance::Native),
        provenance: ToolProvenance::Native,
    });

    let chunks: Vec<String> = stream_msg.stream.collect().await;
    let text: String = chunks.join("");
    assert_eq!(text, "partial");

    assert_eq!(outputs.len(), 1);
    let (source, _) = collect_stream(outputs).await.unwrap();
    assert!(matches!(source, MessageSource::Tool { .. }));
}

// ---------------------------------------------------------------------------
// handle_agent_event: ToolResult
// ---------------------------------------------------------------------------

#[tokio::test]
async fn tool_result_produces_stream_and_updates_message() {
    let mut engine = new_engine();

    engine.handle_agent_event(&AgentEvent::ToolCall {
        call: make_tool_call("read_file", ToolProvenance::Native),
        provenance: ToolProvenance::Native,
    });

    let outputs = engine.handle_agent_event(&AgentEvent::ToolResult {
        result: make_tool_result("file contents", false),
    });

    assert_eq!(outputs.len(), 1);
    let (source, text) = collect_stream(outputs).await.unwrap();
    assert!(matches!(source, MessageSource::Tool { name } if name.is_empty()));
    assert!(text.contains("✓"));
    assert!(text.contains("file contents"));

    let msg = &engine.messages[0];
    let tc = msg.tool_call.as_ref().unwrap();
    let result = tc.result.as_ref().unwrap();
    assert_eq!(result.content, "file contents");
    assert!(!result.is_error);
}

#[tokio::test]
async fn tool_result_error_flag_propagates() {
    let mut engine = new_engine();

    engine.handle_agent_event(&AgentEvent::ToolCall {
        call: make_tool_call("bash", ToolProvenance::Native),
        provenance: ToolProvenance::Native,
    });

    let outputs = engine.handle_agent_event(&AgentEvent::ToolResult {
        result: make_tool_result("command not found", true),
    });

    let tc = engine.messages[0].tool_call.as_ref().unwrap();
    let result = tc.result.as_ref().unwrap();
    assert!(result.is_error);

    let (_, text) = collect_stream(outputs).await.unwrap();
    assert!(text.contains("✗"));
    assert!(text.contains("command not found"));
}

// ---------------------------------------------------------------------------
// handle_agent_event: TurnEnd
// ---------------------------------------------------------------------------

#[tokio::test]
async fn turn_end_finalizes_and_produces_status() {
    let mut engine = new_engine();
    engine.handle_agent_event(&AgentEvent::TurnStart);
    engine.handle_agent_event(&AgentEvent::TextDelta {
        delta: "Done.\n".to_string(),
    });

    let usage = Usage {
        input_tokens: 100,
        output_tokens: 50,
        ..Default::default()
    };
    let outputs = engine.handle_agent_event(&AgentEvent::TurnEnd {
        stop_reason: StopReason::EndTurn,
        usage: usage.clone(),
    });

    assert!(!engine.is_processing);
    assert_eq!(engine.messages.len(), 1);
    assert_eq!(engine.messages[0].role, MessageRole::Assistant);
    assert_eq!(engine.messages[0].content, "Done.\n");
    assert_eq!(engine.usage, usage);

    assert_eq!(outputs.len(), 1);
    let snapshot = find_status(&outputs).unwrap();
    assert_eq!(snapshot.model_name, "claude-sonnet-4");
    assert_eq!(snapshot.usage.input_tokens, 100);
    assert_eq!(snapshot.usage.output_tokens, 50);
    assert!(!snapshot.is_processing);
}

#[tokio::test]
async fn turn_end_with_empty_text_still_produces_status() {
    let mut engine = new_engine();
    engine.handle_agent_event(&AgentEvent::TurnStart);

    let usage = Usage::default();
    let outputs = engine.handle_agent_event(&AgentEvent::TurnEnd {
        stop_reason: StopReason::EndTurn,
        usage: usage.clone(),
    });

    assert!(!engine.is_processing);
    assert_eq!(engine.messages.len(), 0);
    assert_eq!(engine.usage, usage);

    assert_eq!(outputs.len(), 1);
    assert!(find_status(&outputs).is_some());
}

// ---------------------------------------------------------------------------
// handle_agent_event: Error
// ---------------------------------------------------------------------------

#[tokio::test]
async fn error_clears_turn_and_produces_stream_and_status() {
    let mut engine = new_engine();
    engine.handle_agent_event(&AgentEvent::TurnStart);
    engine.current_turn_text = "partial response".to_string();
    engine.is_processing = true;

    let outputs = engine.handle_agent_event(&AgentEvent::Error {
        message: "rate limited".to_string(),
    });

    assert!(!engine.is_processing);
    assert!(engine.current_turn_text.is_empty());
    assert_eq!(engine.messages.len(), 1);
    assert_eq!(engine.messages[0].role, MessageRole::Error);
    assert_eq!(engine.messages[0].content, "[Error] rate limited");

    assert_eq!(outputs.len(), 3);
    let tip = outputs.iter().find_map(|o| match o {
        UiOutput::Tip { kind, text } => Some((kind.clone(), text.clone())),
        _ => None,
    });
    assert_eq!(tip, Some((TipKind::Error, "rate limited".to_string())));
    let snapshot = find_status(&outputs).unwrap();
    assert!(!snapshot.is_processing);
    let (source, text) = collect_stream(outputs).await.unwrap();
    assert_eq!(source, MessageSource::Error);
    assert_eq!(text, "[Error] rate limited");
}

// ---------------------------------------------------------------------------
// handle_user_message
// ---------------------------------------------------------------------------

#[tokio::test]
async fn handle_user_message_creates_stream_with_prefix() {
    let mut engine = new_engine();

    let outputs = engine.handle_user_message("hello world");

    assert_eq!(engine.messages.len(), 1);
    let msg = &engine.messages[0];
    assert_eq!(msg.role, MessageRole::User);
    assert_eq!(msg.status, MessageStatus::Completed);
    assert_eq!(msg.content, "hello world\n");
    assert!(msg.tool_call.is_none());

    assert_eq!(outputs.len(), 1);
    let (source, text) = collect_stream(outputs).await.unwrap();
    assert_eq!(source, MessageSource::User);
    assert_eq!(text, "hello world");
}

#[tokio::test]
async fn handle_user_message_multiline_single_stream() {
    let mut engine = new_engine();

    let outputs = engine.handle_user_message("line1\nline2");

    assert_eq!(engine.messages.len(), 1);
    assert_eq!(engine.messages[0].content, "line1\nline2\n");

    let (_, text) = collect_stream(outputs).await.unwrap();
    assert_eq!(text, "line1\nline2");
}

// ---------------------------------------------------------------------------
// handle_slash_command: /help
// ---------------------------------------------------------------------------

#[tokio::test]
async fn slash_help_returns_all_commands() {
    let mut engine = new_engine();

    let outputs = engine.handle_slash_command("/help");

    assert_eq!(outputs.len(), 1);
    let (_, text) = collect_stream(outputs).await.unwrap();
    assert!(text.contains("/help"));
    assert!(text.contains("/quit"));
    assert!(text.contains("/status"));
    assert!(text.contains("/new"));
    assert!(text.contains("/plugins"));
    assert!(text.contains("/copy"));
    assert!(text.contains("/export"));
    assert!(text.contains("/mock-request"));
}

// ---------------------------------------------------------------------------
// handle_slash_command: /quit and /exit
// ---------------------------------------------------------------------------

#[test]
fn slash_quit_produces_exit() {
    let mut engine = new_engine();

    let outputs = engine.handle_slash_command("/quit");

    assert_eq!(outputs.len(), 1);
    match &outputs[0] {
        UiOutput::Exit => {}
        _ => panic!("expected Exit"),
    }
}

#[test]
fn slash_exit_produces_exit() {
    let mut engine = new_engine();

    let outputs = engine.handle_slash_command("/exit");

    assert_eq!(outputs.len(), 1);
    match &outputs[0] {
        UiOutput::Exit => {}
        _ => panic!("expected Exit"),
    }
}

// ---------------------------------------------------------------------------
// handle_slash_command: /status
// ---------------------------------------------------------------------------

#[tokio::test]
async fn slash_status_shows_model_and_tokens() {
    let mut engine = new_engine();
    engine.usage = Usage {
        input_tokens: 42,
        output_tokens: 17,
        ..Default::default()
    };

    let outputs = engine.handle_slash_command("/status");

    assert_eq!(outputs.len(), 1);
    let (_, text) = collect_stream(outputs).await.unwrap();
    assert!(text.contains("claude-sonnet-4"));
    assert!(text.contains("42"));
    assert!(text.contains("17"));
}

// ---------------------------------------------------------------------------
// handle_slash_command: /new
// ---------------------------------------------------------------------------

#[tokio::test]
async fn slash_new_clears_everything() {
    let mut engine = new_engine();

    engine.handle_user_message("hello");
    engine.current_turn_text = "partial".to_string();
    engine.usage = Usage {
        input_tokens: 100,
        output_tokens: 50,
        ..Default::default()
    };
    engine.branch_id = Some("branch-1".to_string());
    engine.plugin_observations.push(PluginObservation {
        key: "native".to_string(),
        count: 3,
    });
    engine.scrollback.scrolled_line_count = 10;

    let outputs = engine.handle_slash_command("/new");

    assert!(engine.messages.is_empty());
    assert!(engine.current_turn_text.is_empty());
    assert_eq!(engine.usage, Usage::default());
    assert!(engine.branch_id.is_none());
    assert!(engine.plugin_observations.is_empty());
    assert_eq!(engine.scrollback.scrolled_line_count, 0);

    assert_eq!(outputs.len(), 1);
    let (_, text) = collect_stream(outputs).await.unwrap();
    assert_eq!(text, "[System] New session started.\n");
}

// ---------------------------------------------------------------------------
// handle_slash_command: /plugins
// ---------------------------------------------------------------------------

#[tokio::test]
async fn slash_plugins_shows_observations() {
    let mut engine = new_engine();
    engine.plugin_observations.push(PluginObservation {
        key: "native".to_string(),
        count: 5,
    });
    engine.plugin_observations.push(PluginObservation {
        key: "mcp:github".to_string(),
        count: 2,
    });

    let outputs = engine.handle_slash_command("/plugins");

    assert_eq!(outputs.len(), 1);
    let (_, text) = collect_stream(outputs).await.unwrap();
    assert!(text.contains("Observed tool provenance"));
    assert!(text.contains("native") && text.contains("5 calls"));
    assert!(text.contains("mcp:github") && text.contains("2 calls"));
}

#[tokio::test]
async fn slash_plugins_empty_shows_no_tools_message() {
    let mut engine = new_engine();

    let outputs = engine.handle_slash_command("/plugins");

    assert_eq!(outputs.len(), 1);
    let (_, text) = collect_stream(outputs).await.unwrap();
    assert!(text.contains("No tool provenance observed"));
}

// ---------------------------------------------------------------------------
// handle_slash_command: unknown
// ---------------------------------------------------------------------------

#[tokio::test]
async fn unknown_command_returns_error_stream() {
    let mut engine = new_engine();

    let outputs = engine.handle_slash_command("/foobar");

    assert_eq!(outputs.len(), 1);
    let (source, text) = collect_stream(outputs).await.unwrap();
    assert_eq!(source, MessageSource::Error);
    assert!(text.contains("Unknown command"));
    assert!(text.contains("/foobar"));
}

#[test]
fn mock_request_is_model_passthrough_slash_command() {
    assert!(ConversationEngine::is_model_passthrough_slash_command(
        "/mock-request explain this code"
    ));
    assert!(ConversationEngine::is_model_passthrough_slash_command(
        "/mock-request\nexplain this code"
    ));
    assert!(ConversationEngine::is_model_passthrough_slash_command(
        "  /mock-request"
    ));
    assert!(!ConversationEngine::is_model_passthrough_slash_command(
        "/mock-requested"
    ));
    assert!(!ConversationEngine::is_model_passthrough_slash_command(
        "/help"
    ));
}

// ---------------------------------------------------------------------------
// drain_steering_queue
// ---------------------------------------------------------------------------

#[tokio::test]
async fn start_user_message_marks_processing_and_streams_user_input() {
    let mut engine = new_engine();

    let outputs = engine.start_user_message("hello");
    let (source, text) = collect_stream(outputs).await.unwrap();

    assert_eq!(source, MessageSource::User);
    assert_eq!(text, "hello");
    assert!(engine.is_processing());
    assert!(engine.status_snapshot().is_processing);
}

#[test]
fn enqueue_steering_records_queue_and_status() {
    let mut engine = new_engine();

    let outputs = engine.enqueue_steering("queued".to_string());

    assert_eq!(engine.drain_steering_queue(), Some("queued".to_string()));
    assert!(outputs.iter().any(|output| matches!(
        output,
        UiOutput::Tip {
            kind: TipKind::QueueHint,
            ..
        }
    )));
    assert!(outputs.iter().any(|output| matches!(
        output,
        UiOutput::Tip { text, .. } if text.contains("will send after current turn")
            && !text.contains("Esc")
    )));
    assert!(outputs.iter().any(|output| {
        matches!(
            output,
            UiOutput::Status(status) if status.steering_count == 1
        )
    }));
}

#[test]
fn cancel_turn_clears_processing_state() {
    let mut engine = new_engine();
    engine.handle_agent_event(&AgentEvent::TurnStart);
    engine.handle_agent_event(&AgentEvent::TextDelta {
        delta: "partial".to_string(),
    });

    let outputs = engine.cancel_turn();

    assert!(!engine.is_processing());
    assert!(engine.current_turn_text.is_empty());
    assert!(outputs.iter().any(|output| {
        matches!(
            output,
            UiOutput::Status(status) if !status.is_processing
        )
    }));
}

#[test]
fn drain_steering_queue_fifo_order() {
    let mut engine = new_engine();
    engine.steering_queue.push("first".to_string());
    engine.steering_queue.push("second".to_string());
    engine.steering_queue.push("third".to_string());

    assert_eq!(engine.drain_steering_queue(), Some("first".to_string()));
    assert_eq!(engine.drain_steering_queue(), Some("second".to_string()));
    assert_eq!(engine.drain_steering_queue(), Some("third".to_string()));
}

#[test]
fn drain_steering_queue_none_when_empty() {
    let mut engine = new_engine();
    assert_eq!(engine.drain_steering_queue(), None);

    engine.steering_queue.push("one".to_string());
    assert_eq!(engine.drain_steering_queue(), Some("one".to_string()));
    assert_eq!(engine.drain_steering_queue(), None);
}

// ---------------------------------------------------------------------------
// Plugin provenance
// ---------------------------------------------------------------------------

#[test]
fn provenance_native_key() {
    let mut engine = new_engine();

    engine.handle_agent_event(&AgentEvent::ToolCall {
        call: make_tool_call("bash", ToolProvenance::Native),
        provenance: ToolProvenance::Native,
    });

    assert_eq!(engine.plugin_observations.len(), 1);
    assert_eq!(engine.plugin_observations[0].key, "native");
    assert_eq!(engine.plugin_observations[0].count, 1);
}

#[test]
fn provenance_mcp_remote_key() {
    let mut engine = new_engine();

    engine.handle_agent_event(&AgentEvent::ToolCall {
        call: make_tool_call(
            "search",
            ToolProvenance::McpRemote {
                server: "github".to_string(),
            },
        ),
        provenance: ToolProvenance::McpRemote {
            server: "github".to_string(),
        },
    });

    assert_eq!(engine.plugin_observations.len(), 1);
    assert_eq!(engine.plugin_observations[0].key, "mcp:github");
    assert_eq!(engine.plugin_observations[0].count, 1);
}

#[test]
fn provenance_truncates_long_server_names() {
    let mut engine = new_engine();
    let long_name = "a".repeat(30); // 30 chars, > 24

    engine.handle_agent_event(&AgentEvent::ToolCall {
        call: make_tool_call(
            "tool",
            ToolProvenance::McpRemote {
                server: long_name.clone(),
            },
        ),
        provenance: ToolProvenance::McpRemote {
            server: long_name.clone(),
        },
    });

    assert_eq!(engine.plugin_observations.len(), 1);
    let key = &engine.plugin_observations[0].key;
    assert!(key.starts_with("mcp:"));
    // "mcp:" + 23 chars + "…" = 28 chars
    assert_eq!(key.chars().count(), 4 + 23 + 1); // "mcp:" + 23 + ellipsis
    assert!(key.ends_with('…'));
}

#[test]
fn provenance_increment_existing() {
    let mut engine = new_engine();

    engine.handle_agent_event(&AgentEvent::ToolCall {
        call: make_tool_call("bash", ToolProvenance::Native),
        provenance: ToolProvenance::Native,
    });
    engine.handle_agent_event(&AgentEvent::ToolCall {
        call: make_tool_call("bash", ToolProvenance::Native),
        provenance: ToolProvenance::Native,
    });
    engine.handle_agent_event(&AgentEvent::ToolCall {
        call: make_tool_call("bash", ToolProvenance::Native),
        provenance: ToolProvenance::Native,
    });

    assert_eq!(engine.plugin_observations.len(), 1);
    assert_eq!(engine.plugin_observations[0].count, 3);
}

#[test]
fn provenance_groups_mcp_servers() {
    let mut engine = new_engine();

    engine.handle_agent_event(&AgentEvent::ToolCall {
        call: make_tool_call(
            "search",
            ToolProvenance::McpRemote {
                server: "github".to_string(),
            },
        ),
        provenance: ToolProvenance::McpRemote {
            server: "github".to_string(),
        },
    });
    engine.handle_agent_event(&AgentEvent::ToolCall {
        call: make_tool_call(
            "list",
            ToolProvenance::McpRemote {
                server: "filesystem".to_string(),
            },
        ),
        provenance: ToolProvenance::McpRemote {
            server: "filesystem".to_string(),
        },
    });
    engine.handle_agent_event(&AgentEvent::ToolCall {
        call: make_tool_call(
            "search2",
            ToolProvenance::McpRemote {
                server: "github".to_string(),
            },
        ),
        provenance: ToolProvenance::McpRemote {
            server: "github".to_string(),
        },
    });

    assert_eq!(engine.plugin_observations.len(), 2);
    let github = engine
        .plugin_observations
        .iter()
        .find(|e| e.key == "mcp:github")
        .unwrap();
    let fs = engine
        .plugin_observations
        .iter()
        .find(|e| e.key == "mcp:filesystem")
        .unwrap();
    assert_eq!(github.count, 2);
    assert_eq!(fs.count, 1);
}

// ---------------------------------------------------------------------------
// Transcript tests
// ---------------------------------------------------------------------------

#[test]
fn last_assistant_text_returns_latest() {
    let mut engine = new_engine();

    assert!(engine.last_assistant_text().is_none());

    engine.handle_user_message("hello");
    assert!(engine.last_assistant_text().is_none());

    engine.messages.push(ChatMessage {
        role: MessageRole::Assistant,
        status: MessageStatus::Completed,
        content: "I can help!".to_string(),
        tool_call: None,
        created_at: std::time::Instant::now(),
    });
    assert_eq!(
        engine.last_assistant_text(),
        Some("I can help!".to_string())
    );

    engine.messages.push(ChatMessage {
        role: MessageRole::Assistant,
        status: MessageStatus::Completed,
        content: "Updated answer.".to_string(),
        tool_call: None,
        created_at: std::time::Instant::now(),
    });
    assert_eq!(
        engine.last_assistant_text(),
        Some("Updated answer.".to_string())
    );
}

#[test]
fn last_assistant_text_skips_tool_calls() {
    let mut engine = new_engine();

    engine.messages.push(ChatMessage {
        role: MessageRole::Assistant,
        status: MessageStatus::Completed,
        content: String::new(),
        tool_call: Some(crate::types::ToolCallInfo {
            tool_name: "bash".to_string(),
            arguments: "{}".to_string(),
            provenance: ToolProvenance::Native,
            result: None,
        }),
        created_at: std::time::Instant::now(),
    });
    assert!(engine.last_assistant_text().is_none());

    engine.messages.push(ChatMessage {
        role: MessageRole::Assistant,
        status: MessageStatus::Completed,
        content: "Here's the result.".to_string(),
        tool_call: None,
        created_at: std::time::Instant::now(),
    });
    assert_eq!(
        engine.last_assistant_text(),
        Some("Here's the result.".to_string())
    );
}

#[test]
fn transcript_plain_text_concatenates_messages() {
    let mut engine = new_engine();
    engine.handle_user_message("hello");
    engine.messages.push(ChatMessage {
        role: MessageRole::Assistant,
        status: MessageStatus::Completed,
        content: "Hi there!".to_string(),
        tool_call: None,
        created_at: std::time::Instant::now(),
    });

    let plain = engine.transcript_plain_text();
    assert!(plain.contains("hello"));
    assert!(plain.contains("Hi there!"));
}

#[test]
fn transcript_markdown_concatenates_messages() {
    let mut engine = new_engine();
    engine.handle_user_message("hello");
    engine.messages.push(ChatMessage {
        role: MessageRole::Assistant,
        status: MessageStatus::Completed,
        content: "Hi there!".to_string(),
        tool_call: None,
        created_at: std::time::Instant::now(),
    });

    let md = engine.transcript_markdown();
    assert!(md.contains("hello"));
    assert!(md.contains("Hi there!"));
}

#[tokio::test]
async fn transcript_plain_text_includes_tool_call_details() {
    let mut engine = new_engine();
    engine.handle_agent_event(&AgentEvent::ToolCall {
        call: make_tool_call("bash", ToolProvenance::Native),
        provenance: ToolProvenance::Native,
    });
    engine.handle_agent_event(&AgentEvent::ToolResult {
        result: make_tool_result("output", false),
    });

    let plain = engine.transcript_plain_text();
    assert!(plain.contains("bash"));
    assert!(plain.contains("[native]"));
    assert!(plain.contains("✓ output"));
}

#[tokio::test]
async fn transcript_markdown_includes_tool_call_details() {
    let mut engine = new_engine();
    engine.handle_agent_event(&AgentEvent::ToolCall {
        call: make_tool_call("bash", ToolProvenance::Native),
        provenance: ToolProvenance::Native,
    });
    engine.handle_agent_event(&AgentEvent::ToolResult {
        result: make_tool_result("output", false),
    });

    let md = engine.transcript_markdown();
    assert!(md.contains("bash"));
    assert!(md.contains("[native]"));
    assert!(md.contains("```json"));
    assert!(md.contains("**Result:**"));
    assert!(md.contains("output"));
}

// ---------------------------------------------------------------------------
// complete_slash_command
// ---------------------------------------------------------------------------

#[test]
fn complete_slash_command_empty_prefix_returns_all() {
    let engine = new_engine();

    let completions = engine.complete_slash_command("");

    assert_eq!(completions.len(), ConversationEngine::SLASH_COMMANDS.len());
}

#[test]
fn complete_slash_command_matches_prefix() {
    let engine = new_engine();

    let completions = engine.complete_slash_command("/h");

    assert_eq!(completions, vec!["/help"]);
}

#[test]
fn complete_slash_command_no_match_returns_empty() {
    let engine = new_engine();

    let completions = engine.complete_slash_command("/zzz");

    assert!(completions.is_empty());
}

#[test]
fn complete_slash_command_multiple_matches() {
    let engine = new_engine();

    let completions = engine.complete_slash_command("/c");

    assert!(completions.contains(&"/compact"));
    assert!(completions.contains(&"/copy"));
}

#[test]
fn complete_slash_command_includes_mock_request() {
    let engine = new_engine();

    let completions = engine.complete_slash_command("/mock");

    assert_eq!(completions, vec!["/mock-request"]);
}

// ---------------------------------------------------------------------------
// Status snapshot
// ---------------------------------------------------------------------------

#[test]
fn status_snapshot_reflects_current_state() {
    let mut engine = new_engine();
    engine.steering_queue.push("steer".to_string());
    engine.followup_queue.push("follow".to_string());
    engine.followup_queue.push("up".to_string());
    engine.is_processing = true;
    engine.branch_id = Some("b-123".to_string());

    let snapshot = engine.status_snapshot();

    assert_eq!(snapshot.model_name, "claude-sonnet-4");
    assert_eq!(snapshot.steering_count, 1);
    assert_eq!(snapshot.followup_count, 2);
    assert!(snapshot.is_processing);
    assert_eq!(snapshot.branch_id, Some("b-123".to_string()));
}

// ---------------------------------------------------------------------------
// Full turn integration
// ---------------------------------------------------------------------------

#[tokio::test]
async fn full_turn_lifecycle() {
    let mut engine = new_engine();

    let outputs = engine.handle_user_message("What is 2+2?");
    let (_, text) = collect_stream(outputs).await.unwrap();
    assert_eq!(text, "What is 2+2?");

    let outputs = engine.handle_agent_event(&AgentEvent::TurnStart);
    let assistant_stream = outputs
        .into_iter()
        .find_map(|o| match o {
            UiOutput::Stream(m) => Some(m),
            _ => None,
        })
        .unwrap();

    engine.handle_agent_event(&AgentEvent::TextDelta {
        delta: "2+2 equals 4.\n".to_string(),
    });

    let outputs = engine.handle_agent_event(&AgentEvent::ToolCall {
        call: make_tool_call("calculator", ToolProvenance::Native),
        provenance: ToolProvenance::Native,
    });
    let (_, tool_text) = collect_stream(outputs).await.unwrap();
    assert!(tool_text.contains("calculator"));

    let outputs = engine.handle_agent_event(&AgentEvent::ToolResult {
        result: make_tool_result("4", false),
    });
    let (_, result_text) = collect_stream(outputs).await.unwrap();
    assert!(result_text.contains("4"));

    let outputs = engine.handle_agent_event(&AgentEvent::TurnEnd {
        stop_reason: StopReason::EndTurn,
        usage: Usage {
            input_tokens: 200,
            output_tokens: 30,
            ..Default::default()
        },
    });

    assert!(!engine.is_processing);
    assert_eq!(engine.messages.len(), 3);

    let last = &engine.messages[engine.messages.len() - 1];
    assert_eq!(last.role, MessageRole::Assistant);
    assert_eq!(last.content, "2+2 equals 4.\n");

    assert!(outputs.iter().any(|o| matches!(o, UiOutput::Status(_))));

    let chunks: Vec<String> = assistant_stream.stream.collect().await;
    let text: String = chunks.join("");
    assert_eq!(text, "2+2 equals 4.\n");
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn turn_end_resets_scrolled_line_count() {
    let mut engine = new_engine();
    engine.handle_agent_event(&AgentEvent::TurnStart);
    engine.scrollback.scrolled_line_count = 5;

    engine.handle_agent_event(&AgentEvent::TurnEnd {
        stop_reason: StopReason::EndTurn,
        usage: Usage::default(),
    });

    assert_eq!(engine.scrollback.scrolled_line_count, 0);
}

#[test]
fn tool_call_records_provenance() {
    let mut engine = new_engine();

    engine.handle_agent_event(&AgentEvent::ToolCall {
        call: make_tool_call(
            "search",
            ToolProvenance::McpRemote {
                server: "github".to_string(),
            },
        ),
        provenance: ToolProvenance::McpRemote {
            server: "github".to_string(),
        },
    });

    assert_eq!(engine.plugin_observations.len(), 1);
    assert_eq!(engine.plugin_observations[0].key, "mcp:github");
}

#[test]
fn append_message_plain_adds_newline_if_missing() {
    let msg = ChatMessage {
        role: MessageRole::User,
        status: MessageStatus::Completed,
        content: "no trailing newline".to_string(),
        tool_call: None,
        created_at: std::time::Instant::now(),
    };

    let mut out = String::new();
    ConversationEngine::append_message_plain(&mut out, &msg);
    assert!(out.ends_with('\n'));
}

#[test]
fn append_message_plain_preserves_existing_newline() {
    let msg = ChatMessage {
        role: MessageRole::User,
        status: MessageStatus::Completed,
        content: "has trailing newline\n".to_string(),
        tool_call: None,
        created_at: std::time::Instant::now(),
    };

    let mut out = String::new();
    ConversationEngine::append_message_plain(&mut out, &msg);
    assert_eq!(out, "has trailing newline\n");
    // Should not add a second newline
    assert!(!out.ends_with("\n\n"));
}

#[test]
fn followup_queue_not_affected_by_conversation_engine() {
    let mut engine = new_engine();
    engine.followup_queue.push("task1".to_string());

    engine.handle_user_message("hello");
    engine.handle_agent_event(&AgentEvent::TurnStart);

    assert_eq!(engine.followup_queue.len(), 1);
    assert_eq!(engine.followup_queue[0], "task1");
}

// ---------------------------------------------------------------------------
// Stream source verification
// ---------------------------------------------------------------------------

#[tokio::test]
async fn user_message_stream_source_is_user() {
    let mut engine = new_engine();
    let outputs = engine.handle_user_message("test");
    let (source, _) = collect_stream(outputs).await.unwrap();
    assert_eq!(source, MessageSource::User);
}

#[tokio::test]
async fn error_stream_source_is_error() {
    let mut engine = new_engine();
    let outputs = engine.handle_agent_event(&AgentEvent::Error {
        message: "test error".to_string(),
    });
    let (source, _) = collect_stream(outputs).await.unwrap();
    assert_eq!(source, MessageSource::Error);
}

#[tokio::test]
async fn slash_help_stream_source_is_system() {
    let mut engine = new_engine();
    let outputs = engine.handle_slash_command("/help");
    let (source, _) = collect_stream(outputs).await.unwrap();
    assert_eq!(source, MessageSource::System);
}

#[tokio::test]
async fn slash_status_stream_source_is_system() {
    let mut engine = new_engine();
    let outputs = engine.handle_slash_command("/status");
    let (source, _) = collect_stream(outputs).await.unwrap();
    assert_eq!(source, MessageSource::System);
}

#[tokio::test]
async fn unknown_command_stream_source_is_error() {
    let mut engine = new_engine();
    let outputs = engine.handle_slash_command("/unknown");
    let (source, _) = collect_stream(outputs).await.unwrap();
    assert_eq!(source, MessageSource::Error);
}
