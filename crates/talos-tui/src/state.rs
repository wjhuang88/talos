//! TUI state machine — pure UI state only.
//!
//! Business logic (messages, streaming, queues) lives in `talos-conversation`.
//! This module owns only input handling, approval overlay, and display state.

use std::time::{Duration, Instant};

use talos_conversation::{
    CommandExecutionMode, CredentialResponseData, ModelPickerData, ModelPickerItem,
    SessionPickerItem, StatusSnapshot, TipKind,
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
    ConnectPicker,
    CredentialInput {
        provider: String,
        model_id: Option<String>,
        connect_mode: bool,
        default_base_url: Option<String>,
    },
    Approval {
        tool_name: String,
        arguments: String,
    },
}

/// Which input field the credential panel is currently editing.
///
/// Only relevant when `connect_mode` is `true`: normal model-credential setup
/// stays on `ApiKey` for its single-field flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum CredentialField {
    #[default]
    ApiKey,
    BaseUrl,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct BottomPanelState {
    pub(crate) is_open: bool,
    pub(crate) kind: Option<PanelKind>,
    pub(crate) items: Vec<PanelItem>,
    pub(crate) selected_index: usize,
    pub(crate) credential_buffer: String,
    pub(crate) base_url_buffer: String,
    pub(crate) credential_field: CredentialField,
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
            base_url_buffer: String::new(),
            credential_field: CredentialField::ApiKey,
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
            base_url_buffer: String::new(),
            credential_field: CredentialField::ApiKey,
        }
    }

    pub(crate) fn open_model_picker(data: &ModelPickerData) -> Self {
        let mut panel_items: Vec<PanelItem> = Vec::new();

        let (current_models, other_ready): (Vec<&ModelPickerItem>, Vec<&ModelPickerItem>) =
            data.ready_models.iter().partition(|m| m.is_current);

        if !current_models.is_empty() {
            panel_items.push(PanelItem {
                label: "Current".into(),
                description: String::new(),
                action: PanelItemAction::Header,
                is_current: false,
            });
            panel_items.extend(current_models.iter().map(|m| PanelItem {
                label: m.label.clone(),
                description: m.provider.clone(),
                action: PanelItemAction::Select {
                    command: m.command.clone(),
                    value: m.model_id.clone(),
                },
                is_current: true,
            }));
        }

        let mut provider_groups: std::collections::BTreeMap<&str, Vec<&ModelPickerItem>> =
            std::collections::BTreeMap::new();
        for m in &other_ready {
            provider_groups
                .entry(m.provider.as_str())
                .or_default()
                .push(m);
        }

        for (provider, models) in &provider_groups {
            panel_items.push(PanelItem {
                label: (*provider).to_string(),
                description: format!(
                    "{} model{}",
                    models.len(),
                    if models.len() == 1 { "" } else { "s" }
                ),
                action: PanelItemAction::Header,
                is_current: false,
            });
            panel_items.extend(models.iter().map(|m| PanelItem {
                label: m.label.clone(),
                description: m.provider.clone(),
                action: PanelItemAction::Select {
                    command: m.command.clone(),
                    value: m.model_id.clone(),
                },
                is_current: false,
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
                description: "Use /connect to set up".to_string(),
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
            base_url_buffer: String::new(),
            credential_field: CredentialField::ApiKey,
        }
    }

    pub(crate) fn open_connect_picker(data: &talos_conversation::ConnectPickerData) -> Self {
        let mut panel_items: Vec<PanelItem> = Vec::new();

        if !data.connected.is_empty() {
            panel_items.push(PanelItem {
                label: "Connected".into(),
                description: String::new(),
                action: PanelItemAction::Header,
                is_current: false,
            });
            panel_items.extend(data.connected.iter().map(|p| {
                let cred_label = if p.has_credential {
                    "credential present"
                } else {
                    ""
                };
                let url_label = p.api_base_url.as_deref().unwrap_or("");
                let desc = if cred_label.is_empty() && url_label.is_empty() {
                    format!("{} models", p.model_count)
                } else if url_label.is_empty() {
                    format!("{} models   {}", p.model_count, cred_label)
                } else {
                    format!("{} models   {}   {}", p.model_count, cred_label, url_label)
                };
                PanelItem {
                    label: format!("{}   {}", p.name, p.provider),
                    description: desc,
                    action: PanelItemAction::Select {
                        command: "/connect".to_string(),
                        value: p.provider.clone(),
                    },
                    is_current: false,
                }
            }));
        }

        if !data.available.is_empty() {
            panel_items.push(PanelItem {
                label: "Available".into(),
                description: String::new(),
                action: PanelItemAction::Header,
                is_current: false,
            });
            panel_items.extend(data.available.iter().map(|p| {
                let url_label = p.api_base_url.as_deref().unwrap_or("—");
                PanelItem {
                    label: format!("{}   {}", p.name, p.provider),
                    description: format!("{} models   {}", p.model_count, url_label),
                    action: PanelItemAction::Select {
                        command: "/connect".to_string(),
                        value: p.provider.clone(),
                    },
                    is_current: false,
                }
            }));
        }

        let initial_index = panel_items
            .iter()
            .position(|i| i.action != PanelItemAction::Header)
            .unwrap_or(0);

        Self {
            is_open: true,
            kind: Some(PanelKind::ConnectPicker),
            items: panel_items,
            selected_index: initial_index,
            credential_buffer: String::new(),
            base_url_buffer: String::new(),
            credential_field: CredentialField::ApiKey,
        }
    }

    pub(crate) fn open_credential_input(
        provider: &str,
        model_id: Option<&str>,
        connect_mode: bool,
        default_base_url: Option<String>,
    ) -> Self {
        Self {
            is_open: true,
            kind: Some(PanelKind::CredentialInput {
                provider: provider.to_string(),
                model_id: model_id.map(|s| s.to_string()),
                connect_mode,
                default_base_url,
            }),
            items: vec![],
            selected_index: 0,
            credential_buffer: String::new(),
            base_url_buffer: String::new(),
            credential_field: CredentialField::ApiKey,
        }
    }

    pub(crate) fn is_slash(&self) -> bool {
        self.kind == Some(PanelKind::SlashCommand)
    }

    pub(crate) fn is_picker(&self) -> bool {
        matches!(
            self.kind,
            Some(PanelKind::SessionPicker)
                | Some(PanelKind::ModelPicker)
                | Some(PanelKind::ConnectPicker)
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
            base_url_buffer: String::new(),
            credential_field: CredentialField::ApiKey,
        }
    }

    /// Returns indices into `self.items` visible under `query`.
    ///
    /// `SlashCommand` performs flat substring matching. Picker kinds are
    /// grouped by `PanelItemAction::Header` delimiters: a group's header is
    /// included only when at least one sibling item matches; groups with no
    /// matches (including a "Current" pseudo-group) are hidden entirely.
    /// Headers never match independently — only as a byproduct of a
    /// matching sibling.
    pub(crate) fn filtered_indices(&self, query: &str) -> Vec<usize> {
        if query.is_empty() {
            return (0..self.items.len()).collect();
        }

        let lower = query.to_lowercase();
        let item_matches = |item: &PanelItem| -> bool {
            item.label
                .strip_prefix('/')
                .unwrap_or(&item.label)
                .to_lowercase()
                .contains(&lower)
                || item.description.to_lowercase().contains(&lower)
        };

        if self.is_slash() {
            return self
                .items
                .iter()
                .enumerate()
                .filter(|(_, item)| item_matches(item))
                .map(|(i, _)| i)
                .collect();
        }

        let mut result = Vec::new();
        let mut i = 0;
        while i < self.items.len() {
            if self.items[i].action == PanelItemAction::Header {
                let header_idx = i;
                i += 1;
                let group_start = i;
                while i < self.items.len() && self.items[i].action != PanelItemAction::Header {
                    i += 1;
                }
                let group_matches: Vec<usize> = (group_start..i)
                    .filter(|&j| item_matches(&self.items[j]))
                    .collect();
                if !group_matches.is_empty() {
                    result.push(header_idx);
                    result.extend(group_matches);
                }
            } else {
                if item_matches(&self.items[i]) {
                    result.push(i);
                }
                i += 1;
            }
        }
        result
    }

    pub(crate) fn filtered_items(&self, query: &str) -> Vec<&PanelItem> {
        self.filtered_indices(query)
            .into_iter()
            .map(|i| &self.items[i])
            .collect()
    }

    /// Returns the navigable (non-`Header`) indices visible under `query`.
    fn navigable_indices(&self, query: &str) -> Vec<usize> {
        self.filtered_indices(query)
            .into_iter()
            .filter(|&i| self.items[i].action != PanelItemAction::Header)
            .collect()
    }

    pub(crate) fn close(&mut self) {
        self.is_open = false;
        self.kind = None;
        self.items.clear();
        self.selected_index = 0;
    }

    pub(crate) fn select_next(&mut self, query: &str) {
        let navigable = self.navigable_indices(query);
        if navigable.is_empty() {
            return;
        }
        let next_pos = match navigable.iter().position(|&i| i == self.selected_index) {
            Some(pos) => (pos + 1) % navigable.len(),
            None => 0,
        };
        self.selected_index = navigable[next_pos];
    }

    pub(crate) fn select_prev(&mut self, query: &str) {
        let navigable = self.navigable_indices(query);
        if navigable.is_empty() {
            return;
        }
        let prev_pos = match navigable.iter().position(|&i| i == self.selected_index) {
            Some(pos) => {
                if pos == 0 {
                    navigable.len() - 1
                } else {
                    pos - 1
                }
            }
            None => 0,
        };
        self.selected_index = navigable[prev_pos];
    }

    /// Resets `selected_index` to the first navigable item under `query`.
    ///
    /// Called whenever the query changes so a stale selection (now filtered
    /// out, or landed on a `Header`) does not leave the panel with an
    /// invalid or non-navigable selection.
    pub(crate) fn reset_selection_for_query(&mut self, query: &str) {
        let navigable = self.navigable_indices(query);
        self.selected_index = navigable.first().copied().unwrap_or(0);
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
    pub thinking_preview: Option<String>,
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
    /// In `connect_mode`, the first submit (API key) advances to the base
    /// URL field instead of closing the panel; the second submit resolves
    /// the final `base_url` (typed value, else the request's
    /// `default_base_url`, else `None`) and returns the full response.
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

        if connect_mode && self.slash_menu.credential_field == CredentialField::ApiKey {
            let key = self.slash_menu.credential_buffer.trim().to_string();
            if key.is_empty() {
                self.slash_menu.close();
                return None;
            }
            self.slash_menu.credential_field = CredentialField::BaseUrl;
            return None;
        }

        let key = std::mem::take(&mut self.slash_menu.credential_buffer)
            .trim()
            .to_string();
        if key.is_empty() {
            self.slash_menu.close();
            return None;
        }
        let typed_base_url = std::mem::take(&mut self.slash_menu.base_url_buffer)
            .trim()
            .to_string();
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

        state.open_credential_input("openai", None, false, None);
        state.credential_append_str("sk-test-key\n");

        let response = state.credential_submit().expect("credential response");
        assert_eq!(response.provider, "openai");
        assert_eq!(response.api_key, "sk-test-key");
        assert_eq!(response.model_id, None);
        assert!(!response.connect_mode);
        assert!(response.base_url.is_none());
        assert!(!state.slash_menu.is_open);
    }

    #[test]
    fn empty_credential_submit_closes_without_response() {
        let mut state = TuiState::new();

        state.open_credential_input("openai", Some("gpt-4.1"), false, None);

        assert!(state.credential_submit().is_none());
        assert!(!state.slash_menu.is_open);
    }

    // ── /connect two-phase credential (api_key + base_url) ──────────────

    #[test]
    fn connect_mode_first_submit_advances_to_base_url_field() {
        let mut state = TuiState::new();
        state.open_credential_input("groq", None, true, None);

        state.credential_append_str("gsk-test-key");
        let response = state.credential_submit();

        assert!(
            response.is_none(),
            "first Enter in connect_mode must not submit yet"
        );
        assert!(
            state.slash_menu.is_open,
            "panel must stay open for base_url"
        );
        assert_eq!(
            state.slash_menu.credential_field,
            crate::state::CredentialField::BaseUrl
        );
        assert_eq!(state.slash_menu.credential_buffer, "gsk-test-key");
    }

    #[test]
    fn connect_mode_second_submit_returns_typed_base_url() {
        let mut state = TuiState::new();
        state.open_credential_input("groq", None, true, None);

        state.credential_append_str("gsk-test-key");
        state.credential_submit();
        state.credential_append_str("https://custom.groq.example/v1");
        let response = state
            .credential_submit()
            .expect("second submit must return response");

        assert_eq!(response.provider, "groq");
        assert_eq!(response.api_key, "gsk-test-key");
        assert!(response.connect_mode);
        assert_eq!(
            response.base_url.as_deref(),
            Some("https://custom.groq.example/v1")
        );
        assert!(!state.slash_menu.is_open);
    }

    #[test]
    fn connect_mode_empty_base_url_falls_back_to_default() {
        let mut state = TuiState::new();
        state.open_credential_input(
            "groq",
            None,
            true,
            Some("https://api.groq.com/openai/v1".to_string()),
        );

        state.credential_append_str("gsk-test-key");
        state.credential_submit();
        // Base URL left blank.
        let response = state
            .credential_submit()
            .expect("blank base_url must still submit using the default");

        assert_eq!(
            response.base_url.as_deref(),
            Some("https://api.groq.com/openai/v1")
        );
    }

    #[test]
    fn connect_mode_empty_base_url_with_no_default_is_none() {
        let mut state = TuiState::new();
        state.open_credential_input("groq", None, true, None);

        state.credential_append_str("gsk-test-key");
        state.credential_submit();
        let response = state
            .credential_submit()
            .expect("blank base_url with no default must still submit");

        assert!(response.base_url.is_none());
    }

    #[test]
    fn connect_mode_empty_api_key_cancels_without_advancing() {
        let mut state = TuiState::new();
        state.open_credential_input("groq", None, true, None);

        let response = state.credential_submit();

        assert!(response.is_none());
        assert!(
            !state.slash_menu.is_open,
            "empty API key in connect_mode must cancel, not advance"
        );
    }

    #[test]
    fn non_connect_mode_ignores_base_url_and_submits_single_phase() {
        let mut state = TuiState::new();
        state.open_credential_input("anthropic", None, false, None);

        state.credential_append_str("sk-ant-test");
        let response = state
            .credential_submit()
            .expect("non-connect mode must submit on first Enter");

        assert_eq!(response.api_key, "sk-ant-test");
        assert!(response.base_url.is_none());
        assert!(!response.connect_mode);
    }

    #[test]
    fn credential_append_and_backspace_route_to_active_field() {
        let mut state = TuiState::new();
        state.open_credential_input("groq", None, true, None);

        state.credential_append_str("abc");
        state.credential_backspace();
        assert_eq!(state.slash_menu.credential_buffer, "ab");
        assert!(state.slash_menu.base_url_buffer.is_empty());

        state.credential_append_str("x");
        state.credential_submit(); // advance to BaseUrl
        state.credential_append_str("https://x.example");
        state.credential_backspace();

        assert_eq!(state.slash_menu.credential_buffer, "abx");
        assert_eq!(state.slash_menu.base_url_buffer, "https://x.exampl");
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

    // ── MC106: Group-aware search filtering ─────────────────────────────

    fn model_item(id: &str, provider: &str, is_current: bool) -> ModelPickerItem {
        ModelPickerItem {
            command: "/model".to_string(),
            model_id: id.to_string(),
            provider: provider.to_string(),
            label: format!("{id} {provider}"),
            context_limit: Some(100_000),
            pricing: None,
            authenticated: true,
            is_current,
        }
    }

    fn sample_model_picker_data() -> ModelPickerData {
        ModelPickerData {
            ready_models: vec![
                model_item("claude-sonnet-4-5", "anthropic", true),
                model_item("claude-opus-4-1", "anthropic", false),
                model_item("gpt-4o", "openai", false),
                model_item("o3", "openai", false),
            ],
            setup_providers: vec![],
        }
    }

    fn connect_item(
        provider: &str,
        name: &str,
        has_credential: bool,
    ) -> talos_conversation::ConnectPickerItem {
        talos_conversation::ConnectPickerItem {
            provider: provider.to_string(),
            name: name.to_string(),
            model_count: 3,
            api_base_url: None,
            has_credential,
            doc_url: None,
        }
    }

    fn sample_connect_picker_data() -> talos_conversation::ConnectPickerData {
        talos_conversation::ConnectPickerData {
            connected: vec![connect_item("anthropic", "Anthropic", true)],
            available: vec![
                connect_item("openai", "OpenAI", false),
                connect_item("groq", "Groq", false),
            ],
        }
    }

    #[test]
    fn model_picker_search_matching_provider_hides_other_groups() {
        let data = sample_model_picker_data();
        let menu = BottomPanelState::open_model_picker(&data);

        // Groups: "Current" (claude-sonnet-4-5/anthropic), "anthropic"
        // (claude-opus-4-1), "openai" (gpt-4o, o3).
        let indices = menu.filtered_indices("gpt");
        let visible_labels: Vec<&str> = indices
            .iter()
            .map(|&i| menu.items[i].label.as_str())
            .collect();

        assert!(
            visible_labels.contains(&"openai"),
            "openai header must be visible: {visible_labels:?}"
        );
        assert!(
            visible_labels.iter().any(|l| l.contains("gpt-4o")),
            "matching item must be visible: {visible_labels:?}"
        );
        assert!(
            !visible_labels.iter().any(|l| l.contains("o3")),
            "non-matching sibling must be hidden: {visible_labels:?}"
        );
        assert!(
            !visible_labels.contains(&"Current"),
            "non-matching Current group must be hidden: {visible_labels:?}"
        );
        assert!(
            !visible_labels.iter().any(|l| l.contains("claude")),
            "non-matching anthropic group must be hidden entirely: {visible_labels:?}"
        );
    }

    #[test]
    fn model_picker_search_no_match_hides_all_groups() {
        let data = sample_model_picker_data();
        let menu = BottomPanelState::open_model_picker(&data);

        let indices = menu.filtered_indices("zzz-nonexistent");
        assert!(indices.is_empty(), "no groups should match: {indices:?}");
    }

    #[test]
    fn model_picker_empty_query_shows_everything() {
        let data = sample_model_picker_data();
        let menu = BottomPanelState::open_model_picker(&data);

        let indices = menu.filtered_indices("");
        assert_eq!(indices.len(), menu.items.len());
    }

    #[test]
    fn model_picker_navigation_skips_headers_and_filtered_out_items() {
        let data = sample_model_picker_data();
        let mut menu = BottomPanelState::open_model_picker(&data);

        // Filter to only the "openai" group (gpt-4o, o3).
        menu.selected_index = menu
            .filtered_indices("openai")
            .into_iter()
            .find(|&i| menu.items[i].action != PanelItemAction::Header)
            .unwrap();

        let first_selection = menu.selected_index;
        assert_eq!(
            menu.items[first_selection].action != PanelItemAction::Header,
            true
        );

        menu.select_next("openai");
        assert_ne!(
            menu.items[menu.selected_index].action,
            PanelItemAction::Header,
            "select_next must never land on a Header"
        );
        assert_ne!(
            menu.selected_index, first_selection,
            "select_next must move within the filtered openai group"
        );

        // Navigating past the last item in the filtered set wraps back
        // without ever landing on a hidden (anthropic) item or a header.
        menu.select_next("openai");
        let after_wrap = menu.selected_index;
        assert!(
            menu.items[after_wrap].label.contains("gpt-4o")
                || menu.items[after_wrap].label.contains("o3"),
            "wrapped selection must stay within the openai group, got {:?}",
            menu.items[after_wrap].label
        );
    }

    #[test]
    fn model_picker_select_next_prev_never_select_header() {
        let data = sample_model_picker_data();
        let mut menu = BottomPanelState::open_model_picker(&data);

        for _ in 0..(menu.items.len() * 2) {
            menu.select_next("");
            assert_ne!(
                menu.items[menu.selected_index].action,
                PanelItemAction::Header
            );
        }
        for _ in 0..(menu.items.len() * 2) {
            menu.select_prev("");
            assert_ne!(
                menu.items[menu.selected_index].action,
                PanelItemAction::Header
            );
        }
    }

    #[test]
    fn model_picker_enter_selects_correct_original_item_after_filtering() {
        let data = sample_model_picker_data();
        let mut menu = BottomPanelState::open_model_picker(&data);

        let target_idx = menu
            .items
            .iter()
            .position(
                |i| matches!(&i.action, PanelItemAction::Select { value, .. } if value == "gpt-4o"),
            )
            .expect("gpt-4o item must exist");
        menu.selected_index = target_idx;

        // selected_index must remain the correct raw index even though the
        // filtered/visible set has shrunk to a single group.
        let indices = menu.filtered_indices("gpt");
        assert!(indices.contains(&target_idx));

        let action = menu.items[menu.selected_index].action.clone();
        match action {
            PanelItemAction::Select { command, value } => {
                assert_eq!(command, "/model");
                assert_eq!(value, "gpt-4o");
            }
            other => panic!("expected Select action, got {other:?}"),
        }
    }

    #[test]
    fn connect_picker_search_matches_provider_group() {
        let data = sample_connect_picker_data();
        let menu = BottomPanelState::open_connect_picker(&data);

        let indices = menu.filtered_indices("groq");
        let labels: Vec<&str> = indices
            .iter()
            .map(|&i| menu.items[i].label.as_str())
            .collect();

        assert!(
            labels.contains(&"Available"),
            "matching group header must show: {labels:?}"
        );
        assert!(labels.iter().any(|l| l.contains("Groq")), "{labels:?}");
        assert!(
            !labels.iter().any(|l| l.contains("OpenAI")),
            "non-matching sibling in same group must be hidden: {labels:?}"
        );
        assert!(
            !labels.contains(&"Connected"),
            "non-matching Connected group must be hidden entirely: {labels:?}"
        );
    }

    #[test]
    fn connect_picker_is_picker_and_supports_filtering() {
        let data = sample_connect_picker_data();
        let menu = BottomPanelState::open_connect_picker(&data);
        assert!(menu.is_picker());
    }

    #[test]
    fn reset_selection_for_query_lands_on_first_navigable_match() {
        let data = sample_model_picker_data();
        let mut menu = BottomPanelState::open_model_picker(&data);

        menu.reset_selection_for_query("openai");
        assert_ne!(
            menu.items[menu.selected_index].action,
            PanelItemAction::Header
        );
        assert!(
            menu.items[menu.selected_index].label.contains("gpt-4o")
                || menu.items[menu.selected_index].label.contains("o3")
        );
    }

    #[test]
    fn reset_selection_for_query_falls_back_to_zero_when_nothing_matches() {
        let data = sample_model_picker_data();
        let mut menu = BottomPanelState::open_model_picker(&data);

        menu.reset_selection_for_query("zzz-nonexistent");
        assert_eq!(menu.selected_index, 0);
    }

    #[test]
    fn tuistate_panel_query_uses_raw_buffer_for_pickers() {
        let mut state = TuiState::new();
        let data = sample_model_picker_data();
        state.open_model_picker(&data);
        state.input_append_char('g');
        state.input_append_char('p');
        state.input_append_char('t');

        assert_eq!(state.panel_query(), "gpt");
    }

    #[test]
    fn tuistate_panel_query_strips_slash_for_slash_menu() {
        let mut state = TuiState::new();
        state.open_slash_menu(talos_conversation::command_registry());
        state.append_slash_query_char('m');
        state.append_slash_query_char('o');

        assert_eq!(state.panel_query(), "mo");
    }
}
