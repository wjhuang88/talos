#![allow(warnings)]
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use talos_core::message::{AgentEvent, Message, MessageToolResult, StopReason, ToolCall, Usage};
use talos_core::provider::{LanguageModel, ProviderError, ProviderResult};
use talos_core::tool::{AgentTool, ToolRegistry, ToolResult as ToolExecutionResult};
use talos_permission::{PermissionDecision, PermissionEngine};
use talos_plugin::{
    HookContext, HookEvent, HookEventKind, HookHandler, HookRegistry, HookResult, TurnId,
};
use talos_sandbox::{SandboxConfig, SandboxError, SandboxProvider, SandboxResult};
use talos_skill::SkillIndex;
use tokio::sync::Mutex;
use tokio::sync::mpsc;

use crate::{Agent, AgentError, AgentResult, PendingToolCall, ToolDescription};
type Receiver<T> = mpsc::Receiver<T>;

/// Mock language model that returns a predefined sequence of event batches,
/// one batch per call to `stream`.
struct MockModel {
    responses: Arc<Mutex<Vec<Vec<AgentEvent>>>>,
}

impl MockModel {
    fn new(responses: Vec<Vec<AgentEvent>>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(responses)),
        }
    }
}

#[async_trait]
impl LanguageModel for MockModel {
    async fn stream(&self, _messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
        let (tx, rx) = mpsc::channel(64);
        let responses = self.responses.clone();
        tokio::spawn(async move {
            let mut responses = responses.lock().await;
            let events = responses.pop_front().unwrap_or_default();
            for event in events {
                tx.send(event).await.expect("receiver dropped");
            }
        });
        Ok(rx)
    }
}

trait VecDequeExt<T> {
    fn pop_front(&mut self) -> Option<T>;
}

impl<T> VecDequeExt<T> for Vec<T> {
    fn pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            Some(self.remove(0))
        }
    }
}

/// Mock tool that records execution timing and returns a fixed result.
struct TimedMockTool {
    tool_name: String,
    read_only: bool,
    delay_ms: u64,
    result: ToolExecutionResult,
    execution_log: Arc<Mutex<Vec<String>>>,
}

#[async_trait]
impl AgentTool for TimedMockTool {
    fn name(&self) -> &str {
        &self.tool_name
    }

    fn description(&self) -> &str {
        "Mock tool for testing"
    }

    fn parameters(&self) -> Value {
        serde_json::json!({})
    }

    fn is_read_only(&self) -> bool {
        self.read_only
    }

    async fn execute(&self, input: Value) -> ToolExecutionResult {
        self.execution_log
            .lock()
            .await
            .push(format!("start:{}:{}", self.tool_name, input));
        tokio::time::sleep(tokio::time::Duration::from_millis(self.delay_ms)).await;
        self.execution_log
            .lock()
            .await
            .push(format!("end:{}:{}", self.tool_name, input));
        self.result.clone()
    }
}

struct CountingHook {
    events: Arc<Mutex<Vec<talos_plugin::HookEventKind>>>,
}

#[async_trait]
impl talos_plugin::HookHandler for CountingHook {
    fn name(&self) -> &str {
        "counting"
    }

    fn subscribed(&self) -> &'static [talos_plugin::HookEventKind] {
        &[
            talos_plugin::HookEventKind::TurnStart,
            talos_plugin::HookEventKind::BeforeProviderCall,
            talos_plugin::HookEventKind::TurnComplete,
        ]
    }

    async fn on_event(
        &self,
        _ctx: &talos_plugin::HookContext,
        event: &mut talos_plugin::HookEvent<'_>,
    ) -> talos_plugin::HookResult {
        self.events.lock().await.push(event.kind());
        talos_plugin::HookResult::Continue
    }
}

#[tokio::test]
#[allow(deprecated)] // Agent::new is correct for unit tests
async fn test_run_collects_text_deltas() {
    let events = vec![
        AgentEvent::TurnStart,
        AgentEvent::TextDelta {
            delta: "Hello, ".into(),
        },
        AgentEvent::TextDelta {
            delta: "world!".into(),
        },
        AgentEvent::TurnEnd {
            stop_reason: StopReason::EndTurn,
            usage: talos_core::message::Usage::default(),
        },
    ];

    let agent = Agent::new(Arc::new(MockModel::new(vec![events])), ToolRegistry::new());
    let response = agent.run("Hi".into()).await.unwrap();
    assert_eq!(response, "Hello, world!");
}

#[tokio::test]
async fn test_turn_start_hook_fires_once_for_tool_turn() {
    let call = ToolCall {
        id: "call-1".into(),
        name: "read".into(),
        input: serde_json::json!({}),
    };
    let responses = vec![
        vec![
            AgentEvent::TurnStart,
            AgentEvent::ToolCall {
                call,
                provenance: Default::default(),
                summary_fields: vec![],
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::ToolUse,
                usage: talos_core::message::Usage::default(),
            },
        ],
        vec![
            AgentEvent::TurnStart,
            AgentEvent::TextDelta {
                delta: "done".into(),
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                usage: talos_core::message::Usage::default(),
            },
        ],
    ];

    let events = Arc::new(Mutex::new(Vec::new()));
    let mut hooks = HookRegistry::new();
    hooks.register(Arc::new(CountingHook {
        events: events.clone(),
    }));

    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(TimedMockTool {
        tool_name: "read".into(),
        read_only: true,
        delay_ms: 0,
        result: ToolExecutionResult::success("file content"),
        execution_log: Arc::new(Mutex::new(Vec::new())),
    }));

    let agent = Agent::with_security_and_hooks(
        Arc::new(MockModel::new(responses)),
        registry,
        Some(Arc::new(PermissionEngine::new())),
        None,
        PathBuf::from("/tmp"),
        Arc::new(hooks),
    );

    let response = agent.run("read file".into()).await.unwrap();
    assert_eq!(response, "done");

    let events = events.lock().await;
    let turn_start_count = events
        .iter()
        .filter(|kind| **kind == talos_plugin::HookEventKind::TurnStart)
        .count();
    let provider_call_count = events
        .iter()
        .filter(|kind| **kind == talos_plugin::HookEventKind::BeforeProviderCall)
        .count();
    assert_eq!(
        turn_start_count, 1,
        "TurnStart should fire once per user turn"
    );
    assert_eq!(
        provider_call_count, 2,
        "provider can be called multiple times in one user turn"
    );
}

