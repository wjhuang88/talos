//! TUI state machine — pure UI state only.
//!
//! Business logic (messages, streaming, queues) lives in `talos-conversation`.
//! This module owns only input handling, approval overlay, and display state.

use std::time::{Duration, Instant};

use talos_conversation::{
    CredentialResponseData, ModelPickerItem, SessionPickerItem, StatusSnapshot, TipKind,
};
use talos_core::ApprovalChoice;

pub(crate) const DOUBLE_CTRL_C_WINDOW: Duration = Duration::from_secs(2);
pub(crate) const SLASH_MENU_MAX_VISIBLE: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PanelAction {
    None,
    SendMessage(String),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PanelItem {
    pub(crate) label: String,
    pub(crate) description: String,
    pub(crate) value: String,
    /// Slash command to submit when the row is accepted. Empty for slash
    /// commands (the value itself is the full command) and for approval rows.
    #[allow(dead_code)]
    pub(crate) command: String,
    /// If true, this item is a non-navigable group header (used in model picker).
    /// Headers are skipped by Up/Down navigation and cannot be selected.
    #[allow(dead_code)]
    pub(crate) is_header: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PanelKind {
    SlashCommand,
    SessionPicker,
    ModelPicker,
    CredentialInput {
        provider: String,
        model_id: String,
    },
    Approval { tool_name: String, arguments: String },
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
                value: cmd.name.to_string(),
                command: String::new(),
                is_header: false,
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
                label: format!("{}. {} — {} messages", s.ordinal, s.timestamp, s.message_count),
                description: if s.preview.is_empty() {
                    "(empty)".to_string()
                } else {
                    format!("\"{}\"", s.preview)
                },
                value: s.ordinal.to_string(),
                command: s.command.clone(),
                is_header: false,
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

    pub(crate) fn open_model_picker(items: &[ModelPickerItem]) -> Self {
        let mut panel_items: Vec<PanelItem> = Vec::new();
        let ready_header = PanelItem {
            is_header: true,
            label: "Ready".into(),
            description: String::new(),
            value: String::new(),
            command: String::new(),
        };
        panel_items.push(ready_header);
        panel_items.extend(items.iter().filter(|m| m.authenticated).map(|m| PanelItem {
            label: m.label.clone(),
            description: m.provider.clone(),
            value: m.model_id.clone(),
            command: m.command.clone(),
            is_header: false,
        }));
        let setup_header = PanelItem {
            is_header: true,
            label: "Setup required".into(),
            description: String::new(),
            value: String::new(),
            command: String::new(),
        };
        panel_items.push(setup_header);
        panel_items.extend(items.iter().filter(|m| !m.authenticated).map(|m| PanelItem {
            label: m.label.clone(),
            description: format!("{} (setup required)", m.provider),
            value: m.model_id.clone(),
            command: m.command.clone(),
            is_header: false,
        }));
        let initial_index = panel_items
            .iter()
            .position(|i| !i.is_header)
            .unwrap_or(0);
        Self {
            is_open: true,
            kind: Some(PanelKind::ModelPicker),
            items: panel_items,
            selected_index: initial_index,
            credential_buffer: String::new(),
        }
    }

    pub(crate) fn open_credential_input(provider: &str, model_id: &str) -> Self {
        Self {
            is_open: true,
            kind: Some(PanelKind::CredentialInput {
                provider: provider.to_string(),
                model_id: model_id.to_string(),
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
            format!("{}...", &arguments[..72])
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
                    value: "approve".to_string(),
                    command: String::new(),
                    is_header: false,
                },
                PanelItem {
                    label: "[a] always approve".to_string(),
                    description: String::new(),
                    value: "always".to_string(),
                    command: String::new(),
                    is_header: false,
                },
                PanelItem {
                    label: "[n] deny".to_string(),
                    description: String::new(),
                    value: "deny".to_string(),
                    command: String::new(),
                    is_header: false,
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

    pub(crate) fn selected_completion(&self, query: &str) -> Option<String> {
        let filtered = self.filtered_items(query);
        if filtered.is_empty() {
            return None;
        }
        let idx = self.selected_index.min(filtered.len() - 1);
        let item = filtered[idx];
        if self.is_slash() {
            let has_arg = item.description.contains("[") || item.description.contains("<");
            let suffix = if has_arg { " " } else { "" };
            Some(format!("{}{suffix}", item.value))
        } else {
            Some(item.value.clone())
        }
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
            // Use raw items to check header status (avoids borrow conflict with selected_index)
            if self.items.get(self.selected_index).is_none_or(|i| !i.is_header) {
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
            if self.items.get(self.selected_index).is_none_or(|i| !i.is_header) {
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

    pub(crate) fn open_model_picker(&mut self, items: &[ModelPickerItem]) {
        self.slash_menu = BottomPanelState::open_model_picker(items);
    }

    pub(crate) fn open_credential_input(&mut self, provider: &str, model_id: &str) {
        self.slash_menu = BottomPanelState::open_credential_input(provider, model_id);
    }

    pub(crate) fn credential_append_char(&mut self, ch: char) {
        if self.slash_menu.is_credential_input() {
            self.slash_menu.credential_buffer.push(ch);
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
        let key = std::mem::take(&mut self.slash_menu.credential_buffer);
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
        // Guard: skip header items (non-navigable group headers)
        if self
            .slash_menu
            .items
            .get(self.slash_menu.selected_index)
            .is_some_and(|i| i.is_header)
        {
            return PanelAction::None;
        }
        let completion = self.slash_menu.selected_completion(self.slash_query());
        if self.slash_menu.is_picker() {
            let value = completion.unwrap_or_default();
            let command = self
                .slash_menu
                .items
                .get(self.slash_menu.selected_index)
                .map(|i| i.command.clone())
                .filter(|c| !c.is_empty())
                .unwrap_or_else(|| "/resume".to_string());
            self.slash_menu.close();
            return PanelAction::SendMessage(format!("{command} {value}"));
        }
        if let Some(command) = completion {
            self.input_insert_command(&command);
        }
        self.slash_menu.close();
        PanelAction::None
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
