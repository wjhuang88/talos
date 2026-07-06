use std::path::PathBuf;
use std::time::Instant;

use futures::stream;
use talos_core::message::{AgentEvent, MessageToolResult, StopReason, Usage};
use talos_core::tool::ToolProvenance;
use tokio::sync::mpsc;

use crate::command_registry::{MOCK_REQUEST_COMMAND, command_registry};
use crate::types::{
    ChatMessage, CopyScope, McpServerDiagnostic, MessageRole, MessageSource, MessageStatus,
    ModelSwitchRequest, PluginObservation, ScrollbackState, SessionDeleteRequest,
    SessionForkRequest, SessionNewRequest, SessionResumeRequest, SkillCommandRequest,
    SkillDiagnostic, StatusSnapshot, StreamMessage, TipKind, TodoCommandAction, TodoCommandRequest,
    TodoExportFormat, ToolCallDisplay, ToolCallInfo, ToolResultDisplay, TurnPhase, UiOutput,
};

fn is_timeout_error(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("timeout") || lower.contains("timed out")
}

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
        ToolProvenance::Plugin {
            name,
            version,
            carrier,
        } => {
            let name_display = if name.chars().count() > 24 {
                let truncated: String = name.chars().take(23).collect();
                format!("{truncated}…")
            } else {
                name.clone()
            };
            format!("plugin:{name_display}@{version}/{carrier}")
        }
    }
}

fn parse_todo_command(arg: &str) -> Result<TodoCommandRequest, String> {
    let mut tokens = arg.split_whitespace();
    let subcommand = tokens.next().unwrap_or("list");
    let mut request = TodoCommandRequest {
        action: match subcommand {
            "" | "list" => TodoCommandAction::List,
            "show" => {
                let id = tokens
                    .next()
                    .ok_or_else(|| "Usage: /todo show <id>".to_string())?;
                TodoCommandAction::Show { id: id.to_string() }
            }
            "stats" => TodoCommandAction::Stats,
            "export" => {
                let format = match tokens.next() {
                    None | Some("markdown") | Some("md") => TodoExportFormat::Markdown,
                    Some("json") => TodoExportFormat::Json,
                    Some(other) => {
                        return Err(format!("Unknown todo export format: {other}"));
                    }
                };
                TodoCommandAction::Export { format }
            }
            other if other.starts_with("--") => TodoCommandAction::List,
            other => {
                return Err(format!(
                    "Unknown todo command: {other}. Usage: /todo [list|show|stats|export]"
                ));
            }
        },
        status_filter: None,
        priority_filter: None,
        tag_filter: None,
        sort: None,
    };

    let mut pending = if subcommand.starts_with("--") {
        Some(subcommand)
    } else {
        None
    };
    while let Some(token) = pending.take().or_else(|| tokens.next()) {
        match token {
            "--status" => {
                request.status_filter = Some(
                    tokens
                        .next()
                        .ok_or_else(|| "Missing value for --status".to_string())?
                        .to_string(),
                );
            }
            "--priority" => {
                request.priority_filter = Some(
                    tokens
                        .next()
                        .ok_or_else(|| "Missing value for --priority".to_string())?
                        .to_string(),
                );
            }
            "--tag" => {
                request.tag_filter = Some(
                    tokens
                        .next()
                        .ok_or_else(|| "Missing value for --tag".to_string())?
                        .to_string(),
                );
            }
            "--sort" => {
                request.sort = Some(
                    tokens
                        .next()
                        .ok_or_else(|| "Missing value for --sort".to_string())?
                        .to_string(),
                );
            }
            other => return Err(format!("Unknown todo option: {other}")),
        }
    }

    Ok(request)
}