#[tokio::test]
#[allow(deprecated)] // Agent::new is correct for unit tests
async fn test_run_handles_error_event() {
    let events = vec![
        AgentEvent::TurnStart,
        AgentEvent::Error {
            message: "something went wrong".into(),
        },
    ];

    let agent = Agent::new(Arc::new(MockModel::new(vec![events])), ToolRegistry::new());
    let result = agent.run("Hi".into()).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, AgentError::UnexpectedEvent(_)));
}

#[tokio::test]
#[allow(deprecated)] // Agent::new is correct for unit tests
async fn test_run_handles_channel_close_without_turn_end() {
    let agent = Agent::new(Arc::new(MockModel::new(vec![])), ToolRegistry::new());
    let result = agent.run("Hi".into()).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, AgentError::UnexpectedEvent(_)));
}

#[tokio::test]
#[allow(deprecated)] // Agent::new is correct for unit tests
async fn test_run_streaming_forwards_events() {
    let events = vec![
        AgentEvent::TurnStart,
        AgentEvent::TextDelta {
            delta: "Streaming".into(),
        },
        AgentEvent::TurnEnd {
            stop_reason: StopReason::EndTurn,
            usage: talos_core::message::Usage::default(),
        },
    ];

    let agent = Agent::new(
        Arc::new(MockModel::new(vec![events.clone()])),
        ToolRegistry::new(),
    );
    let (tx, mut rx) = mpsc::unbounded_channel::<AgentEvent>();

    let (response, _) = agent.run_streaming("Hi".into(), vec![], tx).await.unwrap();
    assert_eq!(response, "Streaming");

    let mut received = Vec::new();
    while let Ok(event) = rx.try_recv() {
        received.push(event);
    }
    assert_eq!(received.len(), events.len());
    assert_eq!(received, events);
}

#[tokio::test]
#[allow(deprecated)] // Agent::new is correct for unit tests
async fn test_tool_execution_loop_single_call() {
    let responses = vec![
        vec![
            AgentEvent::TurnStart,
            AgentEvent::TextDelta {
                delta: "Let me check ".into(),
            },
            AgentEvent::ToolCall {
                call: ToolCall {
                    id: "call_1".into(),
                    name: "echo".into(),
                    input: serde_json::json!({ "message": "hello" }),
                },
                provenance: Default::default(),
                summary_fields: vec![],
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::ToolUse,
                usage: talos_core::message::Usage::default(),
            },
        ],
        vec![
            AgentEvent::TurnStart,
            AgentEvent::TextDelta {
                delta: "The result is: hello".into(),
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                usage: talos_core::message::Usage::default(),
            },
        ],
    ];

    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(TimedMockTool {
        tool_name: "echo".into(),
        read_only: true,
        delay_ms: 0,
        result: ToolExecutionResult::success("hello"),
        execution_log: Arc::new(Mutex::new(Vec::new())),
    }));

    let agent = Agent::new(Arc::new(MockModel::new(responses)), registry);
    let response = agent.run("Echo hello".into()).await.unwrap();
    assert_eq!(response, "The result is: hello");
}

#[tokio::test]
#[allow(deprecated)] // Agent::new is correct for unit tests
async fn test_tool_execution_loop_multiple_calls() {
    let responses = vec![
        vec![
            AgentEvent::TurnStart,
            AgentEvent::ToolCall {
                call: ToolCall {
                    id: "call_1".into(),
                    name: "read".into(),
                    input: serde_json::json!({ "path": "a.txt" }),
                },
                provenance: Default::default(),
                summary_fields: vec![],
            },
            AgentEvent::ToolCall {
                call: ToolCall {
                    id: "call_2".into(),
                    name: "read".into(),
                    input: serde_json::json!({ "path": "b.txt" }),
                },
                provenance: Default::default(),
                summary_fields: vec![],
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::ToolUse,
                usage: talos_core::message::Usage::default(),
            },
        ],
        vec![
            AgentEvent::TurnStart,
            AgentEvent::TextDelta {
                delta: "Done reading both files".into(),
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                usage: talos_core::message::Usage::default(),
            },
        ],
    ];

    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(TimedMockTool {
        tool_name: "read".into(),
        read_only: true,
        delay_ms: 0,
        result: ToolExecutionResult::success("file content"),
        execution_log: Arc::new(Mutex::new(Vec::new())),
    }));

    let agent = Agent::new(Arc::new(MockModel::new(responses)), registry);
    let response = agent.run("Read files".into()).await.unwrap();
    assert_eq!(response, "Done reading both files");
}

#[tokio::test]
#[allow(deprecated)] // Agent::new is correct for unit tests
async fn test_concurrent_read_only_tools() {
    let responses = vec![
        vec![
            AgentEvent::TurnStart,
            AgentEvent::ToolCall {
                call: ToolCall {
                    id: "call_1".into(),
                    name: "fast_read".into(),
                    input: serde_json::json!({ "path": "a.txt" }),
                },
                provenance: Default::default(),
                summary_fields: vec![],
            },
            AgentEvent::ToolCall {
                call: ToolCall {
                    id: "call_2".into(),
                    name: "fast_read".into(),
                    input: serde_json::json!({ "path": "b.txt" }),
                },
                provenance: Default::default(),
                summary_fields: vec![],
            },
            AgentEvent::ToolCall {
                call: ToolCall {
                    id: "call_3".into(),
                    name: "fast_read".into(),
                    input: serde_json::json!({ "path": "c.txt" }),
                },
                provenance: Default::default(),
                summary_fields: vec![],
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::ToolUse,
                usage: talos_core::message::Usage::default(),
            },
        ],
        vec![
            AgentEvent::TurnStart,
            AgentEvent::TextDelta {
                delta: "All done".into(),
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                usage: talos_core::message::Usage::default(),
            },
        ],
    ];

    let log = Arc::new(Mutex::new(Vec::new()));
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(TimedMockTool {
        tool_name: "fast_read".into(),
        read_only: true,
        delay_ms: 50,
        result: ToolExecutionResult::success("ok"),
        execution_log: log.clone(),
    }));

    let agent = Agent::new(Arc::new(MockModel::new(responses)), registry);
    let _response = agent.run("Read files".into()).await.unwrap();

    let log_entries = log.lock().await;
    let starts: Vec<_> = log_entries
        .iter()
        .filter(|e| e.starts_with("start:"))
        .collect();
    let ends: Vec<_> = log_entries
        .iter()
        .filter(|e| e.starts_with("end:"))
        .collect();

    assert_eq!(starts.len(), 3);
    assert_eq!(ends.len(), 3);

    let last_start_idx = log_entries
        .iter()
        .position(|e| e.starts_with("end:"))
        .unwrap_or(3);
    assert!(
        last_start_idx >= 3,
        "Expected all starts before any end, but log was: {:?}",
        log_entries
    );
}

