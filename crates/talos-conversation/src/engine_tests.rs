#![allow(warnings)]
use futures::StreamExt;
use talos_core::message::{AgentEvent, MessageToolResult, StopReason, ToolCall, Usage};
use talos_core::tool::ToolProvenance;

use crate::engine::ConversationEngine;
use crate::types::{
    ChatMessage, ContentOutput, LoadedPluginDiagnostic, McpServerDiagnostic, MessageRole,
    MessageSource, MessageStatus, ModelInfo, ModelPickerItem, PluginObservation,
    SkillCommandRequest, SkillDiagnostic, TipKind, TodoCommandAction, TodoExportFormat,
    ToolCallDisplay, ToolResultDisplay, TurnPhase, UiOutput,
};

fn new_engine() -> ConversationEngine {
    ConversationEngine::new("claude-sonnet-4".to_string(), "anthropic".to_string())
}

fn make_tool_call(name: &str, _provenance: ToolProvenance) -> ToolCall {
    ToolCall {
        id: "tc-1".to_string(),
        name: name.to_string(),
        input: serde_json::json!({}),
    }
}

fn make_tool_result(content: &str, is_error: bool) -> MessageToolResult {
    MessageToolResult {
        tool_use_id: "tc-1".to_string(),
        content: content.to_string(),
        is_error,
    }
}

/// Extract the first logical content block from ordered or legacy outputs.
async fn collect_stream(outputs: Vec<UiOutput>) -> Option<(MessageSource, String)> {
    let mut ordered_source = None;
    let mut ordered_text = String::new();
    for output in outputs {
        match output {
            UiOutput::Content(ContentOutput::Block { source, text }) => {
                return Some((source, text));
            }
            UiOutput::Content(ContentOutput::Start { source }) => {
                ordered_source = Some(source);
            }
            UiOutput::Content(ContentOutput::Delta { text }) => ordered_text.push_str(&text),
            UiOutput::Content(ContentOutput::End) if ordered_source.is_some() => {
                return ordered_source.map(|source| (source, ordered_text));
            }
            UiOutput::Stream(msg) => {
                let source = msg.source.clone();
                let chunks: Vec<String> = msg.stream.collect().await;
                return Some((source, chunks.join("")));
            }
            _ => {}
        }
    }
    ordered_source.map(|source| (source, ordered_text))
}

fn find_tool_call(outputs: &[UiOutput]) -> Option<&ToolCallDisplay> {
    outputs.iter().find_map(|o| match o {
        UiOutput::ToolCall(d) => Some(d),
        _ => None,
    })
}

fn find_tool_result(outputs: &[UiOutput]) -> Option<&ToolResultDisplay> {
    outputs.iter().find_map(|o| match o {
        UiOutput::ToolResult(d) => Some(d),
        _ => None,
    })
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
fn turn_start_creates_status_and_defers_content_until_delta() {
    let mut engine = new_engine();
    engine.current_turn_text = "leftover".to_string();
    engine.is_processing = false;

    let outputs = engine.handle_agent_event(&AgentEvent::TurnStart);

    assert_eq!(outputs.len(), 1);
    assert!(matches!(&outputs[0], UiOutput::Status(_)));

    assert!(engine.current_turn_text.is_empty());
    assert!(engine.is_processing);
}

// ---------------------------------------------------------------------------
// handle_agent_event: TextDelta
// ---------------------------------------------------------------------------

#[tokio::test]
async fn text_delta_sends_chunks_to_stream() {
    let mut engine = new_engine();
    engine.handle_agent_event(&AgentEvent::TurnStart);

    let outputs = engine.handle_agent_event(&AgentEvent::TextDelta {
        delta: "Hello".to_string(),
    });
    assert!(matches!(
        outputs[0],
        UiOutput::Content(ContentOutput::Start { .. })
    ));
    assert!(
        matches!(&outputs[1], UiOutput::Content(ContentOutput::Delta { text }) if text == "Hello")
    );
    assert!(find_status(&outputs).is_some());
    assert_eq!(engine.current_turn_text, "Hello");

    let outputs = engine.handle_agent_event(&AgentEvent::TextDelta {
        delta: " World".to_string(),
    });
    assert!(
        matches!(&outputs[0], UiOutput::Content(ContentOutput::Delta { text }) if text == " World")
    );
    assert_eq!(engine.current_turn_text, "Hello World");

    let outputs = engine.handle_agent_event(&AgentEvent::TurnEnd {
        stop_reason: StopReason::EndTurn,
        usage: Usage::default(),
    });
    assert!(matches!(outputs[0], UiOutput::Content(ContentOutput::End)));
}

#[tokio::test]
async fn text_delta_accumulates_multiline() {
    let mut engine = new_engine();
    engine.handle_agent_event(&AgentEvent::TurnStart);

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
        summary_fields: vec![],
    });

    assert_eq!(outputs.len(), 2);
    let display = find_tool_call(&outputs).unwrap();
    assert_eq!(display.tool_name, "bash");
    assert_eq!(display.provenance, ToolProvenance::Native);
    let status = find_status(&outputs).expect("tool call status");
    assert_eq!(
        status.phase,
        Some(TurnPhase::RunningTool {
            name: "bash".to_string()
        })
    );

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

    engine.handle_agent_event(&AgentEvent::TurnStart);

    engine.handle_agent_event(&AgentEvent::TextDelta {
        delta: "partial".to_string(),
    });

    let outputs = engine.handle_agent_event(&AgentEvent::ToolCall {
        call: make_tool_call("read", ToolProvenance::Native),
        provenance: ToolProvenance::Native,
        summary_fields: vec![],
    });

    assert!(matches!(outputs[0], UiOutput::Content(ContentOutput::End)));
    assert_eq!(outputs.len(), 3);
    let display = find_tool_call(&outputs).unwrap();
    assert!(matches!(display.provenance, ToolProvenance::Native));
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
        summary_fields: vec![],
    });

    let outputs = engine.handle_agent_event(&AgentEvent::ToolResult {
        result: make_tool_result("file contents", false),
    });

    assert_eq!(outputs.len(), 2);
    let display = find_tool_result(&outputs).unwrap();
    assert_eq!(display.tool_name.as_deref(), Some("read_file"));
    assert!(!display.is_error);
    assert_eq!(display.content, "file contents");
    let status = find_status(&outputs).expect("tool result status");
    assert_eq!(status.phase, Some(TurnPhase::Generating));

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
        summary_fields: vec![],
    });

    let outputs = engine.handle_agent_event(&AgentEvent::ToolResult {
        result: make_tool_result("command not found", true),
    });

    let tc = engine.messages[0].tool_call.as_ref().unwrap();
    let result = tc.result.as_ref().unwrap();
    assert!(result.is_error);

    let display = find_tool_result(&outputs).unwrap();
    assert!(display.is_error);
    assert!(display.content.contains("command not found"));
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
    let completed =
        engine.handle_turn_completed(&talos_core::session::TurnCompletionStatus::Success {
            final_text: "Done.\n".to_string(),
            new_messages: vec![],
        });

    assert!(!engine.is_processing);
    assert_eq!(engine.messages.len(), 1);
    assert_eq!(engine.messages[0].role, MessageRole::Assistant);
    assert_eq!(engine.messages[0].content, "Done.\n");
    assert_eq!(engine.usage, usage);

    let snapshot = find_status(&completed).unwrap();
    assert_eq!(snapshot.model_name, "claude-sonnet-4");
    assert_eq!(snapshot.usage.input_tokens, 100);
    assert_eq!(snapshot.usage.output_tokens, 50);
    assert!(!snapshot.is_processing);
    assert!(find_status(&outputs).is_some());
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

    assert!(engine.is_processing);
    assert_eq!(engine.messages.len(), 0);
    assert_eq!(engine.usage, usage);

    assert_eq!(outputs.len(), 1);
    assert!(find_status(&outputs).is_some());

    let completed =
        engine.handle_turn_completed(&talos_core::session::TurnCompletionStatus::Success {
            final_text: String::new(),
            new_messages: vec![],
        });
    assert!(!find_status(&completed).unwrap().is_processing);
}

