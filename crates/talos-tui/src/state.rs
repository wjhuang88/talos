//! TUI state machine.

use std::time::{Duration, Instant};

use talos_core::ApprovalChoice;
use talos_core::message::{AgentEvent, ToolCall, ToolResult, Usage};
use talos_core::tool::ToolProvenance;

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

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ChatLine {
    Text(String),
    Assistant(String),
    ToolCall {
        tool_name: String,
        arguments: String,
        provenance: ToolProvenance,
        result: Option<ToolResult>,
    },
}

#[derive(Debug, Default)]
pub(crate) struct TuiState {
    pub(crate) chat_lines: Vec<ChatLine>,
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
    /// Messages queued for steering (delivered after current tool batch).
    pub(crate) steering_queue: Vec<String>,
    /// Messages queued for follow-up (delivered when agent would stop).
    pub(crate) followup_queue: Vec<String>,
    pub(crate) plugin_observations: Vec<PluginObservation>,
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
            self.chat_lines
                .push(ChatLine::Assistant(self.current_turn_text.clone()));
            self.current_turn_text.clear();
        }
    }

    pub(crate) fn append_user_message(&mut self, content: &str) {
        self.chat_lines.push(ChatLine::Text(format!("> {content}")));
    }

    pub(crate) fn append_error(&mut self, message: &str) {
        self.chat_lines
            .push(ChatLine::Text(format!("[Error] {message}")));
    }

    pub(crate) fn append_system(&mut self, message: &str) {
        self.chat_lines
            .push(ChatLine::Text(format!("[System] {message}")));
    }

    pub(crate) fn append_tool_call(&mut self, call: &ToolCall, provenance: &ToolProvenance) {
        self.record_provenance(provenance);
        self.chat_lines.push(ChatLine::ToolCall {
            tool_name: call.name.clone(),
            arguments: serde_json::to_string_pretty(&call.input)
                .unwrap_or_else(|_| call.input.to_string()),
            provenance: provenance.clone(),
            result: None,
        });
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
        for line in self.chat_lines.iter_mut().rev() {
            if let ChatLine::ToolCall {
                tool_name: _,
                arguments: _,
                provenance: _,
                result: slot,
            } = line
            {
                if slot.is_none() {
                    *slot = Some(result.clone());
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
                if self.is_processing {
                    self.append_system("Turn cancelled. Press Ctrl+C again to exit.");
                } else {
                    self.append_system("Press Ctrl+C again within 2 seconds to exit.");
                }
                self.ctrl_c_state = CtrlCState::Waiting(now);
                false
            }
            CtrlCState::Waiting(pressed_at) => {
                if now.duration_since(*pressed_at) < DOUBLE_CTRL_C_WINDOW {
                    self.should_exit = true;
                    true
                } else {
                    self.append_system("Press Ctrl+C again within 2 seconds to exit.");
                    self.ctrl_c_state = CtrlCState::Waiting(now);
                    false
                }
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn set_branch_id(&mut self, branch_id: String) {
        self.branch_id = Some(branch_id);
    }

    pub(crate) fn handle_event(&mut self, event: &AgentEvent) {
        match event {
            AgentEvent::TurnStart => {
                self.is_processing = true;
                self.current_turn_text.clear();
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
                self.chat_lines.clear();
                self.current_turn_text.clear();
                self.usage = Usage::default();
                self.branch_id = None;
                self.plugin_observations.clear();
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

    /// Returns the source text of the most recent [`ChatLine::Assistant`],
    /// if any. Streaming-in-progress text (`current_turn_text`) is excluded.
    pub(crate) fn last_assistant_text(&self) -> Option<String> {
        self.chat_lines.iter().rev().find_map(|line| match line {
            ChatLine::Assistant(text) => Some(text.clone()),
            _ => None,
        })
    }

    /// Renders the full transcript as deterministic plain text.
    ///
    /// Source message text is preserved verbatim — the function never reads
    /// from the rendered buffer. `current_turn_text` is excluded so
    /// in-flight streaming turns do not produce half-copied output.
    pub(crate) fn transcript_plain_text(&self) -> String {
        let mut out = String::new();
        for line in &self.chat_lines {
            Self::append_line_plain(&mut out, line);
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
        for line in &self.chat_lines {
            Self::append_line_markdown(&mut out, line);
        }
        out
    }

    pub(crate) fn append_line_plain(out: &mut String, line: &ChatLine) {
        match line {
            ChatLine::Text(text) => {
                out.push_str(text);
                out.push('\n');
            }
            ChatLine::Assistant(text) => {
                out.push_str(text);
                if !text.ends_with('\n') {
                    out.push('\n');
                }
            }
            ChatLine::ToolCall {
                tool_name,
                arguments,
                provenance,
                result,
            } => {
                let marker = plugin_observation_key(provenance);
                out.push_str(&format!("▸ {tool_name} [{marker}]\n"));
                out.push_str(&format!("  {arguments}\n"));
                if let Some(result) = result {
                    let icon = if result.is_error { "✗" } else { "✓" };
                    out.push_str(&format!("  {icon} {}\n", result.content));
                }
            }
        }
    }

    fn append_line_markdown(out: &mut String, line: &ChatLine) {
        match line {
            ChatLine::Text(text) => {
                out.push_str(text);
                out.push('\n');
            }
            ChatLine::Assistant(text) => {
                out.push_str(text);
                if !text.ends_with('\n') {
                    out.push('\n');
                }
            }
            ChatLine::ToolCall {
                tool_name,
                arguments,
                provenance,
                result,
            } => {
                let marker = plugin_observation_key(provenance);
                out.push_str(&format!("### `▸ {tool_name} [{marker}]`\n\n"));
                out.push_str("```json\n");
                out.push_str(arguments);
                out.push_str("\n```\n");
                if let Some(result) = result {
                    let label = if result.is_error { "Error" } else { "Result" };
                    out.push_str(&format!("\n**{label}:**\n\n"));
                    out.push_str("```\n");
                    out.push_str(&result.content);
                    out.push_str("\n```\n");
                }
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
