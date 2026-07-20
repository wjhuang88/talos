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

pub(crate) use crate::panel_state::{
    BottomPanelState, CredentialField, PanelAcceptMode, PanelAction, PanelItemAction, PanelKind,
    SLASH_MENU_MAX_VISIBLE,
};

pub(crate) const DOUBLE_CTRL_C_WINDOW: Duration = Duration::from_secs(2);

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
    pub thinking_preview: Option<String>,
    pub steering_queue_snapshot: Option<talos_conversation::SteeringQueueSnapshot>,
    pub slash_menu: BottomPanelState,
    /// In-memory submitted-input history (TUI-030). Index 0 = oldest.
    pub input_history: Vec<String>,
    /// `None` = editing the live draft; `Some(i)` = browsing `input_history[i]`.
    pub history_cursor: Option<usize>,
    /// Saved draft text when the user navigates into history. Restored on Down
    /// past the newest entry.
    pub draft_input: String,
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
        } else if self.slash_menu.is_open {
            self.input_append_str(text);
            let query = self.panel_query().to_string();
            self.slash_menu.reset_selection_for_query(&query);
        } else {
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

    pub(crate) fn input_cursor_to_line_start(&mut self) {
        let chars: Vec<char> = self.input_buffer.chars().collect();
        let mut pos = self.cursor_pos;
        while pos > 0 && chars[pos - 1] != '\n' {
            pos -= 1;
        }
        self.cursor_pos = pos;
    }

    pub(crate) fn input_cursor_to_line_end(&mut self) {
        let total = self.input_buffer.chars().count();
        let chars: Vec<char> = self.input_buffer.chars().collect();
        let mut pos = self.cursor_pos;
        while pos < total && chars[pos] != '\n' {
            pos += 1;
        }
        self.cursor_pos = pos;
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

    pub(crate) fn open_connect_picker(&mut self, data: &talos_conversation::ConnectPickerData) {
        self.slash_menu = BottomPanelState::open_connect_picker(data);
    }

    pub(crate) fn open_credential_input(
        &mut self,
        provider: &str,
        model_id: Option<&str>,
        connect_mode: bool,
        default_base_url: Option<String>,
    ) {
        self.slash_menu = BottomPanelState::open_credential_input(
            provider,
            model_id,
            connect_mode,
            default_base_url,
        );
    }

    pub(crate) fn credential_append_char(&mut self, ch: char) {
        if self.slash_menu.is_credential_input() {
            match self.slash_menu.credential_field {
                CredentialField::ApiKey => self.slash_menu.credential_buffer.push(ch),
                CredentialField::BaseUrl => self.slash_menu.base_url_buffer.push(ch),
            }
        }
    }

    pub(crate) fn credential_append_str(&mut self, text: &str) {
        if self.slash_menu.is_credential_input() {
            match self.slash_menu.credential_field {
                CredentialField::ApiKey => self.slash_menu.credential_buffer.push_str(text),
                CredentialField::BaseUrl => self.slash_menu.base_url_buffer.push_str(text),
            }
        }
    }

    pub(crate) fn credential_backspace(&mut self) {
        if self.slash_menu.is_credential_input() {
            match self.slash_menu.credential_field {
                CredentialField::ApiKey => {
                    self.slash_menu.credential_buffer.pop();
                }
                CredentialField::BaseUrl => {
                    self.slash_menu.base_url_buffer.pop();
                }
            }
        }
    }

    /// Submits the currently focused credential field.
    ///
    /// In `connect_mode`, standard providers with a default endpoint submit
    /// after the API key. Providers without a default endpoint advance to the
    /// base URL field and require a URL before returning the response.
    /// Non-connect mode preserves the original single-field behavior.
    pub(crate) fn credential_submit(&mut self) -> Option<CredentialResponseData> {
        if !self.slash_menu.is_credential_input() {
            return None;
        }
        let (provider, model_id, connect_mode, default_base_url) = match &self.slash_menu.kind {
            Some(PanelKind::CredentialInput {
                provider,
                model_id,
                connect_mode,
                default_base_url,
            }) => (
                provider.clone(),
                model_id.clone(),
                *connect_mode,
                default_base_url.clone(),
            ),
            _ => return None,
        };

        let key = self.slash_menu.credential_buffer.trim().to_string();
        if key.is_empty() {
            self.slash_menu.close();
            return None;
        }

        if connect_mode && self.slash_menu.credential_field == CredentialField::ApiKey {
            if default_base_url.is_some() {
                self.slash_menu.close();
                return Some(CredentialResponseData {
                    provider,
                    api_key: key,
                    model_id,
                    connect_mode,
                    base_url: default_base_url,
                });
            }
            self.slash_menu.credential_field = CredentialField::BaseUrl;
            return None;
        }

        let key = std::mem::take(&mut self.slash_menu.credential_buffer)
            .trim()
            .to_string();
        let typed_base_url = std::mem::take(&mut self.slash_menu.base_url_buffer)
            .trim()
            .to_string();
        if connect_mode && default_base_url.is_none() && typed_base_url.is_empty() {
            self.slash_menu.credential_buffer = key;
            return None;
        }
        let base_url = if typed_base_url.is_empty() {
            default_base_url
        } else {
            Some(typed_base_url)
        };
        self.slash_menu.close();
        Some(CredentialResponseData {
            provider,
            api_key: key,
            model_id,
            connect_mode,
            base_url,
        })
    }

    pub(crate) fn credential_cancel(&mut self) {
        self.slash_menu.credential_buffer.clear();
        self.slash_menu.base_url_buffer.clear();
        self.slash_menu.close();
    }

    pub(crate) fn slash_query(&self) -> &str {
        self.input_buffer.strip_prefix('/').unwrap_or_default()
    }

    /// Returns the active search query for the currently open panel.
    ///
    /// `SlashCommand` strips the leading `/` (matching `slash_query`).
    /// Picker kinds use the raw composer text as the "type to filter" query
    /// since pickers have no `/` prefix convention.
    pub(crate) fn panel_query(&self) -> &str {
        if self.slash_menu.is_slash() {
            self.slash_query()
        } else if self.slash_menu.is_picker() {
            self.input_buffer.as_str()
        } else {
            ""
        }
    }

    pub(crate) fn append_slash_query_char(&mut self, ch: char) {
        self.input_append_char(ch);
        let query = self.panel_query().to_string();
        self.slash_menu.reset_selection_for_query(&query);
    }

    pub(crate) fn backspace_slash_query(&mut self) {
        self.input_backspace();
        if self.slash_menu.is_slash() && !self.input_buffer.starts_with('/') {
            self.slash_menu.close();
            return;
        }
        let query = self.panel_query().to_string();
        self.slash_menu.reset_selection_for_query(&query);
    }

    pub(crate) fn accept_selected_panel_item(&mut self) -> PanelAction {
        self.accept_selected_panel_item_with_mode(PanelAcceptMode::Enter)
    }

    pub(crate) fn complete_selected_panel_item(&mut self) -> PanelAction {
        self.accept_selected_panel_item_with_mode(PanelAcceptMode::Complete)
    }

    fn accept_selected_panel_item_with_mode(&mut self, mode: PanelAcceptMode) -> PanelAction {
        let action = match self.slash_menu.items.get(self.slash_menu.selected_index) {
            Some(item) => item.action.clone(),
            None => return PanelAction::None,
        };

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
            PanelItemAction::ConnectSelect { provider } => {
                self.slash_menu.close();
                PanelAction::ConnectSelect { provider }
            }
            PanelItemAction::OpenWizard => {
                self.slash_menu = crate::panel_state::BottomPanelState::open_provider_wizard();
                PanelAction::None
            }
            PanelItemAction::OpenModelList { provider } => {
                if let Some(data) = self.slash_menu.model_picker_data.clone() {
                    self.slash_menu =
                        crate::panel_state::BottomPanelState::open_model_list(&provider, &data);
                }
                PanelAction::None
            }
            PanelItemAction::OpenVariantPicker {
                provider,
                model_id,
                variants,
            } => {
                let data = self.slash_menu.model_picker_data.clone();
                self.slash_menu = crate::panel_state::BottomPanelState::open_variant_picker(
                    provider, model_id, variants, data,
                );
                PanelAction::None
            }
            PanelItemAction::SwitchModel {
                provider,
                model_id,
                variant,
                ..
            } => {
                self.slash_menu.close();
                PanelAction::SwitchModel {
                    provider,
                    model_id,
                    variant,
                }
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
        if !content.is_empty() {
            self.record_history(&content);
        }
        self.input_clear();
        self.history_cursor = None;
        self.draft_input.clear();
        content
    }

    /// Record a submitted input, deduplicating consecutive duplicates (TUI-030).
    pub(crate) fn record_history(&mut self, input: &str) {
        if self.input_history.last().is_some_and(|last| last == input) {
            return;
        }
        self.input_history.push(input.to_string());
    }

    /// Navigate to the previous (older) history entry (TUI-030).
    pub(crate) fn history_prev(&mut self) {
        if self.input_history.is_empty() {
            return;
        }
        match self.history_cursor {
            None => {
                self.draft_input = self.input_buffer.clone();
                self.history_cursor = Some(self.input_history.len() - 1);
            }
            Some(0) => return,
            Some(i) => self.history_cursor = Some(i - 1),
        }
        self.load_history_entry();
    }

    /// Navigate to the next (newer) entry or restore the draft (TUI-030).
    pub(crate) fn history_next(&mut self) {
        match self.history_cursor {
            None => {}
            Some(i) => {
                if i + 1 >= self.input_history.len() {
                    self.history_cursor = None;
                    self.input_buffer = self.draft_input.clone();
                    self.draft_input.clear();
                    self.cursor_pos = self.input_buffer.chars().count();
                } else {
                    self.history_cursor = Some(i + 1);
                    self.load_history_entry();
                }
            }
        }
    }

    fn load_history_entry(&mut self) {
        if let Some(i) = self.history_cursor {
            self.input_buffer = self.input_history[i].clone();
            self.cursor_pos = self.input_buffer.chars().count();
        }
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

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
