//! Bottom panel state — slash command menu, pickers, approval overlay, credential input.

use talos_conversation::{
    CommandExecutionMode, ModelPickerData, ModelPickerItem, SessionPickerItem,
};

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
        Self {
            is_open: true,
            kind: Some(PanelKind::Approval {
                tool_name: tool_name.to_string(),
                arguments: arguments.to_string(),
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
            let command_name = |item: &PanelItem| -> String {
                item.label
                    .strip_prefix('/')
                    .unwrap_or(&item.label)
                    .to_lowercase()
            };
            let mut prefix_matches = Vec::new();
            let mut other_matches = Vec::new();
            for (i, item) in self.items.iter().enumerate() {
                if command_name(item).starts_with(&lower) {
                    prefix_matches.push(i);
                } else if item_matches(item) {
                    other_matches.push(i);
                }
            }
            prefix_matches.extend(other_matches);
            return prefix_matches;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PanelAcceptMode {
    Enter,
    Complete,
}