#[tokio::test]
#[allow(deprecated)] // Agent::new is correct for unit tests
async fn test_serial_write_tools() {
    let responses = vec![
        vec![
            AgentEvent::TurnStart,
            AgentEvent::ToolCall {
                call: ToolCall {
                    id: "call_1".into(),
                    name: "write".into(),
                    input: serde_json::json!({ "path": "a.txt", "content": "a" }),
                },
                provenance: Default::default(),
                summary_fields: vec![],
            },
            AgentEvent::ToolCall {
                call: ToolCall {
                    id: "call_2".into(),
                    name: "write".into(),
                    input: serde_json::json!({ "path": "b.txt", "content": "b" }),
                },
                provenance: Default::default(),
                summary_fields: vec![],
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::ToolUse,
                usage: talos_core::message::Usage::default(),
            },
        ],
        vec![
            AgentEvent::TurnStart,
            AgentEvent::TextDelta {
                delta: "Files written".into(),
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                usage: talos_core::message::Usage::default(),
            },
        ],
    ];

    let log = Arc::new(Mutex::new(Vec::new()));
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(TimedMockTool {
        tool_name: "write".into(),
        read_only: false,
        delay_ms: 30,
        result: ToolExecutionResult::success("written"),
        execution_log: log.clone(),
    }));

    let agent = Agent::new(Arc::new(MockModel::new(responses)), registry);
    let _response = agent.run("Write files".into()).await.unwrap();

    let log_entries = log.lock().await;
    assert_eq!(log_entries.len(), 4);

    assert!(log_entries[0].starts_with("start:"));
    assert!(log_entries[1].starts_with("end:"));
    assert!(log_entries[2].starts_with("start:"));
    assert!(log_entries[3].starts_with("end:"));
}

#[tokio::test]
#[allow(deprecated)] // Agent::new is correct for unit tests
async fn test_turn_budget_enforcement() {
    let mut events = vec![AgentEvent::TurnStart];
    for i in 0..51 {
        events.push(AgentEvent::ToolCall {
            call: ToolCall {
                id: format!("call_{i}"),
                name: "echo".into(),
                input: serde_json::json!({ "message": format!("msg_{i}") }),
            },
            provenance: Default::default(),
            summary_fields: vec![],
        });
    }
    events.push(AgentEvent::TurnEnd {
        stop_reason: StopReason::ToolUse,
        usage: talos_core::message::Usage::default(),
    });

    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(TimedMockTool {
        tool_name: "echo".into(),
        read_only: true,
        delay_ms: 0,
        result: ToolExecutionResult::success("ok"),
        execution_log: Arc::new(Mutex::new(Vec::new())),
    }));

    let agent = Agent::new(Arc::new(MockModel::new(vec![events])), registry);
    let result = agent.run("Many tools".into()).await;
    assert!(
        result.is_ok(),
        "budget exceeded should return Ok with messages, not Err"
    );
    let text = result.unwrap();
    assert!(
        text.contains("limit") || text.contains("preserved"),
        "final text should mention tool call limit: got {text}"
    );
}

#[tokio::test]
#[allow(deprecated)] // Agent::new is correct for unit tests
async fn test_turn_budget_allows_50_calls() {
    let mut tool_events = vec![AgentEvent::TurnStart];
    for i in 0..50 {
        tool_events.push(AgentEvent::ToolCall {
            call: ToolCall {
                id: format!("call_{i}"),
                name: "echo".into(),
                input: serde_json::json!({ "message": format!("msg_{i}") }),
            },
            provenance: Default::default(),
            summary_fields: vec![],
        });
    }
    tool_events.push(AgentEvent::TurnEnd {
        stop_reason: StopReason::ToolUse,
        usage: talos_core::message::Usage::default(),
    });

    let text_events = vec![
        AgentEvent::TurnStart,
        AgentEvent::TextDelta {
            delta: "Done".into(),
        },
        AgentEvent::TurnEnd {
            stop_reason: StopReason::EndTurn,
            usage: talos_core::message::Usage::default(),
        },
    ];

    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(TimedMockTool {
        tool_name: "echo".into(),
        read_only: true,
        delay_ms: 0,
        result: ToolExecutionResult::success("ok"),
        execution_log: Arc::new(Mutex::new(Vec::new())),
    }));

    let agent = Agent::new(
        Arc::new(MockModel::new(vec![tool_events, text_events])),
        registry,
    );
    let result = agent.run("50 tools".into()).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Done");
}

#[tokio::test]
#[allow(deprecated)] // Agent::new is correct for unit tests
async fn test_doom_loop_detection() {
    let tool_call_event = AgentEvent::ToolCall {
        call: ToolCall {
            id: "call_1".into(),
            name: "echo".into(),
            input: serde_json::json!({ "message": "same" }),
        },
        provenance: Default::default(),
        summary_fields: vec![],
    };

    let responses = vec![
        vec![
            AgentEvent::TurnStart,
            tool_call_event.clone(),
            AgentEvent::TurnEnd {
                stop_reason: StopReason::ToolUse,
                usage: talos_core::message::Usage::default(),
            },
        ],
        vec![
            AgentEvent::TurnStart,
            tool_call_event.clone(),
            AgentEvent::TurnEnd {
                stop_reason: StopReason::ToolUse,
                usage: talos_core::message::Usage::default(),
            },
        ],
        vec![
            AgentEvent::TurnStart,
            tool_call_event,
            AgentEvent::TurnEnd {
                stop_reason: StopReason::ToolUse,
                usage: talos_core::message::Usage::default(),
            },
        ],
    ];

    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(TimedMockTool {
        tool_name: "echo".into(),
        read_only: true,
        delay_ms: 0,
        result: ToolExecutionResult::success("same"),
        execution_log: Arc::new(Mutex::new(Vec::new())),
    }));

    let agent = Agent::new(Arc::new(MockModel::new(responses)), registry);
    let result = agent.run("Loop".into()).await;
    assert!(
        result.is_ok(),
        "doom loop should return Ok with messages, not Err"
    );
    let text = result.unwrap();
    assert!(
        text.contains("repeated") || text.contains("paused"),
        "final text should mention repeat pattern: got {text}"
    );
}