#[test]
fn turn_end_max_tokens_waits_for_session_completion() {
    let mut engine = new_engine();
    engine.handle_agent_event(&AgentEvent::TurnStart);
    engine.handle_agent_event(&AgentEvent::TextDelta {
        delta: "partial response cut off".to_string(),
    });
    assert!(engine.is_processing);

    let outputs = engine.handle_agent_event(&AgentEvent::TurnEnd {
        stop_reason: StopReason::MaxTokens,
        usage: Usage::default(),
    });

    assert!(engine.is_processing);
    assert_eq!(engine.current_phase, None);
    let status = find_status(&outputs).expect("max-tokens turn end must emit status");
    assert!(status.is_processing);
    assert!(
        engine
            .messages
            .iter()
            .any(|m| m.role == MessageRole::Assistant && m.content == "partial response cut off")
    );

    let completed =
        engine.handle_turn_completed(&talos_core::session::TurnCompletionStatus::Success {
            final_text: "partial response cut off".to_string(),
            new_messages: vec![],
        });
    assert!(!find_status(&completed).unwrap().is_processing);
}

#[test]
fn turn_end_tool_use_keeps_processing_for_continuation() {
    let mut engine = new_engine();
    engine.handle_agent_event(&AgentEvent::TurnStart);
    assert!(engine.is_processing);

    let _ = engine.handle_agent_event(&AgentEvent::TurnEnd {
        stop_reason: StopReason::ToolUse,
        usage: Usage::default(),
    });

    assert!(
        engine.is_processing,
        "ToolUse stop reason must keep processing so the following ToolCall continuation is not lost"
    );
}

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
    assert!(text.contains("/plugins"));
    assert!(text.contains("/mcp"));
    assert!(text.contains("/skills"));
    assert!(text.contains("/copy"));
    assert!(text.contains("/export"));
    assert!(text.contains("/new"));
    assert!(text.contains("/resume"));
    assert!(text.contains("/fork"));
    assert!(!text.contains("/compact"));
    assert!(!text.contains("/mock-request"));
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
// handle_slash_command: /plugins (transition notice) and /mcp
// ---------------------------------------------------------------------------

#[tokio::test]
async fn slash_plugins_shows_diagnostics() {
    let mut engine = new_engine();
    engine.plugin_observations.push(PluginObservation {
        key: "native".to_string(),
        count: 5,
    });

    let outputs = engine.handle_slash_command("/plugins");

    assert_eq!(outputs.len(), 1);
    let (_, text) = collect_stream(outputs).await.unwrap();
    assert!(text.contains("WASM plugin packages: none loaded"));
    assert!(text.contains("Use /mcp for MCP detail"));
    assert!(text.contains("Provenance observations: 1"));
}

#[tokio::test]
async fn slash_plugins_notice_does_not_leak_mcp_status() {
    let mut engine = new_engine().with_mcp_servers(vec![McpServerDiagnostic {
        name: "github".to_string(),
        connected: true,
        tool_count: 3,
        error: None,
    }]);

    let outputs = engine.handle_slash_command("/plugins");

    let (_, text) = collect_stream(outputs).await.unwrap();
    assert!(!text.contains("MCP servers (startup snapshot)"));
    assert!(
        !text.contains("github"),
        "individual server names must not appear in /plugins: {text}"
    );
}

#[tokio::test]
async fn slash_plugins_shows_loaded_plugin_packages() {
    let mut engine = new_engine().with_loaded_plugins(vec![LoadedPluginDiagnostic {
        name: "demo".to_string(),
        version: "0.1.0".to_string(),
        carrier: "wasm".to_string(),
        capabilities: vec!["demo.greet".to_string()],
    }]);

    let outputs = engine.handle_slash_command("/plugins");

    let (_, text) = collect_stream(outputs).await.unwrap();
    assert!(
        text.contains("WASM plugin packages: 1 loaded"),
        "/plugins must show loaded packages: {text}"
    );
    assert!(
        text.contains("demo@0.1.0/wasm"),
        "/plugins must show package identity: {text}"
    );
    assert!(
        text.contains("capabilities: demo.greet"),
        "/plugins must show declared capability: {text}"
    );
}

#[tokio::test]
async fn slash_mcp_shows_observations() {
    let mut engine = new_engine();
    engine.plugin_observations.push(PluginObservation {
        key: "native".to_string(),
        count: 5,
    });
    engine.plugin_observations.push(PluginObservation {
        key: "mcp:github".to_string(),
        count: 2,
    });

    let outputs = engine.handle_slash_command("/mcp");

    assert_eq!(outputs.len(), 1);
    let (_, text) = collect_stream(outputs).await.unwrap();
    assert!(text.contains("Observed tool provenance"));
    assert!(text.contains("native") && text.contains("5 calls"));
    assert!(text.contains("mcp:github") && text.contains("2 calls"));
}

#[tokio::test]
async fn slash_mcp_empty_shows_no_tools_message() {
    let mut engine = new_engine();

    let outputs = engine.handle_slash_command("/mcp");

    assert_eq!(outputs.len(), 1);
    let (_, text) = collect_stream(outputs).await.unwrap();
    assert!(text.contains("No MCP servers configured"));
}

#[tokio::test]
async fn slash_mcp_shows_startup_mcp_status_before_any_call() {
    let mut engine = new_engine().with_mcp_servers(vec![McpServerDiagnostic {
        name: "github".to_string(),
        connected: true,
        tool_count: 3,
        error: None,
    }]);

    let outputs = engine.handle_slash_command("/mcp");

    let (_, text) = collect_stream(outputs).await.unwrap();
    assert!(text.contains("MCP servers (startup snapshot)"));
    assert!(text.contains("github (connected, 3 tools)"));
}

#[tokio::test]
async fn slash_hooks_shows_read_only_diagnostics() {
    let mut engine = new_engine();

    let outputs = engine.handle_slash_command("/hooks");

    assert_eq!(outputs.len(), 1);
    let (_, text) = collect_stream(outputs).await.unwrap();
    assert!(text.contains("Hooks diagnostics"));
    assert!(text.contains("config-introduced hooks: none declared"));
    assert!(text.contains("executable hook carriers: disabled"));
    assert!(text.contains("BeforeProviderCall"));
    assert!(text.contains("TurnComplete"));
}

// ---------------------------------------------------------------------------
// handle_slash_command: /skills
// ---------------------------------------------------------------------------

