//! TUI state machine — pure UI state only.
//!
//! Business logic (messages, streaming, queues) lives in `talos-conversation`.
//! This module owns only input handling, approval overlay, and display state.

use std::time::{Duration, Instant};

use talos_conversation::{
    CommandExecutionMode, CredentialResponseData, ModelPickerData, SessionPickerItem,
    StatusSnapshot, TipKind,
};
use talos_core::ApprovalChoice;

pub(crate) const DOUBLE_CTRL_C_WINDOW: Duration = Duration::from_secs(2);
pub(crate) const SLASH_MENU_MAX_VISIBLE: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PanelAction {
    None,
    SendMessage(String),
    ProviderSetup(String),
}

/// What happens when a [`PanelItem`] is accepted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PanelItemAction {
    /// Slash command selected from the command menu.
    SlashCommand {
        command: String,
        arg_hint: Option<String>,
        execution_mode: CommandExecutionMode,
    },
    /// Picker selection — sends `"{command} {value}"` as a message.
    Select { command: String, value: String },
    /// Unauthenticated provider — triggers provider-level credential entry.
    ProviderSetup { provider: String },
    /// Non-navigable group header.
    Header,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PanelItem {
    pub(crate) label: String,
    pub(crate) description: String,
    pub(crate) action: PanelItemAction,
    pub(crate) is_current: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PanelKind {
    SlashCommand,
    SessionPicker,
    ModelPicker,
    CredentialInput {
        provider: String,
        model_id: Option<String>,
    },
    Approval {
        tool_name: String,
        arguments: String,
    },
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct BottomPanelState {
    pub(crate) is_open: bool,
    pub(crate) kind: Option<PanelKind>,
    pub(crate) items: Vec<PanelItem>,
    pub(crate) selected_index: usize,
    pub(crate) credential_buffer: String,
}

impl BottomPanelState {
    pub(crate) fn open_slash(registry: &talos_conversation::CommandRegistry) -> Self {
        let items = registry
            .available_commands()
            .into_iter()
            .map(|cmd| PanelItem {
                label: cmd.name.to_string(),
                description: cmd.description.to_string(),
                action: PanelItemAction::SlashCommand {
                    command: cmd.name.to_string(),
                    arg_hint: cmd.arg_hint.map(str::to_string),
                    execution_mode: cmd.execution_mode(),
                },
                is_current: false,
            })
            .collect();
        Self {
            is_open: true,
            kind: Some(PanelKind::SlashCommand),
            items,
            selected_index: 0,
            credential_buffer: String::new(),
        }
    }

    pub(crate) fn open_session_picker(sessions: &[SessionPickerItem]) -> Self {
        let items = sessions
            .iter()
            .map(|s| PanelItem {
                label: format!(
                    "{}. {} — {} messages",
                    s.ordinal, s.timestamp, s.message_count
                ),
                description: if s.preview.is_empty() {
                    "(empty)".to_string()
                } else {
                    format!("\"{}\"", s.preview)
                },
                action: PanelItemAction::Select {
                    command: if s.command.is_empty() {
                        "/resume".to_string()
                    } else {
                        s.command.clone()
                    },
                    value: s.ordinal.to_string(),
                },
                is_current: false,
            })
            .collect();
        Self {
            is_open: true,
            kind: Some(PanelKind::SessionPicker),
            items,
            selected_index: 0,
            credential_buffer: String::new(),
        }
    }

    pub(crate) fn open_model_picker(data: &ModelPickerData) -> Self {
        let mut panel_items: Vec<PanelItem> = Vec::new();

        if !data.ready_models.is_empty() {
            panel_items.push(PanelItem {
                label: "Ready".into(),
                description: String::new(),
                action: PanelItemAction::Header,
                is_current: false,
            });
            panel_items.extend(data.ready_models.iter().map(|m| PanelItem {
                label: m.label.clone(),
                description: m.provider.clone(),
                action: PanelItemAction::Select {
                    command: m.command.clone(),
                    value: m.model_id.clone(),
                },
                is_current: m.is_current,
            }));
        }

        if !data.setup_providers.is_empty() {
            panel_items.push(PanelItem {
                label: "Setup required".into(),
                description: String::new(),
                action: PanelItemAction::Header,
                is_current: false,
            });
            panel_items.extend(data.setup_providers.iter().map(|p| PanelItem {
                label: format!(
                    "{}   ({} model{})",
                    p.provider,
                    p.model_count,
                    if p.model_count == 1 { "" } else { "s" }
                ),
                description: "Setup required".to_string(),
                action: PanelItemAction::ProviderSetup {
                    provider: p.provider.clone(),
                },
                is_current: false,
            }));
        }

        let initial_index = panel_items
            .iter()
            .position(|i| i.is_current && i.action != PanelItemAction::Header)
            .or_else(|| {
                panel_items
                    .iter()
                    .position(|i| i.action != PanelItemAction::Header)
            })
            .unwrap_or(0);
        Self {
            is_open: true,
            kind: Some(PanelKind::ModelPicker),
            items: panel_items,
            selected_index: initial_index,
            credential_buffer: String::new(),
        }
    }

    pub(crate) fn open_credential_input(provider: &str, model_id: Option<&str>) -> Self {
        Self {
            is_open: true,
            kind: Some(PanelKind::CredentialInput {
                provider: provider.to_string(),
                model_id: model_id.map(|s| s.to_string()),
            }),
            items: vec![],
            selected_index: 0,
            credential_buffer: String::new(),
        }
    }

    pub(crate) fn is_slash(&self) -> bool {
        self.kind == Some(PanelKind::SlashCommand)
    }

    pub(crate) fn is_picker(&self) -> bool {
        matches!(
            self.kind,
            Some(PanelKind::SessionPicker) | Some(PanelKind::ModelPicker)
        )
    }

    pub(crate) fn is_approval(&self) -> bool {
        matches!(self.kind, Some(PanelKind::Approval { .. }))
    }

    pub(crate) fn is_credential_input(&self) -> bool {
        matches!(self.kind, Some(PanelKind::CredentialInput { .. }))
    }

    pub(crate) fn open_approval(tool_name: &str, arguments: &str) -> Self {
        let args_short = if arguments.len() > 72 {
            let mut end = 72;
            while end > 0 && !arguments.is_char_boundary(end) {
                end -= 1;
            }
            format!("{}...", &arguments[..end])
        } else {
            arguments.to_string()
        };
        Self {
            is_open: true,
            kind: Some(PanelKind::Approval {
                tool_name: tool_name.to_string(),
                arguments: args_short,
            }),
            items: vec![
                PanelItem {
                    label: "[y] approve".to_string(),
                    description: String::new(),
                    action: PanelItemAction::Select {
                        command: String::new(),
                        value: "approve".to_string(),
                    },
                    is_current: false,
                },
                PanelItem {
                    label: "[a] always approve".to_string(),
                    description: String::new(),
                    action: PanelItemAction::Select {
                        command: String::new(),
                        value: "always".to_string(),
                    },
                    is_current: false,
                },
                PanelItem {
                    label: "[n] deny".to_string(),
                    description: String::new(),
                    action: PanelItemAction::Select {
                        command: String::new(),
                        value: "deny".to_string(),
                    },
                    is_current: false,
                },
            ],
            selected_index: 0,
            credential_buffer: String::new(),
        }
    }

    pub(crate) fn filtered_items(&self, query: &str) -> Vec<&PanelItem> {
        if !self.is_slash() {
            return self.items.iter().collect();
        }
        if query.is_empty() {
            return self.items.iter().collect();
        }
        let lower = query.to_lowercase();
        self.items
            .iter()
            .filter(|item| {
                item.label
                    .strip_prefix('/')
                    .unwrap_or(&item.label)
                    .to_lowercase()
                    .contains(&lower)
                    || item.description.to_lowercase().contains(&lower)
            })
            .collect()
    }

    pub(crate) fn close(&mut self) {
        self.is_open = false;
        self.kind = None;
        self.items.clear();
        self.selected_index = 0;
    }

    pub(crate) fn select_next(&mut self, query: &str) {
        let len = self.filtered_items(query).len();
        if len == 0 {
            return;
        }
        for _ in 0..len {
            self.selected_index = (self.selected_index + 1) % len;
            if self
                .items
                .get(self.selected_index)
                .is_none_or(|i| i.action != PanelItemAction::Header)
            {
                return;
            }
        }
    }

    pub(crate) fn select_prev(&mut self, query: &str) {
        let len = self.filtered_items(query).len();
        if len == 0 {
            return;
        }
        for _ in 0..len {
            if self.selected_index == 0 {
                self.selected_index = len - 1;
            } else {
                self.selected_index -= 1;
            }
            if self
                .items
                .get(self.selected_index)
                .is_none_or(|i| i.action != PanelItemAction::Header)
            {
                return;
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum ApprovalState {
    #[default]
    Hidden,
    Visible {
        tool_name: String,
        arguments: String,
        selected: ApprovalChoice,
    },
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) enum CtrlCState {
    #[default]
    Idle,
    Waiting(Instant),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Tip {
    pub kind: TipKind,
    pub text: String,
    pub ttl: Duration,
    pub created_at: Instant,
}

#[derive(Debug, Default)]
pub(crate) struct TuiState {
    pub input_buffer: String,
    pub cursor_pos: usize,
    pub ctrl_c_state: CtrlCState,
    pub should_exit: bool,
    pub approval_state: ApprovalState,
    pub pending_approval_response: Option<tokio::sync::oneshot::Sender<ApprovalChoice>>,
    pub tip: Option<Tip>,
    pub status: StatusSnapshot,
    pub slash_menu: BottomPanelState,
}

impl TuiState {
    pub(crate) fn new() -> Self {
        Self::default()
    }

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

    pub(crate) fn input_append_str(&mut self, text: &str) {
        for ch in text.chars() {
            self.input_append_char(ch);
        }
    }

    pub(crate) fn input_paste(&mut self, text: &str) {
        self.ctrl_c_state = CtrlCState::Idle;
        if !matches!(self.approval_state, ApprovalState::Hidden) {
            return;
        }
        if self.slash_menu.is_credential_input() {
            self.credential_append_str(text);
        } else if self.slash_menu.is_slash() {
            self.input_append_str(text);
            self.slash_menu.selected_index = 0;
        } else if !self.slash_menu.is_open {
            self.input_append_str(text);
        }
    }

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

    pub(crate) fn input_cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
    }

    pub(crate) fn input_cursor_right(&mut self) {
        if self.cursor_pos < self.input_buffer.chars().count() {
            self.cursor_pos += 1;
        }
    }

    pub(crate) fn input_clear(&mut self) {
        self.input_buffer.clear();
        self.cursor_pos = 0;
    }

    pub(crate) fn input_insert_command(&mut self, command: &str) {
        self.input_buffer.clear();
        self.cursor_pos = 0;
        self.input_append_str(command);
    }

    pub(crate) fn open_slash_menu(&mut self, registry: &talos_conversation::CommandRegistry) {
        self.input_append_char('/');
        self.slash_menu = BottomPanelState::open_slash(registry);
    }

    pub(crate) fn open_session_picker(&mut self, sessions: &[SessionPickerItem]) {
        self.slash_menu = BottomPanelState::open_session_picker(sessions);
    }

    pub(crate) fn open_model_picker(&mut self, data: &ModelPickerData) {
        self.slash_menu = BottomPanelState::open_model_picker(data);
    }

    pub(crate) fn open_credential_input(&mut self, provider: &str, model_id: Option<&str>) {
        self.slash_menu = BottomPanelState::open_credential_input(provider, model_id);
    }

    pub(crate) fn credential_append_char(&mut self, ch: char) {
        if self.slash_menu.is_credential_input() {
            self.slash_menu.credential_buffer.push(ch);
        }
    }

    pub(crate) fn credential_append_str(&mut self, text: &str) {
        if self.slash_menu.is_credential_input() {
            self.slash_menu.credential_buffer.push_str(text);
        }
    }

    pub(crate) fn credential_backspace(&mut self) {
        if self.slash_menu.is_credential_input() {
            self.slash_menu.credential_buffer.pop();
        }
    }

    pub(crate) fn credential_submit(&mut self) -> Option<CredentialResponseData> {
        if !self.slash_menu.is_credential_input() {
            return None;
        }
        let (provider, model_id) = match &self.slash_menu.kind {
            Some(PanelKind::CredentialInput { provider, model_id }) => {
                (provider.clone(), model_id.clone())
            }
            _ => return None,
        };
        let key = std::mem::take(&mut self.slash_menu.credential_buffer)
            .trim()
            .to_string();
        self.slash_menu.close();
        if key.is_empty() {
            None
        } else {
            Some(CredentialResponseData {
                provider,
                api_key: key,
                model_id,
            })
        }
    }

    pub(crate) fn credential_cancel(&mut self) {
        self.slash_menu.credential_buffer.clear();
        self.slash_menu.close();
    }

    pub(crate) fn slash_query(&self) -> &str {
        self.input_buffer.strip_prefix('/').unwrap_or_default()
    }

    pub(crate) fn append_slash_query_char(&mut self, ch: char) {
        self.input_append_char(ch);
        self.slash_menu.selected_index = 0;
    }

    pub(crate) fn backspace_slash_query(&mut self) {
        self.input_backspace();
        self.slash_menu.selected_index = 0;
        if !self.input_buffer.starts_with('/') {
            self.slash_menu.close();
        }
    }

    pub(crate) fn accept_selected_panel_item(&mut self) -> PanelAction {
        self.accept_selected_panel_item_with_mode(PanelAcceptMode::Enter)
    }

    pub(crate) fn complete_selected_panel_item(&mut self) -> PanelAction {
        self.accept_selected_panel_item_with_mode(PanelAcceptMode::Complete)
    }

    fn accept_selected_panel_item_with_mode(&mut self, mode: PanelAcceptMode) -> PanelAction {
        let query = self.slash_query().to_string();
        let filtered = self.slash_menu.filtered_items(&query);
        if filtered.is_empty() {
            return PanelAction::None;
        }
        let idx = self.slash_menu.selected_index.min(filtered.len() - 1);
        let action = filtered[idx].action.clone();

        match action {
            PanelItemAction::Header => PanelAction::None,
            PanelItemAction::SlashCommand {
                command,
                arg_hint,
                execution_mode,
            } => {
                if mode == PanelAcceptMode::Enter
                    && execution_mode == CommandExecutionMode::DirectExecution
                {
                    self.input_clear();
                    self.slash_menu.close();
                    return PanelAction::SendMessage(command);
                }
                let inserted = if arg_hint.is_some() {
                    format!("{command} ")
                } else {
                    command
                };
                self.input_insert_command(&inserted);
                self.slash_menu.close();
                PanelAction::None
            }
            PanelItemAction::Select { command, value } => {
                self.slash_menu.close();
                PanelAction::SendMessage(format!("{command} {value}"))
            }
            PanelItemAction::ProviderSetup { provider } => {
                self.slash_menu.close();
                PanelAction::ProviderSetup(provider)
            }
        }
    }

    pub(crate) fn activate_approval(&mut self, tool_name: &str, arguments: &str) {
        self.slash_menu.close();
        self.approval_state = ApprovalState::Visible {
            tool_name: tool_name.to_string(),
            arguments: arguments.to_string(),
            selected: ApprovalChoice::ApproveOnce,
        };
    }

    pub(crate) fn input_submit(&mut self) -> String {
        let content = self.input_buffer.clone();
        self.input_clear();
        content
    }

    pub(crate) fn cursor_byte_pos(&self) -> usize {
        self.input_buffer
            .char_indices()
            .nth(self.cursor_pos)
            .map(|(i, _)| i)
            .unwrap_or(self.input_buffer.len())
    }

    pub(crate) fn handle_ctrl_c(&mut self) -> bool {
        let now = Instant::now();
        match &self.ctrl_c_state {
            CtrlCState::Idle => {
                let text = if self.status.is_processing {
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
                    self.ctrl_c_state = CtrlCState::Waiting(now);
                    false
                }
            }
        }
    }

    pub(crate) fn expire_tip(&mut self) {
        if let Some(ref tip) = self.tip
            && Instant::now().duration_since(tip.created_at) >= tip.ttl
        {
            self.tip = None;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PanelAcceptMode {
    Enter,
    Complete,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_input_collects_pasted_text_and_submits() {
        let mut state = TuiState::new();

        state.open_credential_input("openai", None);
        state.credential_append_str("sk-test-key\n");

        let response = state.credential_submit().expect("credential response");
        assert_eq!(response.provider, "openai");
        assert_eq!(response.api_key, "sk-test-key");
        assert_eq!(response.model_id, None);
        assert!(!state.slash_menu.is_open);
    }

    #[test]
    fn empty_credential_submit_closes_without_response() {
        let mut state = TuiState::new();

        state.open_credential_input("openai", Some("gpt-4.1"));

        assert!(state.credential_submit().is_none());
        assert!(!state.slash_menu.is_open);
    }

    #[test]
    fn paste_is_ignored_while_approval_is_visible() {
        let mut state = TuiState::new();

        state.activate_approval("write", "file edit");
        state.input_paste("secret");

        assert_eq!(state.input_buffer, "");
    }

    #[test]
    fn paste_is_ignored_while_picker_is_visible() {
        let mut state = TuiState::new();
        let data = ModelPickerData {
            ready_models: vec![],
            setup_providers: vec![],
        };

        state.open_model_picker(&data);
        state.input_paste("secret");

        assert_eq!(state.input_buffer, "");
    }

    #[test]
    fn paste_still_updates_slash_query_and_composer() {
        let mut state = TuiState::new();

        state.input_paste("hello");
        assert_eq!(state.input_buffer, "hello");

        state.input_clear();
        state.open_slash_menu(talos_conversation::command_registry());
        state.input_paste("model");

        assert_eq!(state.input_buffer, "/model");
    }

    #[test]
    fn approval_truncation_handles_multibyte_utf8() {
        let cmd = "gh issue create --title \"feat: write 和 edit 工具应显示内容输出\" --label bug";
        let state = BottomPanelState::open_approval("bash", cmd);
        if let PanelKind::Approval { arguments, .. } = state.kind.as_ref().unwrap() {
            assert!(arguments.ends_with("..."));
        }
    }

    #[test]
    fn approval_truncation_short_string_unchanged() {
        let state = BottomPanelState::open_approval("bash", "ls -la");
        if let PanelKind::Approval { arguments, .. } = state.kind.as_ref().unwrap() {
            assert_eq!(arguments, "ls -la");
        }
    }
}