#[tokio::test]
#[allow(deprecated)] // Agent::new is correct for unit tests
async fn test_doom_loop_different_args_allowed() {
    let responses = vec![
        vec![
            AgentEvent::TurnStart,
            AgentEvent::ToolCall {
                call: ToolCall {
                    id: "call_1".into(),
                    name: "echo".into(),
                    input: serde_json::json!({ "message": "first" }),
                },
                provenance: Default::default(),
                summary_fields: vec![],
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::ToolUse,
                usage: talos_core::message::Usage::default(),
            },
        ],
        vec![
            AgentEvent::TurnStart,
            AgentEvent::ToolCall {
                call: ToolCall {
                    id: "call_2".into(),
                    name: "echo".into(),
                    input: serde_json::json!({ "message": "second" }),
                },
                provenance: Default::default(),
                summary_fields: vec![],
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::ToolUse,
                usage: talos_core::message::Usage::default(),
            },
        ],
        vec![
            AgentEvent::TurnStart,
            AgentEvent::TextDelta {
                delta: "Done".into(),
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                usage: talos_core::message::Usage::default(),
            },
        ],
    ];

    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(TimedMockTool {
        tool_name: "echo".into(),
        read_only: true,
        delay_ms: 0,
        result: ToolExecutionResult::success("ok"),
        execution_log: Arc::new(Mutex::new(Vec::new())),
    }));

    let agent = Agent::new(Arc::new(MockModel::new(responses)), registry);
    let result = agent.run("Different args".into()).await;
    assert!(result.is_ok());
}

#[tokio::test]
#[allow(deprecated)] // Agent::new is correct for unit tests
async fn test_tool_not_found_returns_error_result() {
    let responses = vec![
        vec![
            AgentEvent::TurnStart,
            AgentEvent::ToolCall {
                call: ToolCall {
                    id: "call_1".into(),
                    name: "nonexistent_tool".into(),
                    input: serde_json::json!({}),
                },
                provenance: Default::default(),
                summary_fields: vec![],
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::ToolUse,
                usage: talos_core::message::Usage::default(),
            },
        ],
        vec![
            AgentEvent::TurnStart,
            AgentEvent::TextDelta {
                delta: "Tool not available".into(),
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                usage: talos_core::message::Usage::default(),
            },
        ],
    ];

    let agent = Agent::new(Arc::new(MockModel::new(responses)), ToolRegistry::new());
    let result = agent.run("Missing tool".into()).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Tool not available");
}

#[tokio::test]
#[allow(deprecated)] // Agent::new is correct for unit tests
async fn test_tool_execution_error_feeds_back_to_provider() {
    let responses = vec![
        vec![
            AgentEvent::TurnStart,
            AgentEvent::ToolCall {
                call: ToolCall {
                    id: "call_1".into(),
                    name: "failing".into(),
                    input: serde_json::json!({}),
                },
                provenance: Default::default(),
                summary_fields: vec![],
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::ToolUse,
                usage: talos_core::message::Usage::default(),
            },
        ],
        vec![
            AgentEvent::TurnStart,
            AgentEvent::TextDelta {
                delta: "Tool failed, trying alternative".into(),
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                usage: talos_core::message::Usage::default(),
            },
        ],
    ];

    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(TimedMockTool {
        tool_name: "failing".into(),
        read_only: true,
        delay_ms: 0,
        result: ToolExecutionResult::error("internal failure"),
        execution_log: Arc::new(Mutex::new(Vec::new())),
    }));

    let agent = Agent::new(Arc::new(MockModel::new(responses)), registry);
    let result = agent.run("Failing tool".into()).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Tool failed, trying alternative");
}

#[tokio::test]
#[allow(deprecated)] // Agent::new is correct for unit tests
async fn test_mixed_read_only_and_write_tools() {
    let responses = vec![
        vec![
            AgentEvent::TurnStart,
            AgentEvent::ToolCall {
                call: ToolCall {
                    id: "call_1".into(),
                    name: "read".into(),
                    input: serde_json::json!({ "path": "a.txt" }),
                },
                provenance: Default::default(),
                summary_fields: vec![],
            },
            AgentEvent::ToolCall {
                call: ToolCall {
                    id: "call_2".into(),
                    name: "write".into(),
                    input: serde_json::json!({ "path": "b.txt", "content": "b" }),
                },
                provenance: Default::default(),
                summary_fields: vec![],
            },
            AgentEvent::ToolCall {
                call: ToolCall {
                    id: "call_3".into(),
                    name: "read".into(),
                    input: serde_json::json!({ "path": "c.txt" }),
                },
                provenance: Default::default(),
                summary_fields: vec![],
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::ToolUse,
                usage: talos_core::message::Usage::default(),
            },
        ],
        vec![
            AgentEvent::TurnStart,
            AgentEvent::TextDelta {
                delta: "Mixed tools done".into(),
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                usage: talos_core::message::Usage::default(),
            },
        ],
    ];

    let log = Arc::new(Mutex::new(Vec::new()));
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(TimedMockTool {
        tool_name: "read".into(),
        read_only: true,
        delay_ms: 20,
        result: ToolExecutionResult::success("read ok"),
        execution_log: log.clone(),
    }));
    registry.register(Arc::new(TimedMockTool {
        tool_name: "write".into(),
        read_only: false,
        delay_ms: 20,
        result: ToolExecutionResult::success("write ok"),
        execution_log: log.clone(),
    }));

    let agent = Agent::new(Arc::new(MockModel::new(responses)), registry);
    let result = agent.run("Mixed".into()).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Mixed tools done");

    let log_entries = log.lock().await;
    let write_start_idx = log_entries
        .iter()
        .position(|e| e.starts_with("start:write:"))
        .unwrap();
    let write_end_idx = log_entries
        .iter()
        .position(|e| e.starts_with("end:write:"))
        .unwrap();
    assert_eq!(
        write_end_idx,
        write_start_idx + 1,
        "Write tool should be serial: {:?}",
        log_entries
    );
}

