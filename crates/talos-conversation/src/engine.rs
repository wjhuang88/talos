use std::time::Instant;

use futures::stream;
use talos_core::message::{AgentEvent, MessageToolResult, StopReason, Usage};
use talos_core::tool::ToolProvenance;
use tokio::sync::mpsc;

use crate::types::{
    ChatMessage, MessageRole, MessageSource, MessageStatus, PluginObservation, ScrollbackState,
    SkillDiagnostic, StatusSnapshot, StreamMessage, TipKind, ToolCallDisplay, ToolCallInfo,
    ToolResultDisplay, UiOutput,
};

fn plugin_observation_key(provenance: &ToolProvenance) -> String {
    match provenance {
        ToolProvenance::Native => "native".to_string(),
        ToolProvenance::McpRemote { server } => {
            let server = if server.chars().count() > 24 {
                let truncated: String = server.chars().take(23).collect();
                format!("{truncated}…")
            } else {
                server.clone()
            };
            format!("mcp:{server}")
        }
    }
}

const MOCK_REQUEST_COMMAND: &str = "/mock-request";

struct BuiltinCommandDefinition {
    name: &'static str,
    usage: &'static str,
    description: &'static str,
}

const BUILTIN_COMMANDS: &[BuiltinCommandDefinition] = &[
    BuiltinCommandDefinition {
        name: "/help",
        usage: "/help",
        description: "Show this help",
    },
    BuiltinCommandDefinition {
        name: "/quit",
        usage: "/quit",
        description: "Exit Talos",
    },
    BuiltinCommandDefinition {
        name: "/exit",
        usage: "/exit",
        description: "Exit Talos",
    },
    BuiltinCommandDefinition {
        name: "/status",
        usage: "/status",
        description: "Show session info",
    },
    BuiltinCommandDefinition {
        name: "/plugins",
        usage: "/plugins",
        description: "List observed tool provenance",
    },
    BuiltinCommandDefinition {
        name: "/skills",
        usage: "/skills",
        description: "List available runtime skills",
    },
];

pub struct ConversationEngine {
    pub(crate) messages: Vec<ChatMessage>,
    pub(crate) current_turn_text: String,
    pub(crate) steering_queue: Vec<String>,
    pub(crate) followup_queue: Vec<String>,
    pub(crate) usage: Usage,
    pub(crate) model_name: String,
    pub(crate) branch_id: Option<String>,
    pub(crate) plugin_observations: Vec<PluginObservation>,
    pub(crate) skills: Vec<SkillDiagnostic>,
    pub(crate) scrollback: ScrollbackState,
    pub(crate) is_processing: bool,
    last_flushed_message: usize,
    stream_tx: Option<mpsc::UnboundedSender<String>>,
}