#[tokio::test]
async fn slash_skills_empty_shows_no_skills_message() {
    let mut engine = new_engine();

    let outputs = engine.handle_slash_command("/skills");

    assert_eq!(outputs.len(), 1);
    let (_, text) = collect_stream(outputs).await.unwrap();
    assert!(text.contains("No skills available"));
}

#[tokio::test]
async fn slash_skills_lists_runtime_skill_metadata() {
    let mut engine = new_engine().with_skills(vec![SkillDiagnostic {
        name: "review".to_string(),
        description: "Review code".to_string(),
        active: false,
        source: "project".to_string(),
    }]);

    let outputs = engine.handle_slash_command("/skills");

    assert_eq!(outputs.len(), 1);
    let (_, text) = collect_stream(outputs).await.unwrap();
    assert!(text.contains("Available skills"));
    assert!(text.contains("review"));
    assert!(text.contains("Review code"));
    assert!(text.contains("/skills activate"));
}

#[tokio::test]
async fn slash_skills_activate_emits_typed_request() {
    let mut engine = new_engine().with_skills(vec![SkillDiagnostic {
        name: "review".to_string(),
        description: "Review code".to_string(),
        active: false,
        source: "project".to_string(),
    }]);

    let outputs = engine.handle_slash_command("/skills activate review");

    assert_eq!(outputs.len(), 1);
    match &outputs[0] {
        UiOutput::SkillCommand(SkillCommandRequest::Activate { name }) => {
            assert_eq!(name, "review");
        }
        _ => panic!("expected skill activation request"),
    }
}

#[tokio::test]
async fn slash_skills_reference_emits_typed_request() {
    let mut engine = new_engine().with_skills(vec![SkillDiagnostic {
        name: "review".to_string(),
        description: "Review code".to_string(),
        active: true,
        source: "project".to_string(),
    }]);

    let outputs = engine.handle_slash_command("/skills reference references/rules.md");

    assert_eq!(outputs.len(), 1);
    match &outputs[0] {
        UiOutput::SkillCommand(SkillCommandRequest::Reference { path }) => {
            assert_eq!(path, "references/rules.md");
        }
        _ => panic!("expected skill reference request"),
    }
}

#[tokio::test]
async fn slash_skills_activate_requires_name() {
    let mut engine = new_engine();

    let outputs = engine.handle_slash_command("/skills activate");

    let (source, text) = collect_stream(outputs).await.unwrap();
    assert_eq!(source, MessageSource::Error);
    assert!(text.contains("Usage: /skills activate <name>"));
}

#[tokio::test]
async fn slash_skills_output_shows_source() {
    let mut engine = new_engine().with_skills(vec![SkillDiagnostic {
        name: "git-skill".to_string(),
        description: "Git operations".to_string(),
        active: false,
        source: "project".to_string(),
    }]);

    let outputs = engine.handle_slash_command("/skills");

    let (_, text) = collect_stream(outputs).await.unwrap();
    assert!(text.contains("git-skill (project)"));
    assert!(text.contains("Git operations"));
}

#[tokio::test]
async fn slash_skills_shared_shows_shared_source() {
    let mut engine = new_engine().with_skills(vec![SkillDiagnostic {
        name: "shared-skill".to_string(),
        description: "Shared tooling".to_string(),
        active: false,
        source: "shared".to_string(),
    }]);

    let outputs = engine.handle_slash_command("/skills");

    let (_, text) = collect_stream(outputs).await.unwrap();
    assert!(text.contains("shared-skill (shared)"));
}

#[tokio::test]
async fn slash_skills_output_does_not_leak_body() {
    let mut engine = new_engine().with_skills(vec![SkillDiagnostic {
        name: "secret-skill".to_string(),
        description: "Does secret things".to_string(),
        active: false,
        source: "user".to_string(),
    }]);

    let outputs = engine.handle_slash_command("/skills");

    let (_, text) = collect_stream(outputs).await.unwrap();
    assert!(text.contains("secret-skill (user)"));
    assert!(text.contains("Does secret things"));
    assert!(!text.contains("## Skill Body"));
    assert!(!text.contains("skill body content"));
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
            UiOutput::Status(status) if !status.is_processing && status.phase == Some(TurnPhase::Cancelled)
        )
    }));
}

#[test]
fn phase_transitions_turnstart_to_thinking_to_generating_to_end() {
    let mut engine = new_engine();

    let start = engine.handle_agent_event(&AgentEvent::TurnStart);
    let start_status = find_status(&start).expect("turn start status");
    assert_eq!(start_status.phase, Some(TurnPhase::Connecting));

    let thinking = engine.handle_agent_event(&AgentEvent::ThinkingDelta {
        delta: "analyzing".to_string(),
    });
    let thinking_status = find_status(&thinking).expect("thinking status");
    assert_eq!(thinking_status.phase, Some(TurnPhase::Thinking));

    let generating = engine.handle_agent_event(&AgentEvent::TextDelta {
        delta: "result".to_string(),
    });
    let generating_status = find_status(&generating).expect("text delta status");
    assert_eq!(generating_status.phase, Some(TurnPhase::Generating));

    let end = engine.handle_agent_event(&AgentEvent::TurnEnd {
        usage: Usage::default(),
        stop_reason: StopReason::EndTurn,
    });
    let end_status = find_status(&end).expect("turn end status");
    assert_eq!(end_status.phase, None);
}

#[test]
fn timeout_error_sets_timed_out_phase() {
    let mut engine = new_engine();
    engine.handle_agent_event(&AgentEvent::TurnStart);

    let outputs = engine.handle_agent_event(&AgentEvent::Error {
        message: "request timed out after 30s".to_string(),
    });
    let status = find_status(&outputs).expect("error status");
    assert_eq!(status.phase, Some(TurnPhase::TimedOut));
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
        summary_fields: vec![],
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
        summary_fields: vec![],
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
        summary_fields: vec![],
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
        summary_fields: vec![],
    });
    engine.handle_agent_event(&AgentEvent::ToolCall {
        call: make_tool_call("bash", ToolProvenance::Native),
        provenance: ToolProvenance::Native,
        summary_fields: vec![],
    });
    engine.handle_agent_event(&AgentEvent::ToolCall {
        call: make_tool_call("bash", ToolProvenance::Native),
        provenance: ToolProvenance::Native,
        summary_fields: vec![],
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
        summary_fields: vec![],
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
        summary_fields: vec![],
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
        summary_fields: vec![],
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

#[test]
fn provenance_plugin_key() {
    let mut engine = new_engine();
    let provenance = ToolProvenance::Plugin {
        name: "my-plugin".to_string(),
        version: "0.1.0".to_string(),
        carrier: "wasm".to_string(),
    };

    engine.handle_agent_event(&AgentEvent::ToolCall {
        call: make_tool_call("custom_tool", provenance.clone()),
        provenance,
        summary_fields: vec![],
    });

    assert_eq!(engine.plugin_observations.len(), 1);
    assert_eq!(
        engine.plugin_observations[0].key,
        "plugin:my-plugin@0.1.0/wasm"
    );
    assert_eq!(engine.plugin_observations[0].count, 1);
}

#[test]
fn provenance_truncates_long_plugin_names() {
    let mut engine = new_engine();
    let long_name = "a".repeat(30);
    let provenance = ToolProvenance::Plugin {
        name: long_name.clone(),
        version: "0.1.0".to_string(),
        carrier: "wasm".to_string(),
    };

    engine.handle_agent_event(&AgentEvent::ToolCall {
        call: make_tool_call("tool", provenance.clone()),
        provenance,
        summary_fields: vec![],
    });

    assert_eq!(engine.plugin_observations.len(), 1);
    let key = &engine.plugin_observations[0].key;
    assert!(key.starts_with("plugin:"));
    assert!(key.contains('…'));
    assert!(key.ends_with("@0.1.0/wasm"));
}

#[test]
fn provenance_groups_plugin_packages_separately_from_mcp() {
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
        summary_fields: vec![],
    });
    engine.handle_agent_event(&AgentEvent::ToolCall {
        call: make_tool_call(
            "custom",
            ToolProvenance::Plugin {
                name: "my-plugin".to_string(),
                version: "0.1.0".to_string(),
                carrier: "wasm".to_string(),
            },
        ),
        provenance: ToolProvenance::Plugin {
            name: "my-plugin".to_string(),
            version: "0.1.0".to_string(),
            carrier: "wasm".to_string(),
        },
        summary_fields: vec![],
    });

    assert_eq!(engine.plugin_observations.len(), 2);
    assert!(
        engine
            .plugin_observations
            .iter()
            .any(|e| e.key == "mcp:github")
    );
    assert!(
        engine
            .plugin_observations
            .iter()
            .any(|e| e.key == "plugin:my-plugin@0.1.0/wasm")
    );
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
        summary_fields: vec![],
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
        summary_fields: vec![],
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

    assert!(completions.contains(&"/help"));
    assert!(completions.contains(&"/quit"));
    assert!(completions.contains(&"/exit"));
    assert!(completions.contains(&"/status"));
    assert!(completions.contains(&"/plugins"));
    assert!(completions.contains(&"/skills"));
    assert!(completions.contains(&"/copy"));
    assert!(completions.contains(&"/export"));
    assert_eq!(completions, ConversationEngine::slash_commands());
}