#[tokio::test]
#[allow(deprecated)] // Agent::new is correct for unit tests
async fn test_cancellation_token_is_created() {
    let agent = Agent::new(Arc::new(MockModel::new(vec![])), ToolRegistry::new());
    let token = agent.cancellation_token();
    assert!(!token.is_cancelled());
    token.cancel();
    assert!(token.is_cancelled());
}

#[tokio::test]
#[allow(deprecated)] // Agent::new is correct for unit tests
async fn test_tool_result_events_broadcast() {
    let responses = vec![
        vec![
            AgentEvent::TurnStart,
            AgentEvent::ToolCall {
                call: ToolCall {
                    id: "call_1".into(),
                    name: "echo".into(),
                    input: serde_json::json!({ "message": "test" }),
                },
                provenance: Default::default(),
                summary_fields: vec![],
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::ToolUse,
                usage: talos_core::message::Usage::default(),
            },
        ],
        vec![
            AgentEvent::TurnStart,
            AgentEvent::TextDelta {
                delta: "Done".into(),
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                usage: talos_core::message::Usage::default(),
            },
        ],
    ];

    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(TimedMockTool {
        tool_name: "echo".into(),
        read_only: true,
        delay_ms: 0,
        result: ToolExecutionResult::success("test result"),
        execution_log: Arc::new(Mutex::new(Vec::new())),
    }));

    let agent = Agent::new(Arc::new(MockModel::new(responses)), registry);
    let (tx, mut rx) = mpsc::unbounded_channel::<AgentEvent>();

    let _response = agent
        .run_streaming("Echo test".into(), vec![], tx)
        .await
        .unwrap();

    let mut events = Vec::new();
    while let Ok(event) = rx.try_recv() {
        events.push(event);
    }

    let tool_result_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, AgentEvent::ToolResult { .. }))
        .collect();
    assert_eq!(
        tool_result_events.len(),
        1,
        "Expected 1 ToolResult event, got: {:?}",
        events
    );
}

#[tokio::test]
#[allow(deprecated)] // Agent::new is correct for unit tests
async fn test_streaming_tool_events_are_interleaved_per_tool() {
    let responses = vec![
        vec![
            AgentEvent::TurnStart,
            AgentEvent::ToolCall {
                call: ToolCall {
                    id: "call_1".into(),
                    name: "echo".into(),
                    input: serde_json::json!({ "message": "first" }),
                },
                provenance: Default::default(),
                summary_fields: vec![],
            },
            AgentEvent::ToolCall {
                call: ToolCall {
                    id: "call_2".into(),
                    name: "echo".into(),
                    input: serde_json::json!({ "message": "second" }),
                },
                provenance: Default::default(),
                summary_fields: vec![],
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::ToolUse,
                usage: talos_core::message::Usage::default(),
            },
        ],
        vec![
            AgentEvent::TurnStart,
            AgentEvent::TextDelta {
                delta: "Done".into(),
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                usage: talos_core::message::Usage::default(),
            },
        ],
    ];

    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(TimedMockTool {
        tool_name: "echo".into(),
        read_only: true,
        delay_ms: 0,
        result: ToolExecutionResult::success("ok"),
        execution_log: Arc::new(Mutex::new(Vec::new())),
    }));

    let agent = Agent::new(Arc::new(MockModel::new(responses)), registry);
    let (tx, mut rx) = mpsc::unbounded_channel::<AgentEvent>();

    let _response = agent
        .run_streaming("Echo twice".into(), vec![], tx)
        .await
        .unwrap();

    let mut sequence = Vec::new();
    while let Ok(event) = rx.try_recv() {
        match event {
            AgentEvent::ToolCall { call, .. } => {
                sequence.push(format!("call:{}", call.id));
            }
            AgentEvent::ToolResult { result } => {
                sequence.push(format!("result:{}", result.tool_use_id));
            }
            _ => {}
        }
    }

    assert_eq!(
        sequence,
        vec![
            "call:call_1",
            "result:call_1",
            "call:call_2",
            "result:call_2"
        ]
    );
}

/// Mock sandbox that tracks execution and returns configurable results.
struct MockSandbox {
    available: bool,
    execution_log: Arc<Mutex<Vec<String>>>,
    result: Option<SandboxResult>,
}

impl MockSandbox {
    fn new(available: bool, result: SandboxResult) -> Self {
        Self {
            available,
            execution_log: Arc::new(Mutex::new(Vec::new())),
            result: Some(result),
        }
    }

    fn unavailable() -> Self {
        Self {
            available: false,
            execution_log: Arc::new(Mutex::new(Vec::new())),
            result: None,
        }
    }
}

#[async_trait]
impl SandboxProvider for MockSandbox {
    async fn execute(
        &self,
        command: &str,
        _config: &SandboxConfig,
    ) -> Result<SandboxResult, SandboxError> {
        self.execution_log
            .lock()
            .await
            .push(format!("sandbox_execute:{command}"));
        if !self.available {
            return Err(SandboxError::NotAvailable);
        }
        Ok(self.result.clone().unwrap_or_else(|| SandboxResult {
            stdout: "sandboxed".into(),
            stderr: String::new(),
            exit_code: 0,
        }))
    }

    fn is_available(&self) -> bool {
        self.available
    }
}

#[tokio::test]
async fn test_permission_check_blocks_denied_tool() {
    let mut engine = PermissionEngine {
        rules: Vec::new(),
        workspace_root: None,
    };
    engine.add_rule(talos_permission::PermissionRule {
        tool_name: "echo".into(),
        path_pattern: None,
        decision: PermissionDecision::Deny("not allowed".into()),
        nature: None,
        resource: None,
        resource_kind: None,
    });

    let responses = vec![
        vec![
            AgentEvent::TurnStart,
            AgentEvent::ToolCall {
                call: ToolCall {
                    id: "call_1".into(),
                    name: "echo".into(),
                    input: serde_json::json!({ "message": "hello" }),
                },
                provenance: Default::default(),
                summary_fields: vec![],
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::ToolUse,
                usage: talos_core::message::Usage::default(),
            },
        ],
        vec![
            AgentEvent::TurnStart,
            AgentEvent::TextDelta {
                delta: "Done".into(),
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                usage: talos_core::message::Usage::default(),
            },
        ],
    ];

    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(TimedMockTool {
        tool_name: "echo".into(),
        read_only: true,
        delay_ms: 0,
        result: ToolExecutionResult::success("should not reach"),
        execution_log: Arc::new(Mutex::new(Vec::new())),
    }));

    let agent = Agent::with_security(
        Arc::new(MockModel::new(responses)),
        registry,
        Some(Arc::new(engine)),
        None,
        PathBuf::from("/tmp"),
    );

    let result = agent.run("Test".into()).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Done");
}