impl ConversationEngine {
    /// Slash command names currently exposed by help and completion.
    ///
    /// Retained as a compatibility view while CMD-001 evolves the registry into
    /// public command definitions with tool-backed metadata resolution.
    pub const SLASH_COMMANDS: &'static [&'static str] =
        &["/help", "/quit", "/exit", "/status", "/plugins", "/skills"];

    pub fn new(model_name: String) -> Self {
        Self {
            messages: Vec::new(),
            current_turn_text: String::new(),
            steering_queue: Vec::new(),
            followup_queue: Vec::new(),
            usage: Usage::default(),
            model_name,
            branch_id: None,
            plugin_observations: Vec::new(),
            skills: Vec::new(),
            scrollback: ScrollbackState::default(),
            is_processing: false,
            last_flushed_message: 0,
            stream_tx: None,
        }
    }

    pub fn with_skills(mut self, skills: Vec<SkillDiagnostic>) -> Self {
        self.skills = skills;
        self
    }

    pub fn status_snapshot(&self) -> StatusSnapshot {
        StatusSnapshot {
            model_name: self.model_name.clone(),
            usage: self.usage.clone(),
            branch_id: self.branch_id.clone(),
            steering_count: self.steering_queue.len(),
            followup_count: self.followup_queue.len(),
            is_processing: self.is_processing,
        }
    }

    pub fn is_processing(&self) -> bool {
        self.is_processing
    }

    pub fn start_user_message(&mut self, msg: &str) -> Vec<UiOutput> {
        self.is_processing = true;
        self.handle_user_message(msg)
    }

    pub fn enqueue_steering(&mut self, msg: String) -> Vec<UiOutput> {
        self.steering_queue.push(msg);
        vec![
            UiOutput::Tip {
                text: "Message queued and will send after current turn.".into(),
                kind: TipKind::QueueHint,
            },
            UiOutput::Status(self.status_snapshot()),
        ]
    }

    pub fn cancel_turn(&mut self) -> Vec<UiOutput> {
        self.close_stream();
        self.is_processing = false;
        self.current_turn_text.clear();
        vec![
            UiOutput::Tip {
                text: "Turn cancellation requested.".into(),
                kind: TipKind::ExitHint,
            },
            UiOutput::Status(self.status_snapshot()),
        ]
    }

    pub fn handle_agent_event(&mut self, event: &AgentEvent) -> Vec<UiOutput> {
        let mut outputs = Vec::new();

        match event {
            AgentEvent::TurnStart => {
                self.is_processing = true;
                self.current_turn_text.clear();

                let (tx, rx) = mpsc::unbounded_channel::<String>();
                self.stream_tx = Some(tx);
                outputs.push(UiOutput::Stream(StreamMessage {
                    source: MessageSource::Assistant,
                    stream: Box::pin(tokio_stream::wrappers::UnboundedReceiverStream::new(rx)),
                }));
                outputs.push(UiOutput::Status(self.status_snapshot()));
            }
            AgentEvent::TextDelta { delta } => {
                self.current_turn_text.push_str(delta);
                if let Some(ref tx) = self.stream_tx {
                    let _ = tx.send(delta.clone());
                }
                outputs.push(UiOutput::Status(self.status_snapshot()));
            }
            AgentEvent::ToolCallStarted { name } => {
                self.close_stream();
                outputs.push(UiOutput::ToolCallStarted {
                    name: name.to_string(),
                });
            }
            AgentEvent::ToolCall {
                call,
                provenance,
                summary_fields,
            } => {
                self.close_stream();
                self.record_provenance(provenance);
                self.messages.push(ChatMessage {
                    role: MessageRole::Assistant,
                    status: MessageStatus::Completed,
                    content: String::new(),
                    tool_call: Some(ToolCallInfo {
                        tool_name: call.name.clone(),
                        arguments: serde_json::to_string_pretty(&call.input)
                            .unwrap_or_else(|_| call.input.to_string()),
                        provenance: provenance.clone(),
                        result: None,
                    }),
                    created_at: Instant::now(),
                });
                outputs.push(UiOutput::ToolCall(ToolCallDisplay {
                    tool_name: call.name.clone(),
                    arguments: call.input.clone(),
                    provenance: provenance.clone(),
                    summary_fields: summary_fields.clone(),
                }));
            }
            AgentEvent::ToolResult { result } => {
                self.close_stream();
                let tool_name = self.set_tool_result(result);
                outputs.push(UiOutput::ToolResult(ToolResultDisplay {
                    tool_name,
                    is_error: result.is_error,
                    content: result.content.clone(),
                }));
            }
            AgentEvent::TurnEnd { usage, stop_reason } => {
                self.close_stream();
                if matches!(stop_reason, StopReason::EndTurn) {
                    self.is_processing = false;
                }
                self.finalize_turn();
                self.usage = usage.clone();
                self.last_flushed_message = self.messages.len();

                outputs.push(UiOutput::Status(self.status_snapshot()));
            }
            AgentEvent::Error { message } => {
                self.close_stream();
                self.is_processing = false;
                self.current_turn_text.clear();

                outputs.push(UiOutput::Tip {
                    text: message.clone(),
                    kind: TipKind::Error,
                });

                let (tx, rx) = mpsc::unbounded_channel::<String>();
                let text = format!("[Error] {message}");
                let _ = tx.send(text);
                drop(tx);
                outputs.push(UiOutput::Stream(StreamMessage {
                    source: MessageSource::Error,
                    stream: Box::pin(tokio_stream::wrappers::UnboundedReceiverStream::new(rx)),
                }));

                self.messages.push(ChatMessage {
                    role: MessageRole::Error,
                    status: MessageStatus::Completed,
                    content: format!("[Error] {message}"),
                    tool_call: None,
                    created_at: Instant::now(),
                });
                self.last_flushed_message = self.messages.len();

                outputs.push(UiOutput::Status(self.status_snapshot()));
            }
            _ => {}
        }

        outputs
    }

    pub fn handle_user_message(&mut self, msg: &str) -> Vec<UiOutput> {
        let msg_owned = msg.to_string();
        self.messages.push(ChatMessage {
            role: MessageRole::User,
            status: MessageStatus::Completed,
            content: format!("{msg_owned}\n"),
            tool_call: None,
            created_at: Instant::now(),
        });
        self.last_flushed_message = self.messages.len();

        vec![UiOutput::Stream(StreamMessage {
            source: MessageSource::User,
            stream: Box::pin(stream::once(async move { msg_owned })),
        })]
    }

    pub fn handle_slash_command(&mut self, input: &str) -> Vec<UiOutput> {
        let parts: Vec<&str> = input.splitn(2, ' ').collect();
        let cmd = parts[0];
        let _arg = parts.get(1).copied().unwrap_or("");
        let mut outputs = Vec::new();

        match cmd {
            "/help" => {
                let mut text = String::from("[System] Available commands:\n");
                for command in BUILTIN_COMMANDS {
                    text.push_str(&format!(
                        "[System]   {:<10} — {}\n",
                        command.usage, command.description
                    ));
                }
                outputs.push(UiOutput::Stream(StreamMessage {
                    source: MessageSource::System,
                    stream: Box::pin(stream::once(async move { text })),
                }));
            }
            "/quit" | "/exit" => {
                outputs.push(UiOutput::Exit);
            }
            "/status" => {
                let text = format!(
                    "[System] Model: {} | Input: {} | Output: {} tokens\n",
                    self.model_name, self.usage.input_tokens, self.usage.output_tokens,
                );
                outputs.push(UiOutput::Stream(StreamMessage {
                    source: MessageSource::System,
                    stream: Box::pin(stream::once(async move { text })),
                }));
            }
            "/plugins" => {
                outputs.extend(self.handle_plugins_command());
            }
            "/skills" => {
                outputs.extend(self.handle_skills_command());
            }
            _ => {
                let text =
                    format!("[Error] Unknown command: {cmd}. Type /help for available commands.\n");
                outputs.push(UiOutput::Stream(StreamMessage {
                    source: MessageSource::Error,
                    stream: Box::pin(stream::once(async move { text })),
                }));
            }
        }

        outputs
    }

    fn handle_plugins_command(&mut self) -> Vec<UiOutput> {
        if self.plugin_observations.is_empty() {
            let text = "[System] No tool provenance observed yet.\n".to_string();
            return vec![UiOutput::Stream(StreamMessage {
                source: MessageSource::System,
                stream: Box::pin(stream::once(async move { text })),
            })];
        }
        let mut text = String::from("[System] Observed tool provenance (this session):\n");
        for entry in &self.plugin_observations {
            text.push_str(&format!(
                "[System]   {} ({} call{})\n",
                entry.key,
                entry.count,
                if entry.count == 1 { "" } else { "s" },
            ));
        }
        vec![UiOutput::Stream(StreamMessage {
            source: MessageSource::System,
            stream: Box::pin(stream::once(async move { text })),
        })]
    }

    fn handle_skills_command(&mut self) -> Vec<UiOutput> {
        if self.skills.is_empty() {
            let text = "[System] No skills available.\n".to_string();
            return vec![UiOutput::Stream(StreamMessage {
                source: MessageSource::System,
                stream: Box::pin(stream::once(async move { text })),
            })];
        }

        let mut text = String::from("[System] Available skills (Level 0 metadata):\n");
        for skill in &self.skills {
            let state = if skill.active { "active" } else { "available" };
            text.push_str(&format!(
                "[System]   {} ({state}) — {}\n",
                skill.name, skill.description,
            ));
        }
        text.push_str(
            "[System] Level 1 skill bodies and Level 2 references are gated until explicit activation lands.\n",
        );
        vec![UiOutput::Stream(StreamMessage {
            source: MessageSource::System,
            stream: Box::pin(stream::once(async move { text })),
        })]
    }

    pub fn drain_steering_queue(&mut self) -> Option<String> {
        if self.steering_queue.is_empty() {
            None
        } else {
            Some(self.steering_queue.remove(0))
        }
    }

    pub fn is_model_passthrough_slash_command(input: &str) -> bool {
        let trimmed = input.trim_start();
        if trimmed == MOCK_REQUEST_COMMAND {
            return true;
        }

        trimmed
            .strip_prefix(MOCK_REQUEST_COMMAND)
            .and_then(|rest| rest.chars().next())
            .is_some_and(char::is_whitespace)
    }

    pub fn complete_slash_command(&self, input: &str) -> Vec<&str> {
        BUILTIN_COMMANDS
            .iter()
            .map(|command| command.name)
            .filter(|name| name.starts_with(input))
            .collect()
    }

    pub fn last_assistant_text(&self) -> Option<String> {
        self.messages.iter().rev().find_map(|msg| {
            if msg.role == MessageRole::Assistant
                && msg.tool_call.is_none()
                && !msg.content.is_empty()
            {
                Some(msg.content.clone())
            } else {
                None
            }
        })
    }

    pub fn transcript_plain_text(&self) -> String {
        let mut out = String::new();
        for msg in &self.messages {
            Self::append_message_plain(&mut out, msg);
        }
        out
    }

    pub fn transcript_markdown(&self) -> String {
        let mut out = String::new();
        for msg in &self.messages {
            Self::append_message_markdown(&mut out, msg);
        }
        out
    }

    pub fn append_message_plain(out: &mut String, msg: &ChatMessage) {
        if !msg.content.is_empty() {
            out.push_str(&msg.content);
            if !msg.content.ends_with('\n') {
                out.push('\n');
            }
        }
        if let Some(ref tool_call) = msg.tool_call {
            let marker = plugin_observation_key(&tool_call.provenance);
            out.push_str(&format!("▸ {} [{marker}]\n", tool_call.tool_name));
            out.push_str(&format!("  {}\n", tool_call.arguments));
            if let Some(ref result) = tool_call.result {
                let icon = if result.is_error { "✗" } else { "✓" };
                let content = if result.content.is_empty() {
                    "(no output)"
                } else {
                    &result.content
                };
                out.push_str(&format!("  {icon} {content}\n"));
            }
        }
    }

    fn append_message_markdown(out: &mut String, msg: &ChatMessage) {
        if !msg.content.is_empty() {
            out.push_str(&msg.content);
            if !msg.content.ends_with('\n') {
                out.push('\n');
            }
        }
        if let Some(ref tool_call) = msg.tool_call {
            let marker = plugin_observation_key(&tool_call.provenance);
            out.push_str(&format!("### `▸ {} [{marker}]`\n\n", tool_call.tool_name));
            out.push_str("```json\n");
            out.push_str(&tool_call.arguments);
            out.push_str("\n```\n");
            if let Some(ref result) = tool_call.result {
                let label = if result.is_error { "Error" } else { "Result" };
                out.push_str(&format!("\n**{label}:**\n\n"));
                out.push_str("```\n");
                out.push_str(&result.content);
                out.push_str("\n```\n");
            }
        }
    }

    fn close_stream(&mut self) {
        drop(self.stream_tx.take());
    }

    fn finalize_turn(&mut self) {
        self.scrollback.scrolled_line_count = 0;
        if self.current_turn_text.is_empty() {
            return;
        }
        let text = std::mem::take(&mut self.current_turn_text);
        self.messages.push(ChatMessage {
            role: MessageRole::Assistant,
            status: MessageStatus::Completed,
            content: text,
            tool_call: None,
            created_at: Instant::now(),
        });
    }

    fn set_tool_result(&mut self, result: &MessageToolResult) -> Option<String> {
        for msg in self.messages.iter_mut().rev() {
            if let Some(ref mut tool_call) = msg.tool_call
                && tool_call.result.is_none()
            {
                let tool_name = tool_call.tool_name.clone();
                tool_call.result = Some(result.clone());
                return Some(tool_name);
            }
        }
        None
    }

    fn record_provenance(&mut self, provenance: &ToolProvenance) {
        let key = plugin_observation_key(provenance);
        if let Some(entry) = self.plugin_observations.iter_mut().find(|e| e.key == key) {
            entry.count += 1;
        } else {
            self.plugin_observations
                .push(PluginObservation { key, count: 1 });
        }
    }
}