#[test]
fn complete_slash_command_matches_prefix() {
    let engine = new_engine();

    let completions = engine.complete_slash_command("/h");

    assert_eq!(completions, vec!["/help", "/hooks"]);
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

    assert_eq!(completions, vec!["/copy", "/connect"]);
}

#[test]
fn complete_slash_command_hides_mock_request_diagnostics() {
    let engine = new_engine();

    let completions = engine.complete_slash_command("/mock");

    assert!(completions.is_empty());
}

#[tokio::test]
async fn every_visible_slash_command_has_an_execution_path() {
    let mut engine = new_engine();
    let commands: Vec<String> = engine
        .complete_slash_command("")
        .into_iter()
        .map(str::to_owned)
        .collect();

    for command in commands {
        let outputs = engine.handle_slash_command(&command);
        assert!(!outputs.is_empty(), "{command} produced no output");
        if let Some((_, text)) = collect_stream(outputs).await {
            assert!(
                !text.contains("Unknown command"),
                "{command} is only a placeholder"
            );
        }
    }
}

#[test]
fn slash_todo_list_produces_read_only_request() {
    let mut engine = new_engine();

    let outputs = engine.handle_slash_command("/todo list --status blocked --sort created");

    assert_eq!(outputs.len(), 1);
    match &outputs[0] {
        UiOutput::TodoCommand(req) => {
            assert_eq!(req.action, TodoCommandAction::List);
            assert_eq!(req.status_filter.as_deref(), Some("blocked"));
            assert_eq!(req.sort.as_deref(), Some("created"));
        }
        _ => panic!("expected todo command request"),
    }
}

#[test]
fn slash_todo_show_produces_read_only_request() {
    let mut engine = new_engine();

    let outputs = engine.handle_slash_command("/todo show abc123");

    match &outputs[0] {
        UiOutput::TodoCommand(req) => {
            assert_eq!(
                req.action,
                TodoCommandAction::Show {
                    id: "abc123".to_string()
                }
            );
        }
        _ => panic!("expected todo command request"),
    }
}

#[test]
fn slash_todo_export_json_produces_read_only_request() {
    let mut engine = new_engine();

    let outputs = engine.handle_slash_command("/todo export json --priority high");

    match &outputs[0] {
        UiOutput::TodoCommand(req) => {
            assert_eq!(
                req.action,
                TodoCommandAction::Export {
                    format: TodoExportFormat::Json
                }
            );
            assert_eq!(req.priority_filter.as_deref(), Some("high"));
        }
        _ => panic!("expected todo command request"),
    }
}

#[test]
fn slash_todo_delete_with_confirm_parses_request() {
    let mut engine = new_engine();

    let outputs = engine.handle_slash_command("/todo delete abc12345 --confirm");

    match &outputs[0] {
        UiOutput::TodoCommand(req) => {
            assert_eq!(
                req.action,
                TodoCommandAction::Delete {
                    id: "abc12345".to_string(),
                    confirm: true,
                }
            );
        }
        _ => panic!("expected todo command request"),
    }
}

#[test]
fn slash_todo_delete_without_confirm_parses_request() {
    let mut engine = new_engine();

    let outputs = engine.handle_slash_command("/todo delete abc12345");

    match &outputs[0] {
        UiOutput::TodoCommand(req) => {
            assert_eq!(
                req.action,
                TodoCommandAction::Delete {
                    id: "abc12345".to_string(),
                    confirm: false,
                }
            );
        }
        _ => panic!("expected todo command request"),
    }
}

#[tokio::test]
async fn slash_todo_delete_without_id_returns_error() {
    let mut engine = new_engine();

    let outputs = engine.handle_slash_command("/todo delete");

    let (source, text) = collect_stream(outputs).await.unwrap();
    assert_eq!(source, MessageSource::Error);
    assert!(text.contains("Usage: /todo delete"));
}

#[test]
fn thinking_delta_updates_preview_without_history() {
    let mut engine = new_engine();
    engine.handle_agent_event(&AgentEvent::TurnStart);

    let outputs = engine.handle_agent_event(&AgentEvent::ThinkingDelta {
        delta: "checking constraints".to_string(),
    });

    assert!(matches!(
        &outputs[0],
        UiOutput::ThinkingPreview { text: Some(text) } if text == "checking constraints"
    ));
    assert!(engine.messages.is_empty());

    let outputs = engine.handle_agent_event(&AgentEvent::TurnEnd {
        usage: Usage::default(),
        stop_reason: StopReason::EndTurn,
    });
    assert!(
        outputs
            .iter()
            .any(|output| matches!(output, UiOutput::ThinkingPreview { text: None }))
    );
    assert!(
        outputs.iter().any(|output| matches!(
            output,
            UiOutput::Content(ContentOutput::Block {
                source: MessageSource::Reasoning,
                text,
            }) if text == "Thinking: checking constraints\n"
        )),
        "finalized thinking must use the non-streaming output path"
    );
    assert_eq!(engine.messages.len(), 1);
    assert_eq!(engine.messages[0].role, MessageRole::Reasoning);
    assert_eq!(engine.messages[0].content, "checking constraints");
}

// ---------------------------------------------------------------------------

#[tokio::test]
async fn slash_agile_without_workspace_returns_unavailable() {
    let mut engine = new_engine();
    let outputs = engine.handle_slash_command("/agile status");
    assert_eq!(outputs.len(), 1);
    if let Some((_, text)) = collect_stream(outputs).await {
        assert!(text.contains("unavailable") || text.contains("no workspace"));
    }
}