#[tokio::test]
async fn test_permission_check_allows_permitted_tool() {
    let mut engine = PermissionEngine {
        rules: Vec::new(),
        workspace_root: None,
    };
    engine.add_rule(talos_permission::PermissionRule {
        tool_name: "echo".into(),
        path_pattern: None,
        decision: PermissionDecision::Allow,
        nature: None,
        resource: None,
        resource_kind: None,
    });

    let responses = vec![
        vec![
            AgentEvent::TurnStart,
            AgentEvent::ToolCall {
                call: ToolCall {
                    id: "call_1".into(),
                    name: "echo".into(),
                    input: serde_json::json!({ "message": "hello" }),
                },
                provenance: Default::default(),
                summary_fields: vec![],
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::ToolUse,
                usage: talos_core::message::Usage::default(),
            },
        ],
        vec![
            AgentEvent::TurnStart,
            AgentEvent::TextDelta {
                delta: "Result: hello".into(),
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                usage: talos_core::message::Usage::default(),
            },
        ],
    ];

    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(TimedMockTool {
        tool_name: "echo".into(),
        read_only: true,
        delay_ms: 0,
        result: ToolExecutionResult::success("hello"),
        execution_log: Arc::new(Mutex::new(Vec::new())),
    }));

    let agent = Agent::with_security(
        Arc::new(MockModel::new(responses)),
        registry,
        Some(Arc::new(engine)),
        None,
        PathBuf::from("/tmp"),
    );

    let result = agent.run("Test".into()).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Result: hello");
}

#[tokio::test]
async fn test_permission_ask_defaults_to_deny() {
    let mut engine = PermissionEngine {
        rules: Vec::new(),
        workspace_root: None,
    };
    engine.add_rule(talos_permission::PermissionRule {
        tool_name: "echo".into(),
        path_pattern: None,
        decision: PermissionDecision::Ask,
        nature: None,
        resource: None,
        resource_kind: None,
    });

    let responses = vec![
        vec![
            AgentEvent::TurnStart,
            AgentEvent::ToolCall {
                call: ToolCall {
                    id: "call_1".into(),
                    name: "echo".into(),
                    input: serde_json::json!({ "message": "hello" }),
                },
                provenance: Default::default(),
                summary_fields: vec![],
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::ToolUse,
                usage: talos_core::message::Usage::default(),
            },
        ],
        vec![
            AgentEvent::TurnStart,
            AgentEvent::TextDelta {
                delta: "Denied".into(),
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                usage: talos_core::message::Usage::default(),
            },
        ],
    ];

    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(TimedMockTool {
        tool_name: "echo".into(),
        read_only: true,
        delay_ms: 0,
        result: ToolExecutionResult::success("should not reach"),
        execution_log: Arc::new(Mutex::new(Vec::new())),
    }));

    let agent = Agent::with_security(
        Arc::new(MockModel::new(responses)),
        registry,
        Some(Arc::new(engine)),
        None,
        PathBuf::from("/tmp"),
    );

    let result = agent.run("Test".into()).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Denied");
}

#[tokio::test]
async fn test_sandbox_execution_for_bash_tool() {
    let sandbox_result = SandboxResult {
        stdout: "sandboxed output".into(),
        stderr: String::new(),
        exit_code: 0,
    };
    let sandbox = MockSandbox::new(true, sandbox_result);
    let log = sandbox.execution_log.clone();

    let responses = vec![
        vec![
            AgentEvent::TurnStart,
            AgentEvent::ToolCall {
                call: ToolCall {
                    id: "call_1".into(),
                    name: "bash".into(),
                    input: serde_json::json!({ "command": "echo hello" }),
                },
                provenance: Default::default(),
                summary_fields: vec![],
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::ToolUse,
                usage: talos_core::message::Usage::default(),
            },
        ],
        vec![
            AgentEvent::TurnStart,
            AgentEvent::TextDelta {
                delta: "Done".into(),
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                usage: talos_core::message::Usage::default(),
            },
        ],
    ];

    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(TimedMockTool {
        tool_name: "bash".into(),
        read_only: false,
        delay_ms: 0,
        result: ToolExecutionResult::success("direct execution"),
        execution_log: Arc::new(Mutex::new(Vec::new())),
    }));

    let agent = Agent::with_security(
        Arc::new(MockModel::new(responses)),
        registry,
        None,
        Some(Box::new(sandbox)),
        PathBuf::from("/tmp"),
    );

    let result = agent.run("Test".into()).await;
    assert!(result.is_ok());

    let log_entries = log.lock().await;
    assert!(
        log_entries.iter().any(|e| e.contains("echo hello")),
        "Sandbox should have been called with the command, log: {:?}",
        log_entries
    );
}

#[tokio::test]
async fn test_sandbox_fallback_when_not_available() {
    let sandbox = MockSandbox::unavailable();

    let responses = vec![
        vec![
            AgentEvent::TurnStart,
            AgentEvent::ToolCall {
                call: ToolCall {
                    id: "call_1".into(),
                    name: "bash".into(),
                    input: serde_json::json!({ "command": "echo hello" }),
                },
                provenance: Default::default(),
                summary_fields: vec![],
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::ToolUse,
                usage: talos_core::message::Usage::default(),
            },
        ],
        vec![
            AgentEvent::TurnStart,
            AgentEvent::TextDelta {
                delta: "Fallback worked".into(),
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                usage: talos_core::message::Usage::default(),
            },
        ],
    ];

    let log = Arc::new(Mutex::new(Vec::new()));
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(TimedMockTool {
        tool_name: "bash".into(),
        read_only: false,
        delay_ms: 0,
        result: ToolExecutionResult::success("direct execution"),
        execution_log: log.clone(),
    }));

    let agent = Agent::with_security(
        Arc::new(MockModel::new(responses)),
        registry,
        None,
        Some(Box::new(sandbox)),
        PathBuf::from("/tmp"),
    );

    let result = agent.run("Test".into()).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Fallback worked");

    let log_entries = log.lock().await;
    assert!(
        log_entries.iter().any(|e| e.starts_with("start:bash:")),
        "Direct execution should have been used as fallback, log: {:?}",
        log_entries
    );
}

