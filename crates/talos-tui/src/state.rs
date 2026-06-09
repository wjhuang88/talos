//! TUI state machine.

use std::time::{Duration, Instant};

use talos_core::ApprovalChoice;
use talos_core::message::{AgentEvent, ToolCall, ToolResult, Usage};
use talos_core::tool::ToolProvenance;
use tokio::sync::mpsc;

/// Plugin/tool provenance observation summary.
///
/// `key` is the display identifier: `native` or `mcp:<server>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PluginObservation {
    pub key: String,
    pub count: usize,
}

pub(crate) fn plugin_observation_key(provenance: &ToolProvenance) -> String {
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

/// Duration window for detecting double Ctrl+C press.
pub(crate) const DOUBLE_CTRL_C_WINDOW: Duration = Duration::from_secs(2);

/// State of the approval overlay.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ApprovalState {
    /// Overlay is hidden.
    #[default]
    Hidden,
    /// Overlay is visible with pending tool call details.
    Visible {
        /// Name of the tool requesting approval.
        tool_name: String,
        /// Formatted tool arguments.
        arguments: String,
        /// Currently selected approval choice.
        selected: ApprovalChoice,
    },
}

/// State machine for Ctrl+C handling.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) enum CtrlCState {
    /// No Ctrl+C pressed yet.
    #[default]
    Idle,
    /// First Ctrl+C pressed, waiting for second press within window.
    Waiting(Instant),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MessageRole {
    User,
    Assistant,
    System,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MessageStatus {
    Pending,
    Accepted,
    Streaming,
    Completed,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ChatMessage {
    pub role: MessageRole,
    pub status: MessageStatus,
    pub content: String,
    pub tool_call: Option<ToolCallInfo>,
    pub created_at: Instant,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ToolCallInfo {
    pub tool_name: String,
    pub arguments: String,
    pub provenance: ToolProvenance,
    pub result: Option<ToolResult>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TipKind {
    ExitHint,
    QueueHint,
    ApprovalResult,
    LagWarning,
    Info,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Tip {
    pub kind: TipKind,
    pub text: String,
    pub ttl: Duration,
    pub created_at: Instant,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TuiStateEvent {
    MessageAdded { index: usize, role: MessageRole },
    MessageStatusChanged { index: usize, from: MessageStatus, to: MessageStatus },
    TipShown { kind: TipKind },
    SplashComplete,
}


#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct ScrollbackState {
    pub(crate) scrolled_line_count: usize,
}

#[derive(Debug, Default)]
pub(crate) struct TuiState {
    pub(crate) messages: Vec<ChatMessage>,
    pub(crate) input_buffer: String,
    pub(crate) cursor_pos: usize,
    pub(crate) is_processing: bool,
    pub(crate) current_turn_text: String,
    pub(crate) ctrl_c_state: CtrlCState,
    pub(crate) should_exit: bool,
    pub(crate) usage: Usage,
    pub(crate) model_name: String,
    pub(crate) approval_state: ApprovalState,
    pub(crate) branch_id: Option<String>,
    pub(crate) pending_approval_response: Option<tokio::sync::oneshot::Sender<ApprovalChoice>>,
    pub(crate) steering_queue: Vec<String>,
    pub(crate) followup_queue: Vec<String>,
    pub(crate) plugin_observations: Vec<PluginObservation>,
    pub(crate) scrollback: ScrollbackState,
    pub(crate) tip: Option<Tip>,
    pub(crate) event_tx: Option<mpsc::UnboundedSender<TuiStateEvent>>,
}

impl TuiState {
    /// Creates a new default TUI state.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Appends a text delta to the current turn's streaming text.
    pub(crate) fn append_delta(&mut self, delta: &str) {
        self.current_turn_text.push_str(delta);
    }

    pub(crate) fn finalize_turn(&mut self) {
        if !self.current_turn_text.is_empty() {
            self.current_turn_text.clear();
        }
    }

    fn emit_event(&mut self, event: TuiStateEvent) {
        if let Some(ref tx) = self.event_tx {
            let _ = tx.send(event);
        }
    }

    pub(crate) fn append_user_message(&mut self, content: &str) {
        let index = self.messages.len();
        self.messages.push(ChatMessage {
            role: MessageRole::User,
            status: MessageStatus::Completed,
            content: format!("> {content}"),
            tool_call: None,
            created_at: Instant::now(),
        });
        self.emit_event(TuiStateEvent::MessageAdded { index, role: MessageRole::User });
    }

    pub(crate) fn append_error(&mut self, message: &str) {
        let index = self.messages.len();
        self.messages.push(ChatMessage {
            role: MessageRole::Error,
            status: MessageStatus::Completed,
            content: format!("[Error] {message}"),
            tool_call: None,
            created_at: Instant::now(),
        });
        self.emit_event(TuiStateEvent::MessageAdded { index, role: MessageRole::Error });
    }

    pub(crate) fn append_system(&mut self, message: &str) {
        let index = self.messages.len();
        self.messages.push(ChatMessage {
            role: MessageRole::System,
            status: MessageStatus::Completed,
            content: format!("[System] {message}"),
            tool_call: None,
            created_at: Instant::now(),
        });
        self.emit_event(TuiStateEvent::MessageAdded { index, role: MessageRole::System });
    }

    pub(crate) fn append_tool_call(&mut self, call: &ToolCall, provenance: &ToolProvenance) {
        self.record_provenance(provenance);
        let index = self.messages.len();
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
        self.emit_event(TuiStateEvent::MessageAdded { index, role: MessageRole::Assistant });
    }

    fn record_provenance(&mut self, provenance: &ToolProvenance) {
        let key = plugin_observation_key(provenance);
        if let Some(entry) = self
            .plugin_observations
            .iter_mut()
            .find(|entry| entry.key == key)
        {
            entry.count += 1;
        } else {
            self.plugin_observations.push(PluginObservation { key, count: 1 });
        }
    }

    pub(crate) fn set_tool_result(&mut self, result: &ToolResult) {
        for msg in self.messages.iter_mut().rev() {
            if let Some(ref mut tool_call) = msg.tool_call {
                if tool_call.result.is_none() {
                    tool_call.result = Some(result.clone());
                    break;
                }
            }
        }
    }

    /// Appends a character to the input buffer at the cursor position.
    pub(crate) fn input_append_char(&mut self, ch: char) {
        let byte_pos = self
            .input_buffer
            .char_indices()
            .nth(self.cursor_pos)
            .map(|(i, _)| i)
            .unwrap_or(self.input_buffer.len());
        self.input_buffer.insert(byte_pos, ch);
        self.cursor_pos += 1;
    }

    /// Deletes the character before the cursor in the input buffer.
    pub(crate) fn input_backspace(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            let byte_pos = self
                .input_buffer
                .char_indices()
                .nth(self.cursor_pos)
                .map(|(i, _)| i)
                .unwrap_or(self.input_buffer.len());
            self.input_buffer.remove(byte_pos);
        }
    }

    /// Moves the cursor left in the input buffer.
    pub(crate) fn input_cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
    }

    /// Moves the cursor right in the input buffer.
    pub(crate) fn input_cursor_right(&mut self) {
        if self.cursor_pos < self.input_buffer.chars().count() {
            self.cursor_pos += 1;
        }
    }

    /// Clears the input buffer and resets cursor position.
    pub(crate) fn input_clear(&mut self) {
        self.input_buffer.clear();
        self.cursor_pos = 0;
    }

    /// Submits the input buffer, returning its content and clearing it.
    pub(crate) fn input_submit(&mut self) -> String {
        let content = self.input_buffer.clone();
        self.input_clear();
        content
    }

    #[allow(dead_code)]
    /// Drains the first message from the steering queue (FIFO).
    pub(crate) fn drain_steering_queue(&mut self) -> Option<String> {
        if self.steering_queue.is_empty() {
            None
        } else {
            Some(self.steering_queue.remove(0))
        }
    }

    /// Restores the most recent queued message to the input buffer.
    pub(crate) fn restore_last_queued(&mut self) -> bool {
        if let Some(msg) = self.steering_queue.pop() {
            self.input_buffer = msg;
            self.cursor_pos = self.input_buffer.chars().count();
            true
        } else if let Some(msg) = self.followup_queue.pop() {
            self.input_buffer = msg;
            self.cursor_pos = self.input_buffer.chars().count();
            true
        } else {
            false
        }
    }

    /// Handles a Ctrl+C press, returning whether to exit.
    ///
    /// First press cancels the current turn (if processing).
    /// Second press within [`DOUBLE_CTRL_C_WINDOW`] exits the TUI.
    pub(crate) fn handle_ctrl_c(&mut self) -> bool {
        let now = Instant::now();
        match &self.ctrl_c_state {
            CtrlCState::Idle => {
                let text = if self.is_processing {
                    "Turn cancelled. Press Ctrl+C again to exit.".to_string()
                } else {
                    "Press Ctrl+C again within 2 seconds to exit.".to_string()
                };
                self.tip = Some(Tip {
                    kind: TipKind::ExitHint,
                    text,
                    ttl: Duration::from_secs(2),
                    created_at: now,
                });
                self.emit_event(TuiStateEvent::TipShown { kind: TipKind::ExitHint });
                self.ctrl_c_state = CtrlCState::Waiting(now);
                false
            }
            CtrlCState::Waiting(pressed_at) => {
                if now.duration_since(*pressed_at) < DOUBLE_CTRL_C_WINDOW {
                    self.should_exit = true;
                    true
                } else {
                    self.tip = Some(Tip {
                        kind: TipKind::ExitHint,
                        text: "Press Ctrl+C again within 2 seconds to exit.".to_string(),
                        ttl: Duration::from_secs(2),
                        created_at: now,
                    });
                    self.emit_event(TuiStateEvent::TipShown { kind: TipKind::ExitHint });
                    self.ctrl_c_state = CtrlCState::Waiting(now);
                    false
                }
            }
        }
    }

    pub(crate) fn expire_tip(&mut self) {
        if let Some(ref tip) = self.tip {
            if Instant::now().duration_since(tip.created_at) >= tip.ttl {
                self.tip = None;
            }
        }
    }

    pub(crate) fn handle_event(&mut self, event: &AgentEvent) {
        match event {
            AgentEvent::TurnStart => {
                self.is_processing = true;
                self.current_turn_text.clear();
                self.tip = None;
            }
            AgentEvent::TextDelta { delta } => {
                self.append_delta(delta);
            }
            AgentEvent::ToolCall { call, provenance } => {
                self.append_tool_call(call, provenance);
            }
            AgentEvent::ToolResult { result } => {
                self.set_tool_result(result);
            }
            AgentEvent::TurnEnd { usage, .. } => {
                self.is_processing = false;
                self.finalize_turn();
                self.tip = None;
                self.usage = usage.clone();
            }
            AgentEvent::Error { message } => {
                self.is_processing = false;
                self.current_turn_text.clear();
                self.append_error(message);
            }
            _ => {}
        }
    }

    pub(crate) const SLASH_COMMANDS: &[&str] = &[
        "/help", "/quit", "/exit", "/status", "/new", "/compact", "/diff", "/model", "/resume",
        "/fork", "/vim", "/plugins", "/copy", "/export",
    ];

    pub(crate) fn handle_slash_command(&mut self, input: &str) {
        let parts: Vec<&str> = input.splitn(2, ' ').collect();
        let cmd = parts[0];
        let arg = parts.get(1).copied().unwrap_or("");

        match cmd {
            "/help" => {
                self.append_system("Available commands:");
                self.append_system("  /help       — Show this help");
                self.append_system("  /quit       — Exit Talos");
                self.append_system("  /status     — Show session info");
                self.append_system("  /new        — Start fresh session");
                self.append_system("  /compact    — Compact conversation context");
                self.append_system("  /diff       — Show git diff");
                self.append_system("  /model      — Switch model");
                self.append_system("  /resume     — Resume a session");
                self.append_system("  /fork       — Fork current session");
                self.append_system("  /vim        — Toggle vim keybindings");
                self.append_system("  /plugins    — List observed tool provenance");
                self.append_system("  /copy last  — Copy last assistant message");
                self.append_system("  /copy all   — Copy full transcript");
                self.append_system("  /export <p> — Export transcript to path");
            }
            "/quit" | "/exit" => {
                self.should_exit = true;
            }
            "/status" => {
                let usage = &self.usage;
                self.append_system(&format!(
                    "Model: {} | Input: {} | Output: {} tokens",
                    self.model_name, usage.input_tokens, usage.output_tokens,
                ));
            }
            "/new" => {
                self.messages.clear();
                self.current_turn_text.clear();
                self.usage = Usage::default();
                self.branch_id = None;
                self.plugin_observations.clear();
                self.scrollback.scrolled_line_count = 0;
                self.append_system("New session started.");
            }
            "/plugins" => {
                self.handle_plugins_command();
            }
            "/copy" => {
                self.handle_copy_command(arg);
            }
            "/export" => {
                self.handle_export_command(arg);
            }
            _ => {
                self.append_error(&format!(
                    "Unknown command: {cmd}. Type /help for available commands."
                ));
            }
        }
    }

    fn handle_plugins_command(&mut self) {
        if self.plugin_observations.is_empty() {
            self.append_system("No tool provenance observed yet.");
            return;
        }
        self.append_system("Observed tool provenance (this session):");
        let lines: Vec<String> = self
            .plugin_observations
            .iter()
            .map(|entry| {
                format!(
                    "  {} ({} call{})",
                    entry.key,
                    entry.count,
                    if entry.count == 1 { "" } else { "s" },
                )
            })
            .collect();
        for line in lines {
            self.append_system(&line);
        }
    }

    fn handle_copy_command(&mut self, arg: &str) {
        let arg = arg.trim();
        let text = match arg {
            "last" => match self.last_assistant_text() {
                Some(text) => text,
                None => {
                    self.append_error("No assistant message to copy.");
                    return;
                }
            },
            "all" => self.transcript_plain_text(),
            other => {
                self.append_error(&format!(
                    "Unknown /copy target: '{other}'. Use 'last' or 'all'."
                ));
                return;
            }
        };

        if text.is_empty() {
            self.append_error("Nothing to copy (empty content).");
            return;
        }

        match crate::clipboard::copy_text(&text) {
            Ok(backend) => {
                let label = match backend {
                    crate::clipboard::ClipboardBackend::Osc52 => "OSC 52",
                    crate::clipboard::ClipboardBackend::Pbcopy => "pbcopy",
                };
                self.append_system(&format!(
                    "Copied {} character(s) to clipboard via {label}.",
                    text.chars().count(),
                ));
            }
            Err(e) => {
                self.append_error(&format!("Clipboard write failed: {e:?}"));
            }
        }
    }

    fn handle_export_command(&mut self, arg: &str) {
        let path_str = arg.trim();
        if path_str.is_empty() {
            self.append_error("Usage: /export <path>");
            return;
        }

        let path = std::path::PathBuf::from(path_str);
        let content = self.transcript_markdown();
        let engine = talos_permission::PermissionEngine::new();

        match crate::export::export_transcript(&engine, &path, &content) {
            Ok(()) => {
                self.append_system(&format!(
                    "Exported transcript ({} character(s)) to {}.",
                    content.chars().count(),
                    path.display(),
                ));
            }
            Err(e) => {
                self.append_error(&format!("Export failed: {e:?}"));
            }
        }
    }

    /// Returns the source text of the most recent assistant message (text-only, no tool call),
    /// if any. Streaming-in-progress text (`current_turn_text`) is excluded.
    pub(crate) fn last_assistant_text(&self) -> Option<String> {
        self.messages.iter().rev().find_map(|msg| {
            if msg.role == MessageRole::Assistant && msg.tool_call.is_none() && !msg.content.is_empty() {
                Some(msg.content.clone())
            } else {
                None
            }
        })
    }

    /// Renders the full transcript as deterministic plain text.
    ///
    /// Source message text is preserved verbatim — the function never reads
    /// from the rendered buffer. `current_turn_text` is excluded so
    /// in-flight streaming turns do not produce half-copied output.
    pub(crate) fn transcript_plain_text(&self) -> String {
        let mut out = String::new();
        for msg in &self.messages {
            Self::append_message_plain(&mut out, msg);
        }
        out
    }

    /// Renders the full transcript as Markdown for `/export` consumers.
    ///
    /// Assistant text is emitted verbatim (it is already Markdown).
    /// Tool calls are rendered as fenced code blocks for downstream
    /// processors.
    pub(crate) fn transcript_markdown(&self) -> String {
        let mut out = String::new();
        for msg in &self.messages {
            Self::append_message_markdown(&mut out, msg);
        }
        out
    }


    pub(crate) fn append_message_plain(out: &mut String, msg: &ChatMessage) {
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
                out.push_str(&format!("  {icon} {}\n", result.content));
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

    pub(crate) fn complete_slash_command(&mut self) {
        let input = &self.input_buffer;
        let matches: Vec<&&str> = Self::SLASH_COMMANDS
            .iter()
            .filter(|c| c.starts_with(input.as_str()))
            .collect();
        if matches.len() == 1 {
            self.input_buffer = matches[0].to_string();
            self.cursor_pos = self.input_buffer.len();
            self.input_append_char(' ');
        } else if !matches.is_empty() {
            let listing = matches
                .iter()
                .map(|m| m.to_string())
                .collect::<Vec<_>>()
                .join("  ");
            self.append_system(&format!("Commands: {listing}"));
        }
    }
}