#[tokio::test]
async fn slash_agile_with_workspace_returns_governance_summary() {
    let dir = tempfile::tempdir().unwrap();
    let docs_dir = dir.path().join("docs");
    std::fs::create_dir_all(docs_dir.join("iterations")).unwrap();
    std::fs::write(
        docs_dir.join("BOARD.md"),
        "# Board\n\n## Now\n\n| Item | State | Owner Doc | Gate |\n|---|---|---|---|\n| I080 Test | Active | [x](x.md) | Gate |\n\n## Next\n\n| Item | State | Owner Doc | Gate |\n|---|---|---|---|\n| I081 Next | Planned | [x](x.md) | Evidence |\n",
    )
    .unwrap();
    std::fs::write(
        docs_dir.join("iterations").join("README.md"),
        "## Current Iterations\n\n| ID | Codename | State | Verified |\n|---|---|---|---|\n| I080 | Frontline | Planned | no |\n",
    )
    .unwrap();

    let mut engine = ConversationEngine::new("test".to_string(), "test".to_string())
        .with_workspace_root(dir.path().to_path_buf());
    let outputs = engine.handle_slash_command("/agile status");
    assert_eq!(outputs.len(), 1);
    if let Some((_, text)) = collect_stream(outputs).await {
        assert!(text.contains("Governance Status"));
        assert!(text.contains("I080 Test"));
        assert!(text.contains("I081 Next"));
        assert!(text.contains("Frontline"));
    }
}

#[tokio::test]
async fn slash_validate_without_workspace_returns_unavailable() {
    let mut engine = new_engine();
    let outputs = engine.handle_slash_command("/validate governance");
    assert_eq!(outputs.len(), 1);
    let (_, text) = collect_stream(outputs).await.unwrap();
    assert!(text.contains("unavailable") || text.contains("no workspace"));
}

#[tokio::test]
async fn slash_validate_governance_uses_internal_profile() {
    let dir = tempfile::tempdir().unwrap();
    let gov_dir = dir.path().join(".agent-governance");
    std::fs::create_dir_all(&gov_dir).unwrap();
    std::fs::write(gov_dir.join("manifest.yaml"), "profile: product\n").unwrap();
    let script_dir = dir.path().join("scripts");
    std::fs::create_dir_all(&script_dir).unwrap();
    std::fs::write(
        script_dir.join("validate_project_governance.sh"),
        "#!/usr/bin/env bash\ntouch executed-marker\n",
    )
    .unwrap();

    let mut engine = ConversationEngine::new("test".to_string(), "test".to_string())
        .with_workspace_root(dir.path().to_path_buf());
    let outputs = engine.handle_slash_command("/validate governance");

    assert_eq!(outputs.len(), 1);
    let (_, text) = collect_stream(outputs).await.unwrap();
    assert!(text.contains("Talos Validation Evidence"));
    assert!(text.contains("internal:governance_validation"));
    assert!(text.contains("execution") || text.contains("allowlisted validation profile"));
    assert!(!dir.path().join("executed-marker").exists());
}

#[tokio::test]
async fn slash_validate_rejects_host_tool_profiles() {
    let mut engine = new_engine();
    let outputs = engine.handle_slash_command("/validate workspace");
    assert_eq!(outputs.len(), 1);
    let (_, text) = collect_stream(outputs).await.unwrap();
    assert!(text.contains("Unsupported internal validation profile"));
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
    engine.set_model_info(&ModelInfo {
        model_name: "claude-sonnet-4".to_string(),
        provider: "anthropic".to_string(),
        variant: Some("medium-reasoning".to_string()),
        ..Default::default()
    });

    let snapshot = engine.status_snapshot();

    assert_eq!(snapshot.model_name, "claude-sonnet-4");
    assert_eq!(snapshot.steering_count, 1);
    assert_eq!(snapshot.followup_count, 2);
    assert!(snapshot.is_processing);
    assert_eq!(snapshot.branch_id, Some("b-123".to_string()));
    assert_eq!(snapshot.variant.as_deref(), Some("medium-reasoning"));
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

    engine.handle_agent_event(&AgentEvent::TurnStart);

    let text_outputs = engine.handle_agent_event(&AgentEvent::TextDelta {
        delta: "2+2 equals 4.\n".to_string(),
    });
    assert!(matches!(
        &text_outputs[1],
        UiOutput::Content(ContentOutput::Delta { text }) if text == "2+2 equals 4.\n"
    ));

    let outputs = engine.handle_agent_event(&AgentEvent::ToolCall {
        call: make_tool_call("calculator", ToolProvenance::Native),
        provenance: ToolProvenance::Native,
        summary_fields: vec![],
    });
    let display = find_tool_call(&outputs).unwrap();
    assert_eq!(display.tool_name, "calculator");

    let outputs = engine.handle_agent_event(&AgentEvent::ToolResult {
        result: make_tool_result("4", false),
    });
    let result_display = find_tool_result(&outputs).unwrap();
    assert!(result_display.content.contains("4"));

    let outputs = engine.handle_agent_event(&AgentEvent::TurnEnd {
        stop_reason: StopReason::EndTurn,
        usage: Usage {
            input_tokens: 200,
            output_tokens: 30,
            ..Default::default()
        },
    });
    let completed =
        engine.handle_turn_completed(&talos_core::session::TurnCompletionStatus::Success {
            final_text: "2+2 equals 4.\n".to_string(),
            new_messages: vec![],
        });

    assert!(!engine.is_processing);
    assert_eq!(engine.messages.len(), 3);

    let last = &engine.messages[engine.messages.len() - 1];
    assert_eq!(last.role, MessageRole::Assistant);
    assert_eq!(last.content, "2+2 equals 4.\n");

    assert!(outputs.iter().any(|o| matches!(o, UiOutput::Status(_))));
    assert!(!find_status(&completed).unwrap().is_processing);
}