#[test]
fn test_agent_with_security_constructor() {
    let provider: Arc<dyn LanguageModel> = Arc::new(MockModel::new(vec![]));
    let tools = ToolRegistry::new();
    let permission = PermissionEngine::new();

    let agent = Agent::with_security(
        provider.clone(),
        tools,
        Some(Arc::new(permission)),
        None,
        PathBuf::from("/tmp/workspace"),
    );

    let _token = agent.cancellation_token();
    assert!(!_token.is_cancelled());
}

#[test]
#[allow(deprecated)] // Agent::new is correct for unit tests
fn test_agent_new_has_no_security() {
    let provider: Arc<dyn LanguageModel> = Arc::new(MockModel::new(vec![]));
    let tools = ToolRegistry::new();

    let agent = Agent::new(provider, tools);
    let _token = agent.cancellation_token();
    assert!(!_token.is_cancelled());
}

#[tokio::test]
async fn test_sandbox_result_to_tool_result_success() {
    let sandbox_result = SandboxResult {
        stdout: "hello".into(),
        stderr: "warning".into(),
        exit_code: 0,
    };

    let tool_result = Agent::sandbox_result_to_tool_result(sandbox_result);
    assert!(!tool_result.is_error);
    assert!(tool_result.content.contains("hello"));
    assert!(tool_result.content.contains("warning"));
}

#[tokio::test]
async fn test_sandbox_result_to_tool_result_error() {
    let sandbox_result = SandboxResult {
        stdout: String::new(),
        stderr: "error occurred".into(),
        exit_code: 1,
    };

    let tool_result = Agent::sandbox_result_to_tool_result(sandbox_result);
    assert!(tool_result.is_error);
    assert!(tool_result.content.contains("error occurred"));
}

#[tokio::test]
async fn test_execute_bash_in_sandbox_missing_command() {
    let sandbox = MockSandbox::new(
        true,
        SandboxResult {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: 0,
        },
    );

    let input = serde_json::json!({});
    let agent = Agent::with_security(
        Arc::new(MockModel::new(vec![])),
        ToolRegistry::new(),
        None,
        Some(Box::new(sandbox)),
        PathBuf::from("/tmp"),
    );
    let hook_ctx = HookContext::new(TurnId::new(), PathBuf::from("/tmp"));

    let result = agent
        .execute_bash_in_sandbox(
            &hook_ctx,
            agent.sandbox.as_deref().expect("sandbox should be present"),
            &input,
        )
        .await;
    assert!(result.is_error);
    assert!(result.content.contains("missing required field 'command'"));
}

#[test]
#[allow(deprecated)] // Agent::new is correct for prompt-builder unit tests.
fn test_clear_append_prompt() {
    let provider: Arc<dyn LanguageModel> = Arc::new(MockModel::new(vec![]));
    let tools = ToolRegistry::new();

    let mut agent = Agent::new(provider, tools);

    // Set an append prompt
    agent.set_append_prompt("test".to_string());

    // Verify it is set
    let prompt = agent.prompt_builder.build();
    assert!(prompt.contains("test"), "Append prompt should be set");

    // Clear the append prompt
    agent.clear_append_prompt();

    // Verify it is cleared
    let prompt = agent.prompt_builder.build();
    assert!(
        !prompt.contains("Additional Instructions"),
        "Append prompt section should be gone after clear"
    );
}

#[test]
#[allow(deprecated)] // Agent::new is correct for prompt-builder unit tests.
fn test_set_append_prompt_opt_none() {
    let provider: Arc<dyn LanguageModel> = Arc::new(MockModel::new(vec![]));
    let tools = ToolRegistry::new();

    let mut agent = Agent::new(provider, tools);

    // Set an append prompt
    agent.set_append_prompt("test".to_string());

    // Verify it is set
    let prompt = agent.prompt_builder.build();
    assert!(prompt.contains("test"), "Append prompt should be set");

    // Clear using set_append_prompt_opt(None)
    agent.set_append_prompt_opt(None);

    // Verify it is cleared (same as clear_append_prompt)
    let prompt = agent.prompt_builder.build();
    assert!(
        !prompt.contains("Additional Instructions"),
        "Append prompt section should be gone after set_append_prompt_opt(None)"
    );
}

/// Mock model that captures the system prompt from each stream call.
struct CapturingModel {
    responses: Arc<Mutex<Vec<Vec<AgentEvent>>>>,
    captured_system_prompts: Arc<std::sync::Mutex<Vec<String>>>,
}

impl CapturingModel {
    fn new(responses: Vec<Vec<AgentEvent>>) -> (Self, Arc<std::sync::Mutex<Vec<String>>>) {
        let captured = Arc::new(std::sync::Mutex::new(Vec::new()));
        (
            Self {
                responses: Arc::new(Mutex::new(responses)),
                captured_system_prompts: captured.clone(),
            },
            captured,
        )
    }
}

#[async_trait]
impl LanguageModel for CapturingModel {
    async fn stream(&self, messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
        for msg in messages {
            if let Message::System { content, .. } = msg {
                self.captured_system_prompts
                    .lock()
                    .expect("lock poisoned")
                    .push(content.clone());
            }
        }
        let (tx, rx) = mpsc::channel(64);
        let responses = self.responses.clone();
        tokio::spawn(async move {
            let mut responses = responses.lock().await;
            let events = responses.pop_front().unwrap_or_default();
            for event in events {
                tx.send(event).await.expect("receiver dropped");
            }
        });
        Ok(rx)
    }
}