pub struct ConversationEngine {
    pub(crate) messages: Vec<ChatMessage>,
    pub(crate) current_turn_text: String,
    pub(crate) steering_queue: Vec<String>,
    pub(crate) followup_queue: Vec<String>,
    pub(crate) usage: Usage,
    pub(crate) current_thinking_text: String,
    pub(crate) model_name: String,
    pub(crate) provider_name: String,
    pub(crate) branch_id: Option<String>,
    pub(crate) plugin_observations: Vec<PluginObservation>,
    pub(crate) mcp_servers: Vec<McpServerDiagnostic>,
    pub(crate) skills: Vec<SkillDiagnostic>,
    pub(crate) scrollback: ScrollbackState,
    pub(crate) is_processing: bool,
    pub(crate) current_phase: Option<TurnPhase>,
    pub(crate) context_limit: Option<u32>,
    pub(crate) input_price_per_million: Option<f64>,
    pub(crate) output_price_per_million: Option<f64>,
    pub(crate) workspace_root: Option<PathBuf>,
    last_flushed_message: usize,
    stream_tx: Option<mpsc::UnboundedSender<String>>,
}

impl ConversationEngine {
    /// Slash command names currently exposed by help and completion.
    ///
    /// Derived from the shared `CommandRegistry` so help, completion, and TUI-010
    /// always reflect the same executable command set.
    pub fn slash_commands() -> Vec<&'static str> {
        command_registry().available_names()
    }

    pub fn new(model_name: String, provider_name: String) -> Self {
        Self {
            messages: Vec::new(),
            current_turn_text: String::new(),
            steering_queue: Vec::new(),
            followup_queue: Vec::new(),
            usage: Usage::default(),
            current_thinking_text: String::new(),
            model_name,
            provider_name,
            branch_id: None,
            plugin_observations: Vec::new(),
            mcp_servers: Vec::new(),
            skills: Vec::new(),
            scrollback: ScrollbackState::default(),
            is_processing: false,
            current_phase: None,
            context_limit: None,
            input_price_per_million: None,
            output_price_per_million: None,
            workspace_root: None,
            last_flushed_message: 0,
            stream_tx: None,
        }
    }

    pub fn with_workspace_root(mut self, workspace_root: PathBuf) -> Self {
        self.workspace_root = Some(workspace_root);
        self
    }

    pub fn with_skills(mut self, skills: Vec<SkillDiagnostic>) -> Self {
        self.skills = skills;
        self
    }

    pub fn set_skills(&mut self, skills: Vec<SkillDiagnostic>) {
        self.skills = skills;
    }

    pub fn with_mcp_servers(mut self, servers: Vec<McpServerDiagnostic>) -> Self {
        self.mcp_servers = servers;
        self
    }

    pub fn status_snapshot(&self) -> StatusSnapshot {
        StatusSnapshot {
            model_name: self.model_name.clone(),
            provider: self.provider_name.clone(),
            workspace_path: String::new(),
            usage: self.usage.clone(),
            branch_id: self.branch_id.clone(),
            steering_count: self.steering_queue.len(),
            followup_count: self.followup_queue.len(),
            is_processing: self.is_processing,
            phase: self.current_phase.clone(),
            context_limit: self.context_limit,
            input_price_per_million: self.input_price_per_million,
            output_price_per_million: self.output_price_per_million,
        }
    }

    pub fn set_model_info(&mut self, info: &crate::types::ModelInfo) {
        self.model_name = info.model_name.clone();
        self.provider_name = info.provider.clone();
        self.context_limit = info.context_limit;
        self.input_price_per_million = info.input_price_per_million;
        self.output_price_per_million = info.output_price_per_million;
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
        self.current_phase = Some(TurnPhase::Cancelled);
        self.current_turn_text.clear();
        let had_thinking = !self.current_thinking_text.is_empty();
        self.current_thinking_text.clear();
        let mut outputs = Vec::new();
        if had_thinking {
            outputs.push(UiOutput::ThinkingPreview { text: None });
        }
        outputs.extend([
            UiOutput::Tip {
                text: "Turn cancellation requested.".into(),
                kind: TipKind::ExitHint,
            },
            UiOutput::Status(self.status_snapshot()),
        ]);
        outputs
    }

    pub fn handle_agent_event(&mut self, event: &AgentEvent) -> Vec<UiOutput> {
        let mut outputs = Vec::new();

        match event {
            AgentEvent::TurnStart => {
                self.is_processing = true;
                self.current_phase = Some(TurnPhase::Connecting);
                self.current_turn_text.clear();
                self.current_thinking_text.clear();

                let (tx, rx) = mpsc::unbounded_channel::<String>();
                self.stream_tx = Some(tx);
                outputs.push(UiOutput::Stream(StreamMessage {
                    source: MessageSource::Assistant,
                    stream: Box::pin(tokio_stream::wrappers::UnboundedReceiverStream::new(rx)),
                }));
                outputs.push(UiOutput::Status(self.status_snapshot()));
            }
            AgentEvent::TextDelta { delta } => {
                self.current_phase = Some(TurnPhase::Generating);
                self.current_turn_text.push_str(delta);
                if let Some(ref tx) = self.stream_tx {
                    let _ = tx.send(delta.clone());
                }
                outputs.push(UiOutput::Status(self.status_snapshot()));
            }
            AgentEvent::ThinkingDelta { delta } => {
                self.current_phase = Some(TurnPhase::Thinking);
                self.current_thinking_text.push_str(delta);
                outputs.push(UiOutput::ThinkingPreview {
                    text: Some(self.current_thinking_text.clone()),
                });
                outputs.push(UiOutput::Status(self.status_snapshot()));
            }
            AgentEvent::ToolCallStarted { name } => {
                self.current_phase = Some(TurnPhase::RunningTool { name: name.clone() });
                self.close_stream();
                outputs.push(UiOutput::ToolCallStarted {
                    name: name.to_string(),
                });
                outputs.push(UiOutput::Status(self.status_snapshot()));
            }
            AgentEvent::ToolCall {
                call,
                provenance,
                summary_fields,
            } => {
                self.current_phase = Some(TurnPhase::RunningTool {
                    name: call.name.clone(),
                });
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
                outputs.push(UiOutput::Status(self.status_snapshot()));
            }
            AgentEvent::ToolResult { result } => {
                self.close_stream();
                let tool_name = self.set_tool_result(result);
                outputs.push(UiOutput::ToolResult(ToolResultDisplay {
                    tool_name,
                    is_error: result.is_error,
                    content: result.content.clone(),
                }));
                self.current_phase = Some(TurnPhase::Generating);
                outputs.push(UiOutput::Status(self.status_snapshot()));
            }
            AgentEvent::TurnEnd { usage, stop_reason } => {
                self.close_stream();
                if matches!(stop_reason, StopReason::EndTurn) {
                    self.is_processing = false;
                }
                self.current_phase = None;
                self.finalize_turn();
                self.usage = usage.clone();
                self.last_flushed_message = self.messages.len();
                let had_thinking = !self.current_thinking_text.is_empty();
                self.current_thinking_text.clear();

                if had_thinking {
                    outputs.push(UiOutput::ThinkingPreview { text: None });
                }
                outputs.push(UiOutput::Status(self.status_snapshot()));
            }
            AgentEvent::Error { message } => {
                self.close_stream();
                self.is_processing = false;
                self.current_phase = Some(if is_timeout_error(message) {
                    TurnPhase::TimedOut
                } else {
                    TurnPhase::Failed
                });
                self.current_turn_text.clear();
                let had_thinking = !self.current_thinking_text.is_empty();
                self.current_thinking_text.clear();

                if had_thinking {
                    outputs.push(UiOutput::ThinkingPreview { text: None });
                }
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
            AgentEvent::ReasoningComplete { .. } => {}
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
        let arg = parts.get(1).copied().unwrap_or("");
        let mut outputs = Vec::new();

        match cmd {
            "/help" => {
                let mut text = String::from("[System] Available commands:\n");
                for command in command_registry().available_commands() {
                    text.push_str(&format!(
                        "[System]   {:<20} — {}\n",
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
                let text = "[System] /plugins is reserved for future plugin packages.\n[System] Use /mcp to inspect MCP server status and tool provenance.\n".to_string();
                outputs.push(UiOutput::Stream(StreamMessage {
                    source: MessageSource::System,
                    stream: Box::pin(stream::once(async move { text })),
                }));
            }
            "/mcp" => {
                outputs.extend(self.handle_mcp_command());
            }
            "/hooks" => {
                outputs.extend(self.handle_hooks_command());
            }
            "/skills" => {
                outputs.extend(self.handle_skills_command(arg));
            }
            "/copy" => {
                outputs.extend(self.handle_copy_command(arg));
            }
            "/export" => {
                outputs.extend(self.handle_export_command(arg));
            }
            "/new" => {
                if self.is_processing {
                    let text = "[System] Cannot start a new session while a turn is active. Wait for the current turn to finish.\n".to_string();
                    outputs.push(UiOutput::Stream(StreamMessage {
                        source: MessageSource::System,
                        stream: Box::pin(stream::once(async move { text })),
                    }));
                } else {
                    outputs.push(UiOutput::SessionNew(SessionNewRequest));
                }
            }
            "/resume" => {
                if self.is_processing {
                    let text = "[System] Cannot resume a session while a turn is active. Wait for the current turn to finish.\n".to_string();
                    outputs.push(UiOutput::Stream(StreamMessage {
                        source: MessageSource::System,
                        stream: Box::pin(stream::once(async move { text })),
                    }));
                } else {
                    let session_id = if arg.is_empty() {
                        None
                    } else {
                        Some(arg.to_string())
                    };
                    outputs.push(UiOutput::SessionResume(SessionResumeRequest { session_id }));
                }
            }
            "/fork" => {
                if self.is_processing {
                    let text = "[System] Cannot fork a session while a turn is active. Wait for the current turn to finish.\n".to_string();
                    outputs.push(UiOutput::Stream(StreamMessage {
                        source: MessageSource::System,
                        stream: Box::pin(stream::once(async move { text })),
                    }));
                } else {
                    outputs.push(UiOutput::SessionFork(SessionForkRequest));
                }
            }
            "/delete" => {
                if self.is_processing {
                    let text = "[System] Cannot delete a session while a turn is active. Wait for the current turn to finish.\n".to_string();
                    outputs.push(UiOutput::Stream(StreamMessage {
                        source: MessageSource::System,
                        stream: Box::pin(stream::once(async move { text })),
                    }));
                } else if arg.is_empty() {
                    outputs.push(UiOutput::SessionDelete(SessionDeleteRequest {
                        selection: None,
                    }));
                } else {
                    outputs.push(UiOutput::SessionDelete(SessionDeleteRequest {
                        selection: Some(arg.to_string()),
                    }));
                }
            }
            "/model" => {
                if self.is_processing {
                    let text = "[System] Cannot switch models while a turn is active. Wait for the current turn to finish.\n".to_string();
                    outputs.push(UiOutput::Stream(StreamMessage {
                        source: MessageSource::System,
                        stream: Box::pin(stream::once(async move { text })),
                    }));
                } else {
                    outputs.push(UiOutput::ModelSwitchRequest(ModelSwitchRequest {
                        model_id: arg.to_string(),
                        provider_needs_credential: false,
                    }));
                }
            }
            "/connect" => {
                outputs.push(UiOutput::ConnectProviderRequest {
                    provider: arg.to_string(),
                });
            }
            "/todo" => {
                outputs.extend(self.handle_todo_command(arg));
            }
            "/agile" => {
                outputs.extend(self.handle_agile_command(arg));
            }
            "/validate" => {
                outputs.extend(self.handle_validate_command(arg));
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

    fn handle_todo_command(&self, arg: &str) -> Vec<UiOutput> {
        match parse_todo_command(arg) {
            Ok(request) => vec![UiOutput::TodoCommand(request)],
            Err(message) => vec![UiOutput::Stream(StreamMessage {
                source: MessageSource::Error,
                stream: Box::pin(stream::once(async move { format!("[Error] {message}\n") })),
            })],
        }
    }

    fn handle_agile_command(&self, _arg: &str) -> Vec<UiOutput> {
        let Some(ref ws) = self.workspace_root else {
            return vec![UiOutput::Stream(StreamMessage {
                source: MessageSource::System,
                stream: Box::pin(stream::once(async move {
                    "[System] /agile is unavailable — no workspace path set.\n".to_string()
                })),
            })];
        };
        let text = crate::governance_summary::format_governance_summary(ws);
        vec![UiOutput::Stream(StreamMessage {
            source: MessageSource::System,
            stream: Box::pin(stream::once(async move { text })),
        })]
    }

    fn handle_validate_command(&self, arg: &str) -> Vec<UiOutput> {
        let profile = match arg.trim() {
            "" | "governance" => crate::ValidationProfile::Governance,
            other => {
                let text = format!(
                    "[Error] Unsupported internal validation profile: {other}. Usage: /validate [governance]\n"
                );
                return vec![UiOutput::Stream(StreamMessage {
                    source: MessageSource::Error,
                    stream: Box::pin(stream::once(async move { text })),
                })];
            }
        };
        let Some(ref ws) = self.workspace_root else {
            return vec![UiOutput::Stream(StreamMessage {
                source: MessageSource::System,
                stream: Box::pin(stream::once(async move {
                    "[System] /validate is unavailable — no workspace path set.\n".to_string()
                })),
            })];
        };

        let plan = crate::collect_validation_plan(ws, profile);
        let evidence = crate::run_validation_plan(ws, plan);
        let text = crate::render_text_evidence(&evidence);
        vec![UiOutput::Stream(StreamMessage {
            source: MessageSource::System,
            stream: Box::pin(stream::once(async move { text })),
        })]
    }

    fn handle_copy_command(&self, scope: &str) -> Vec<UiOutput> {
        let (text, scope_enum, label) = match scope {
            "last" => {
                let content = self
                    .last_assistant_text()
                    .unwrap_or_else(|| "(no assistant messages yet)".to_string());
                (content, CopyScope::Last, "last assistant message")
            }
            "all" => {
                let content = self.transcript_plain_text();
                if content.is_empty() {
                    ("(empty transcript)".to_string(), CopyScope::All, "all")
                } else {
                    (content, CopyScope::All, "full transcript")
                }
            }
            _ => {
                let hint = "[Error] Usage: /copy last | /copy all\n".to_string();
                return vec![UiOutput::Stream(StreamMessage {
                    source: MessageSource::Error,
                    stream: Box::pin(stream::once(async move { hint })),
                })];
            }
        };

        let confirm = format!("[System] Copying {label} to clipboard…\n");
        let mut outputs = vec![UiOutput::Stream(StreamMessage {
            source: MessageSource::System,
            stream: Box::pin(stream::once(async move { confirm })),
        })];
        outputs.push(UiOutput::CopyToClipboard {
            text,
            scope: scope_enum,
        });
        outputs
    }

    fn handle_export_command(&self, path_arg: &str) -> Vec<UiOutput> {
        let path = path_arg.trim();
        if path.is_empty() {
            let hint =
                "[Error] Usage: /export <path>\nExample: /export transcript.md\n".to_string();
            return vec![UiOutput::Stream(StreamMessage {
                source: MessageSource::Error,
                stream: Box::pin(stream::once(async move { hint })),
            })];
        }

        let content = self.transcript_plain_text();
        if content.is_empty() {
            let msg = "[System] Transcript is empty — nothing to export.\n".to_string();
            return vec![UiOutput::Stream(StreamMessage {
                source: MessageSource::System,
                stream: Box::pin(stream::once(async move { msg })),
            })];
        }

        let confirm = format!("[System] Exporting transcript to {path}…\n");
        let mut outputs = vec![UiOutput::Stream(StreamMessage {
            source: MessageSource::System,
            stream: Box::pin(stream::once(async move { confirm })),
        })];
        outputs.push(UiOutput::ExportToFile {
            path: PathBuf::from(path),
            content,
        });
        outputs
    }

    fn handle_mcp_command(&mut self) -> Vec<UiOutput> {
        if self.mcp_servers.is_empty() && self.plugin_observations.is_empty() {
            let text = "[System] No MCP servers configured and no tool provenance observed yet.\n"
                .to_string();
            return vec![UiOutput::Stream(StreamMessage {
                source: MessageSource::System,
                stream: Box::pin(stream::once(async move { text })),
            })];
        }
        let mut text = String::new();
        if !self.mcp_servers.is_empty() {
            text.push_str("[System] MCP servers (startup snapshot):\n");
            for server in &self.mcp_servers {
                if server.connected {
                    text.push_str(&format!(
                        "[System]   {} (connected, {} tool{})\n",
                        server.name,
                        server.tool_count,
                        if server.tool_count == 1 { "" } else { "s" },
                    ));
                } else {
                    let error = server.error.as_deref().unwrap_or("unavailable");
                    text.push_str(&format!(
                        "[System]   {} (unavailable: {error})\n",
                        server.name
                    ));
                }
            }
        }
        if !self.plugin_observations.is_empty() {
            text.push_str("[System] Observed tool provenance (this session):\n");
            for entry in &self.plugin_observations {
                text.push_str(&format!(
                    "[System]   {} ({} call{})\n",
                    entry.key,
                    entry.count,
                    if entry.count == 1 { "" } else { "s" },
                ));
            }
        }
        vec![UiOutput::Stream(StreamMessage {
            source: MessageSource::System,
            stream: Box::pin(stream::once(async move { text })),
        })]
    }

    fn handle_hooks_command(&self) -> Vec<UiOutput> {
        let mut text = String::new();
        text.push_str("[System] Hooks diagnostics:\n");
        text.push_str("[System]   config-introduced hooks: not enabled\n");
        text.push_str("[System]   executable hook carriers: disabled\n");
        text.push_str("[System]   builtin hook event catalog:\n");
        for kind in talos_plugin::ALL_HOOK_EVENT_KINDS {
            text.push_str(&format!("[System]     {kind}\n"));
        }
        vec![UiOutput::Stream(StreamMessage {
            source: MessageSource::System,
            stream: Box::pin(stream::once(async move { text })),
        })]
    }

    fn handle_skills_command(&mut self, arg: &str) -> Vec<UiOutput> {
        let mut parts = arg.split_whitespace();
        match parts.next() {
            Some("activate") => {
                if self.is_processing {
                    let text = "[System] Cannot activate a skill while a turn is active. Wait for the current turn to finish.\n".to_string();
                    return vec![UiOutput::Stream(StreamMessage {
                        source: MessageSource::System,
                        stream: Box::pin(stream::once(async move { text })),
                    })];
                }
                let name = parts.collect::<Vec<_>>().join(" ");
                if name.trim().is_empty() {
                    let text = "[Error] Usage: /skills activate <name>\n".to_string();
                    return vec![UiOutput::Stream(StreamMessage {
                        source: MessageSource::Error,
                        stream: Box::pin(stream::once(async move { text })),
                    })];
                }
                return vec![UiOutput::SkillCommand(SkillCommandRequest::Activate {
                    name,
                })];
            }
            Some("reference") => {
                if self.is_processing {
                    let text = "[System] Cannot load a skill reference while a turn is active. Wait for the current turn to finish.\n".to_string();
                    return vec![UiOutput::Stream(StreamMessage {
                        source: MessageSource::System,
                        stream: Box::pin(stream::once(async move { text })),
                    })];
                }
                let path = parts.collect::<Vec<_>>().join(" ");
                if path.trim().is_empty() {
                    let text = "[Error] Usage: /skills reference <relative-path>\n".to_string();
                    return vec![UiOutput::Stream(StreamMessage {
                        source: MessageSource::Error,
                        stream: Box::pin(stream::once(async move { text })),
                    })];
                }
                return vec![UiOutput::SkillCommand(SkillCommandRequest::Reference {
                    path,
                })];
            }
            Some(other) => {
                let text = format!(
                    "[Error] Unknown /skills action: {other}. Usage: /skills [activate <name> | reference <path>]\n"
                );
                return vec![UiOutput::Stream(StreamMessage {
                    source: MessageSource::Error,
                    stream: Box::pin(stream::once(async move { text })),
                })];
            }
            None => {}
        }

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
                "[System]   {} ({source}) ({state}) — {}\n",
                skill.name,
                skill.description,
                source = skill.source,
            ));
        }
        text.push_str(
            "[System] Use /skills activate <name> to load one Skill body, then /skills reference <relative-path> for bounded references.\n",
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
        command_registry().complete(input)
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