#[test]
fn canonical_tool_loop_uses_one_fifo_content_protocol_without_legacy_streams() {
    let mut engine = new_engine();
    let mut outputs = engine.handle_turn_started();
    outputs.extend(engine.handle_agent_event(&AgentEvent::TurnStart));
    outputs.extend(engine.handle_agent_event(&AgentEvent::ThinkingDelta {
        delta: "inspect first".into(),
    }));
    outputs.extend(engine.handle_agent_event(&AgentEvent::TextDelta {
        delta: "before tool\n".into(),
    }));
    outputs.extend(engine.handle_agent_event(&AgentEvent::ToolCallStarted {
        name: "read".into(),
    }));
    outputs.extend(engine.handle_agent_event(&AgentEvent::ToolCall {
        call: make_tool_call("read", ToolProvenance::Native),
        provenance: ToolProvenance::Native,
        summary_fields: vec![],
    }));
    outputs.extend(engine.handle_agent_event(&AgentEvent::TurnEnd {
        stop_reason: StopReason::ToolUse,
        usage: Usage::default(),
    }));
    outputs.extend(engine.handle_agent_event(&AgentEvent::ToolResult {
        result: make_tool_result("tool output", false),
    }));
    outputs.extend(engine.handle_agent_event(&AgentEvent::TurnStart));
    outputs.extend(engine.handle_agent_event(&AgentEvent::TextDelta {
        delta: "after tool\n".into(),
    }));
    outputs.extend(engine.handle_agent_event(&AgentEvent::TurnEnd {
        stop_reason: StopReason::EndTurn,
        usage: Usage::default(),
    }));
    outputs.extend(engine.handle_turn_completed(
        &talos_core::session::TurnCompletionStatus::Success {
            final_text: "after tool\n".into(),
            new_messages: vec![],
        },
    ));

    assert!(
        outputs
            .iter()
            .all(|output| !matches!(output, UiOutput::Stream(_))),
        "canonical runtime path must not create nested stream receivers"
    );
    let content = outputs
        .iter()
        .filter_map(|output| match output {
            UiOutput::Content(ContentOutput::Block { source, text })
                if *source == MessageSource::Reasoning =>
            {
                Some(text.as_str())
            }
            UiOutput::Content(ContentOutput::Delta { text }) => Some(text.as_str()),
            _ => None,
        })
        .collect::<String>();
    assert_eq!(
        content,
        "Thinking: inspect first\nbefore tool\nafter tool\n"
    );
    assert!(!engine.is_processing());
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
        summary_fields: vec![],
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

// ---------------------------------------------------------------------------
// handle_slash_command: /fork
// ---------------------------------------------------------------------------

#[tokio::test]
async fn slash_fork_produces_session_fork_output() {
    let mut engine = new_engine();
    let outputs = engine.handle_slash_command("/fork");
    assert_eq!(outputs.len(), 1);
    assert!(matches!(outputs[0], UiOutput::SessionFork(_)));
}

#[tokio::test]
async fn slash_fork_refuses_while_processing() {
    let mut engine = new_engine();
    engine.is_processing = true;
    let outputs = engine.handle_slash_command("/fork");
    assert_eq!(outputs.len(), 1);
    let (source, text) = collect_stream(outputs).await.unwrap();
    assert_eq!(source, MessageSource::System);
    assert!(text.contains("Cannot fork a session while a turn is active"));
}

#[tokio::test]
async fn slash_fork_stream_source_is_system() {
    let mut engine = new_engine();
    engine.is_processing = true;
    let outputs = engine.handle_slash_command("/fork");
    let (source, _) = collect_stream(outputs).await.unwrap();
    assert_eq!(source, MessageSource::System);
}

// ---------------------------------------------------------------------------
// handle_slash_command: /model
// ---------------------------------------------------------------------------

#[test]
fn slash_model_no_arg_emits_switch_request_with_empty_id() {
    let mut engine = new_engine();
    let outputs = engine.handle_slash_command("/model");
    assert_eq!(outputs.len(), 1);
    match &outputs[0] {
        UiOutput::ModelSwitchRequest(req) => {
            assert_eq!(req.model_id, "");
            assert!(!req.provider_needs_credential);
        }
        _ => panic!("expected ModelSwitchRequest"),
    }
}

#[test]
fn slash_model_with_id_emits_switch_request_with_model_id() {
    let mut engine = new_engine();
    let outputs = engine.handle_slash_command("/model gpt-4o");
    assert_eq!(outputs.len(), 1);
    match &outputs[0] {
        UiOutput::ModelSwitchRequest(req) => {
            assert_eq!(req.model_id, "gpt-4o");
            assert!(!req.provider_needs_credential);
        }
        _ => panic!("expected ModelSwitchRequest"),
    }
}

#[tokio::test]
async fn slash_model_refuses_while_processing() {
    let mut engine = new_engine();
    engine.is_processing = true;
    let outputs = engine.handle_slash_command("/model");
    assert_eq!(outputs.len(), 1);
    let (source, text) = collect_stream(outputs).await.unwrap();
    assert_eq!(source, MessageSource::System);
    assert!(text.contains("Cannot switch models while a turn is active"));
}

#[tokio::test]
async fn slash_model_stream_source_is_system_when_processing() {
    let mut engine = new_engine();
    engine.is_processing = true;
    let outputs = engine.handle_slash_command("/model claude-sonnet-4");
    let (source, _) = collect_stream(outputs).await.unwrap();
    assert_eq!(source, MessageSource::System);
}

#[test]
fn model_picker_item_fields_accessible() {
    let item = ModelPickerItem {
        command: "/model".to_string(),
        model_id: "claude-sonnet-4-20250514".to_string(),
        provider: "anthropic".to_string(),
        label: "claude-sonnet-4-20250514   Anthropic  200K  $3/$15".to_string(),
        context_limit: Some(200_000),
        pricing: Some("$3/$15 per 1M".to_string()),
        authenticated: true,
        is_current: false,
        variants: vec![],
        variant: None,
    };
    assert_eq!(item.command, "/model");
    assert_eq!(item.model_id, "claude-sonnet-4-20250514");
    assert_eq!(item.provider, "anthropic");
    assert_eq!(item.context_limit, Some(200_000));
    assert_eq!(item.pricing.as_deref(), Some("$3/$15 per 1M"));
    assert!(item.authenticated);
}

#[test]
fn model_picker_item_unauthenticated_flag() {
    let item = ModelPickerItem {
        command: "/model".to_string(),
        model_id: "gpt-4o".to_string(),
        provider: "openai".to_string(),
        label: "gpt-4o   OpenAI  128K".to_string(),
        context_limit: Some(128_000),
        pricing: None,
        authenticated: false,
        is_current: false,
        variants: vec![],
        variant: None,
    };
    assert!(!item.authenticated);
    assert!(item.pricing.is_none());
}

// --- RUNTIME-002: is_processing clears on terminal error paths ---

#[test]
fn error_after_tool_call_clears_processing() {
    let mut engine = new_engine();
    // Simulate: TurnStart → ToolCall → Error (provider failure after tool use)
    engine.handle_agent_event(&AgentEvent::TurnStart);

    // Emit a ToolCall (phase becomes RunningTool, processing stays true).
    engine.handle_agent_event(&AgentEvent::ToolCall {
        call: make_tool_call("bash", ToolProvenance::Native),
        provenance: ToolProvenance::Native,
        summary_fields: vec![],
    });
    assert!(engine.is_processing(), "still processing after ToolCall");

    // Provider error after tool call — is_processing must clear.
    let outputs = engine.handle_agent_event(&AgentEvent::Error {
        message: "provider connection reset after tool call".to_string(),
    });
    assert!(
        !engine.is_processing(),
        "is_processing should be false after Error"
    );
    assert_eq!(engine.current_phase, Some(TurnPhase::Failed));
    // Verify a Status output is emitted with is_processing=false.
    let status = find_status(&outputs).expect("error must emit status");
    assert!(!status.is_processing);
}

#[test]
fn error_after_tool_result_clears_processing() {
    let mut engine = new_engine();
    // Simulate: TurnStart → ToolCall → ToolResult → Error (provider failure after results)
    engine.handle_agent_event(&AgentEvent::TurnStart);

    engine.handle_agent_event(&AgentEvent::ToolCall {
        call: make_tool_call("bash", ToolProvenance::Native),
        provenance: ToolProvenance::Native,
        summary_fields: vec![],
    });
    engine.handle_agent_event(&AgentEvent::ToolResult {
        result: make_tool_result("ok", false),
    });
    assert!(engine.is_processing(), "still processing after ToolResult");

    // Provider error after tool result — is_processing must clear.
    let outputs = engine.handle_agent_event(&AgentEvent::Error {
        message: "provider internal error after tool results".to_string(),
    });
    assert!(
        !engine.is_processing(),
        "is_processing should be false after Error"
    );
    assert_eq!(engine.current_phase, Some(TurnPhase::Failed));
    let status = find_status(&outputs).expect("error must emit status");
    assert!(!status.is_processing);
}

#[test]
fn error_without_prior_turn_clears_processing() {
    let mut engine = new_engine();
    // Error event may arrive without any prior TurnStart (edge case).
    engine.is_processing = true;

    let outputs = engine.handle_agent_event(&AgentEvent::Error {
        message: "unexpected error before turn start".to_string(),
    });
    assert!(
        !engine.is_processing(),
        "is_processing should clear even without prior turn"
    );
    assert_eq!(engine.current_phase, Some(TurnPhase::Failed));
    let status = find_status(&outputs).expect("error must emit status");
    assert!(!status.is_processing);
}

#[test]
fn error_sets_visible_terminal_phase() {
    let mut engine = new_engine();
    engine.handle_agent_event(&AgentEvent::TurnStart);
    engine.handle_agent_event(&AgentEvent::ToolResult {
        result: make_tool_result("done", false),
    });

    // Generic provider error → Failed
    let outputs = engine.handle_agent_event(&AgentEvent::Error {
        message: "provider error".to_string(),
    });
    assert_eq!(engine.current_phase, Some(TurnPhase::Failed));

    // Timeout error → TimedOut
    engine.is_processing = true; // reset for test
    let outputs = engine.handle_agent_event(&AgentEvent::Error {
        message: "request timed out after 10s".to_string(),
    });
    assert_eq!(engine.current_phase, Some(TurnPhase::TimedOut));
}

#[test]
fn error_message_becomes_tip_and_error_stream() {
    let mut engine = new_engine();
    let outputs = engine.handle_agent_event(&AgentEvent::Error {
        message: "connection reset by peer".to_string(),
    });
    // Must include a Tip with the error message
    let has_tip = outputs.iter().any(|o| {
        matches!(o, UiOutput::Tip { text, kind: TipKind::Error } if text == "connection reset by peer")
    });
    assert!(has_tip, "error must emit a Tip with the error message");
    // Must include an ordered error content block.
    let has_error_stream = outputs.iter().any(|o| {
        matches!(
            o,
            UiOutput::Content(ContentOutput::Block {
                source: MessageSource::Error,
                ..
            })
        )
    });
    assert!(has_error_stream, "error must emit an Error stream");
}

#[test]
fn extension_snapshot_empty() {
    let engine = new_engine();
    let snap = engine.extension_snapshot();
    assert!(snap.mcp_servers.is_empty());
    assert!(snap.hooks.declarations.is_empty());
    assert!(!snap.hooks.executable_carriers_enabled);
    assert!(snap.provenance.is_empty());
    assert!(snap.collisions.is_empty());
}

#[test]
fn extension_snapshot_with_mcp_servers() {
    let engine = new_engine().with_mcp_servers(vec![
        McpServerDiagnostic {
            name: "filesystem".to_string(),
            connected: true,
            tool_count: 3,
            error: None,
        },
        McpServerDiagnostic {
            name: "remote".to_string(),
            connected: false,
            tool_count: 0,
            error: Some("timeout".to_string()),
        },
    ]);
    let snap = engine.extension_snapshot();
    assert_eq!(snap.mcp_servers.len(), 2);
    assert!(
        snap.mcp_servers
            .iter()
            .any(|s| s.name == "filesystem" && s.connected)
    );
    assert!(
        snap.mcp_servers
            .iter()
            .any(|s| s.name == "remote" && !s.connected)
    );
}

#[test]
fn extension_snapshot_with_hooks() {
    let mut engine = new_engine();
    engine.set_hook_declarations(vec![
        ("pre-turn".to_string(), "TurnStart".to_string(), true),
        ("post-tool".to_string(), "AfterToolCall".to_string(), false),
    ]);
    let snap = engine.extension_snapshot();
    assert_eq!(snap.hooks.declarations.len(), 2);
    assert!(
        snap.hooks
            .declarations
            .iter()
            .any(|d| d.name == "pre-turn" && d.enabled)
    );
    assert!(
        snap.hooks
            .declarations
            .iter()
            .any(|d| d.name == "post-tool" && !d.enabled)
    );
    assert!(!snap.hooks.event_catalog.is_empty());
}

#[test]
fn extension_snapshot_detects_mcp_name_collision() {
    let engine = new_engine().with_mcp_servers(vec![
        McpServerDiagnostic {
            name: "dup".to_string(),
            connected: true,
            tool_count: 1,
            error: None,
        },
        McpServerDiagnostic {
            name: "dup".to_string(),
            connected: false,
            tool_count: 0,
            error: Some("conflict".to_string()),
        },
    ]);
    let snap = engine.extension_snapshot();
    assert!(
        snap.collisions.iter().any(|c| c == "mcp:dup"),
        "collisions: {:?}",
        snap.collisions
    );
}

#[test]
fn extension_snapshot_detects_hook_name_collision() {
    let mut engine = new_engine();
    engine.set_hook_declarations(vec![
        ("my-hook".to_string(), "TurnStart".to_string(), true),
        ("my-hook".to_string(), "AfterToolCall".to_string(), true),
    ]);
    let snap = engine.extension_snapshot();
    assert!(
        snap.collisions.iter().any(|c| c == "hook:my-hook"),
        "collisions: {:?}",
        snap.collisions
    );
}

#[test]
fn extension_snapshot_serializes_to_valid_json() {
    let mut engine = new_engine();
    engine.set_hook_declarations(vec![("h".to_string(), "TurnStart".to_string(), true)]);
    let engine = engine.with_mcp_servers(vec![McpServerDiagnostic {
        name: "s".to_string(),
        connected: true,
        tool_count: 2,
        error: None,
    }]);
    let snap = engine.extension_snapshot();
    let json = serde_json::to_string(&snap).expect("serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("parse");
    assert!(value["mcp_servers"].is_array());
    assert!(value["hooks"].is_object());
    assert!(value["provenance"].is_array());
    assert!(value["collisions"].is_array());
}

#[test]
fn extension_snapshot_no_secrets() {
    let engine = new_engine().with_mcp_servers(vec![McpServerDiagnostic {
        name: "server".to_string(),
        connected: true,
        tool_count: 1,
        error: None,
    }]);
    let snap = engine.extension_snapshot();
    let json = serde_json::to_string(&snap).expect("serialize");
    assert!(!json.to_lowercase().contains("api_key"));
    assert!(!json.to_lowercase().contains("secret"));
    assert!(!json.to_lowercase().contains("token"));
    assert!(!json.to_lowercase().contains("password"));
}

#[tokio::test]
async fn slash_mcp_shows_unavailable_server_error() {
    let mut engine = new_engine().with_mcp_servers(vec![McpServerDiagnostic {
        name: "broken-server".to_string(),
        connected: false,
        tool_count: 0,
        error: Some("connection refused".to_string()),
    }]);
    let outputs = engine.handle_slash_command("/mcp");
    let (_, text) = collect_stream(outputs).await.unwrap();
    assert!(text.contains("broken-server"), "server name must appear");
    assert!(
        text.contains("unavailable"),
        "unavailable status must appear"
    );
    assert!(
        text.contains("connection_failed"),
        "bounded error category must appear instead of raw text: {text}"
    );
}

#[tokio::test]
async fn slash_hooks_shows_disabled_hook() {
    let mut engine = new_engine();
    engine.set_hook_declarations(vec![(
        "my-hook".to_string(),
        "TurnStart".to_string(),
        false,
    )]);
    let outputs = engine.handle_slash_command("/hooks");
    let (_, text) = collect_stream(outputs).await.unwrap();
    assert!(text.contains("my-hook"), "hook name must appear");
    assert!(text.contains("disabled"), "disabled status must appear");
}

#[tokio::test]
async fn slash_plugins_shows_summary_counts() {
    let mut engine = new_engine();
    engine.set_hook_declarations(vec![("h".to_string(), "TurnStart".to_string(), true)]);
    let mut engine = engine.with_mcp_servers(vec![
        McpServerDiagnostic {
            name: "s1".to_string(),
            connected: true,
            tool_count: 2,
            error: None,
        },
        McpServerDiagnostic {
            name: "s2".to_string(),
            connected: false,
            tool_count: 0,
            error: Some("timeout".to_string()),
        },
    ]);
    let outputs = engine.handle_slash_command("/plugins");
    let (_, text) = collect_stream(outputs).await.unwrap();
    assert!(text.contains("MCP servers: 2"), "total count: {text}");
    assert!(text.contains("1 connected"), "connected count: {text}");
    assert!(text.contains("Hook declarations: 1"), "hook count: {text}");
}

#[tokio::test]
async fn slash_mcp_shows_collision_warning() {
    let mut engine = new_engine().with_mcp_servers(vec![
        McpServerDiagnostic {
            name: "dup".to_string(),
            connected: true,
            tool_count: 1,
            error: None,
        },
        McpServerDiagnostic {
            name: "dup".to_string(),
            connected: false,
            tool_count: 0,
            error: Some("conflict".to_string()),
        },
    ]);
    let outputs = engine.handle_slash_command("/mcp");
    let (_, text) = collect_stream(outputs).await.unwrap();
    assert!(
        text.contains("collision"),
        "collision must be visible: {text}"
    );
    assert!(
        text.contains("mcp:dup"),
        "collision identifier must appear: {text}"
    );
}

#[test]
fn extension_snapshot_no_crash_on_empty_state() {
    let engine = new_engine();
    let snap = engine.extension_snapshot();
    assert!(snap.mcp_servers.is_empty());
    assert!(snap.collisions.is_empty());
    assert!(
        snap.hooks.event_catalog.len() > 0,
        "event catalog should always be populated"
    );
}

#[test]
fn extension_snapshot_categorizes_api_key_error() {
    let engine = new_engine().with_mcp_servers(vec![McpServerDiagnostic {
        name: "leaky".to_string(),
        connected: false,
        tool_count: 0,
        error: Some(
            "failed to connect to https://api.example.com/v1?api_key=sk-secret-key".to_string(),
        ),
    }]);
    let snap = engine.extension_snapshot();
    let error = snap.mcp_servers[0].error.as_ref().unwrap();
    assert!(
        !error.contains("sk-secret-key") && !error.contains("api_key"),
        "no raw substring of error text may appear: {error}"
    );
}

#[test]
fn extension_snapshot_categorizes_bearer_token_error() {
    let engine = new_engine().with_mcp_servers(vec![McpServerDiagnostic {
        name: "auth".to_string(),
        connected: false,
        tool_count: 0,
        error: Some("Authorization: Bearer abc123token failed".to_string()),
    }]);
    let snap = engine.extension_snapshot();
    let error = snap.mcp_servers[0].error.as_ref().unwrap();
    assert!(
        !error.contains("abc123token") && !error.contains("Bearer"),
        "no raw substring of error text may appear: {error}"
    );
}

#[test]
fn extension_snapshot_categorizes_url_query_secret() {
    let engine = new_engine().with_mcp_servers(vec![McpServerDiagnostic {
        name: "url".to_string(),
        connected: false,
        tool_count: 0,
        error: Some("request to https://mcp.example.com/sse?token=hidden&ok=1 failed".to_string()),
    }]);
    let snap = engine.extension_snapshot();
    let error = snap.mcp_servers[0].error.as_ref().unwrap();
    assert!(
        !error.contains("hidden")
            && !error.contains("token=")
            && !error.contains("mcp.example.com"),
        "no raw substring of error text may appear: {error}"
    );
}

#[test]
fn extension_snapshot_categorizes_multiple_secrets_in_one_error() {
    let raw = "MCP failed: token=first token=second api_key=third secret=fourth";
    let engine = new_engine().with_mcp_servers(vec![McpServerDiagnostic {
        name: "multi".to_string(),
        connected: false,
        tool_count: 0,
        error: Some(raw.to_string()),
    }]);
    let snap = engine.extension_snapshot();
    let error = snap.mcp_servers[0].error.as_ref().unwrap();
    assert!(
        !error.contains("first")
            && !error.contains("second")
            && !error.contains("third")
            && !error.contains("fourth"),
        "no secret value may appear: {error}"
    );
    assert!(
        !error.contains("token=") && !error.contains("api_key=") && !error.contains("secret="),
        "no pattern name may appear: {error}"
    );
}

#[test]
fn extension_snapshot_error_is_bounded_category() {
    let cases = [
        ("MCP server 'x' timed out after 30s", "timeout"),
        (
            "invalid MCP config: missing transport",
            "invalid_configuration",
        ),
        ("failed to spawn MCP server 'x': not found", "spawn_failed"),
        ("MCP server 'x' disconnected", "disconnected"),
        ("connection refused by host", "connection_failed"),
        ("MCP RPC error from 'x': boom", "protocol_error"),
        ("initialization failed at step 1", "initialization_failed"),
        ("MCP HTTP error: status 500", "network_error"),
        ("something completely unknown", "unavailable"),
    ];
    for (raw, expected) in cases {
        let engine = new_engine().with_mcp_servers(vec![McpServerDiagnostic {
            name: "t".to_string(),
            connected: false,
            tool_count: 0,
            error: Some(raw.to_string()),
        }]);
        let snap = engine.extension_snapshot();
        let error = snap.mcp_servers[0].error.as_ref().unwrap();
        assert_eq!(
            error, expected,
            "raw {raw:?} should categorize as {expected:?}, got {error:?}"
        );
    }
}

#[test]
fn build_extension_snapshot_matches_engine_snapshot() {
    let mut engine = new_engine();
    engine.set_hook_declarations(vec![("h".to_string(), "TurnStart".to_string(), true)]);
    let engine = engine.with_mcp_servers(vec![McpServerDiagnostic {
        name: "s".to_string(),
        connected: true,
        tool_count: 1,
        error: None,
    }]);

    let from_engine = engine.extension_snapshot();
    let from_builder = crate::build_extension_snapshot(
        &[McpServerDiagnostic {
            name: "s".to_string(),
            connected: true,
            tool_count: 1,
            error: None,
        }],
        &[("h".to_string(), "TurnStart".to_string(), true)],
        &[],
    );

    assert_eq!(from_engine.mcp_servers, from_builder.mcp_servers);
    assert_eq!(
        from_engine.hooks.declarations,
        from_builder.hooks.declarations
    );
    assert_eq!(from_engine.collisions, from_builder.collisions);
}