#[tokio::test]
#[allow(deprecated)]
async fn test_stable_prefix_identical_across_turns() {
    let response_events = vec![
        AgentEvent::TurnStart,
        AgentEvent::TextDelta { delta: "OK".into() },
        AgentEvent::TurnEnd {
            stop_reason: StopReason::EndTurn,
            usage: Usage::default(),
        },
    ];

    let all_responses = vec![
        response_events.clone(),
        response_events.clone(),
        response_events.clone(),
    ];

    let (model, captured) = CapturingModel::new(all_responses);
    let agent = Agent::new(Arc::new(model), ToolRegistry::new());

    // Run 3 turns
    let _ = agent.run("turn 1".into()).await.unwrap();
    let _ = agent.run("turn 2".into()).await.unwrap();
    let _ = agent.run("turn 3".into()).await.unwrap();

    let prompts = captured.lock().expect("lock poisoned");
    assert_eq!(prompts.len(), 3, "should have 3 system prompts");

    // Extract the stable prefix portion (everything before "# Context" or "# Runtime Context")
    // The stable prefix is Identity + Tools + Skills, which ends before the dynamic sections.
    fn stable_part(prompt: &str) -> &str {
        // Find the start of the first dynamic section
        for marker in ["# Runtime Context", "# Context", "# User Preferences"] {
            if let Some(pos) = prompt.find(marker) {
                return &prompt[..pos];
            }
        }
        prompt
    }

    let stable_0 = stable_part(&prompts[0]);
    let stable_1 = stable_part(&prompts[1]);
    let stable_2 = stable_part(&prompts[2]);

    assert_eq!(
        stable_0, stable_1,
        "stable prefix should be identical between turn 1 and 2"
    );
    assert_eq!(
        stable_1, stable_2,
        "stable prefix should be identical between turn 2 and 3"
    );
}

#[tokio::test]
#[allow(deprecated)]
async fn test_stable_prefix_changes_after_set_tools() {
    let response_events = vec![
        AgentEvent::TurnStart,
        AgentEvent::TextDelta { delta: "OK".into() },
        AgentEvent::TurnEnd {
            stop_reason: StopReason::EndTurn,
            usage: Usage::default(),
        },
    ];

    let (model, captured) =
        CapturingModel::new(vec![response_events.clone(), response_events.clone()]);
    let mut agent = Agent::new(Arc::new(model), ToolRegistry::new());

    // Run first turn
    let _ = agent.run("turn 1".into()).await.unwrap();

    // Change tools — should invalidate cache
    agent.set_tools(vec![ToolDescription {
        name: "new_tool".into(),
        description: "A new tool".into(),
        ..Default::default()
    }]);

    // Run second turn
    let _ = agent.run("turn 2".into()).await.unwrap();

    let prompts = captured.lock().expect("lock poisoned");
    assert_eq!(prompts.len(), 2, "should have 2 system prompts");

    fn stable_part(prompt: &str) -> &str {
        for marker in ["# Runtime Context", "# Context", "# User Preferences"] {
            if let Some(pos) = prompt.find(marker) {
                return &prompt[..pos];
            }
        }
        prompt
    }

    let stable_0 = stable_part(&prompts[0]);
    let stable_1 = stable_part(&prompts[1]);

    assert_ne!(
        stable_0, stable_1,
        "stable prefix should differ after set_tools()"
    );
    assert!(
        stable_1.contains("new_tool"),
        "new stable prefix should contain the new tool"
    );
}

#[tokio::test]
#[allow(deprecated)]
async fn test_stable_prefix_changes_after_set_skill_index() {
    let response_events = vec![
        AgentEvent::TurnStart,
        AgentEvent::TextDelta { delta: "OK".into() },
        AgentEvent::TurnEnd {
            stop_reason: StopReason::EndTurn,
            usage: Usage::default(),
        },
    ];

    let (model, captured) =
        CapturingModel::new(vec![response_events.clone(), response_events.clone()]);
    let mut agent = Agent::new(Arc::new(model), ToolRegistry::new());

    let _ = agent.run("turn 1".into()).await.unwrap();

    agent.set_skill_index(vec![SkillIndex {
        name: "new-skill".into(),
        description: "A new skill".into(),
        triggers: vec!["new".into()],
        estimated_tokens: 0,
    }]);

    let _ = agent.run("turn 2".into()).await.unwrap();

    let prompts = captured.lock().expect("lock poisoned");
    assert_eq!(prompts.len(), 2);

    fn stable_part(prompt: &str) -> &str {
        for marker in ["# Runtime Context", "# Context", "# User Preferences"] {
            if let Some(pos) = prompt.find(marker) {
                return &prompt[..pos];
            }
        }
        prompt
    }

    let stable_0 = stable_part(&prompts[0]);
    let stable_1 = stable_part(&prompts[1]);

    assert_ne!(
        stable_0, stable_1,
        "stable prefix should differ after set_skill_index()"
    );
    assert!(
        stable_1.contains("new-skill"),
        "new stable prefix should contain the new skill"
    );
}

#[test]
fn test_build_stable_prefix_and_dynamic_suffix() {
    use crate::prompt::SystemPromptBuilder;

    let builder = SystemPromptBuilder::new()
        .with_tools(vec![ToolDescription {
            name: "read".into(),
            description: "Read a file".into(),
            ..Default::default()
        }])
        .with_skill_index(vec![SkillIndex {
            name: "test-skill".into(),
            description: "A test skill".into(),
            triggers: vec!["test".into()],
            estimated_tokens: 0,
        }]);

    let stable = builder.build_stable_prefix();
    let dynamic = builder.build_dynamic_suffix();

    // Stable prefix should contain Identity, Tools, Skills
    assert!(stable.contains("# Identity"));
    assert!(stable.contains("# Tools"));
    assert!(stable.contains("## read"));
    assert!(stable.contains("# Skills"));
    assert!(stable.contains("test-skill"));

    // Stable prefix should NOT contain dynamic sections
    assert!(!stable.contains("# Runtime Context"));

    // Dynamic suffix should contain Runtime Context
    assert!(dynamic.contains("# Runtime Context"));
    assert!(dynamic.contains("unix_seconds="));

    // Combined should equal full build
    let combined = if stable.is_empty() {
        dynamic.clone()
    } else if dynamic.is_empty() {
        stable.clone()
    } else {
        format!("{stable}\n{dynamic}")
    };
    let full = builder.build();
    assert_eq!(combined, full);
}
