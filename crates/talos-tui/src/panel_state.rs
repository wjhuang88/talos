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
    SwitchModel {
        provider: String,
        model_id: String,
        variant: Option<String>,
    },
    ConnectSelect {
        provider: String,
    },
    RegisterCustomProvider {
        name: String,
        protocol: String,
        base_url: String,
        api_key: String,
    },
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
    /// Level 1 → Level 2: open the model list scoped to this provider.
    OpenModelList { provider: String },
    /// Level 2 → Level 3: open the variant list for this `(provider, model)`.
    OpenVariantPicker {
        provider: String,
        model_id: String,
        variants: Vec<talos_conversation::ModelPickerVariantItem>,
    },
    /// Direct switch (used by Recent items, variant-less models, and variant
    /// selections). `variant` carries the recorded/selected variant, if any.
    /// `provider` and `model_id` are a structured identity; `model_id` remains
    /// the opaque provider-side identifier.
    SwitchModel {
        provider: String,
        model_id: String,
        variant: Option<String>,
    },
    /// `/connect` picker selection — carries the provider name structurally
    /// so the TUI can emit `UserInput::ConnectSelect` instead of reserializing
    /// to `/connect name` command text (TUI-033).
    ConnectSelect { provider: String },
    /// Open the custom provider wizard (MODEL-008-A/I147).
    OpenWizard,
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
    /// `/model` Level 1: Recent (optional) + provider list.
    ModelPicker,
    /// `/model` Level 2: models scoped to a single provider.
    ModelList {
        provider: String,
    },
    ConnectPicker,
    CredentialInput {
        provider: String,
        model_id: Option<String>,
        connect_mode: bool,
        default_base_url: Option<String>,
    },
    /// Custom provider wizard (MODEL-008-A/I147). Five-step state machine:
    /// Name → Protocol → BaseUrl → ApiKey → Confirm.
    ProviderWizard {
        step: WizardStep,
        name: String,
        protocol: String,
        base_url: String,
        api_key: String,
        is_update: bool,
    },
    Approval {
        tool_name: String,
        arguments: String,
    },
    /// `/model` Level 3: variants for a single `(provider, model)`.
    VariantPicker {
        provider: String,
        model_id: String,
        variants: Vec<talos_conversation::ModelPickerVariantItem>,
    },
}

/// Wizard step for the custom provider registration flow (MODEL-008-A/I147).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WizardStep {
    Name,
    Protocol,
    BaseUrl,
    ApiKey,
    Confirm,
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
    /// Retained while any `/model` level is active so Level 2/3 can derive
    /// their items without an engine round-trip. `None` for non-ModelPicker panels.
    pub(crate) model_picker_data: Option<ModelPickerData>,
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
            model_picker_data: None,
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
            model_picker_data: None,
        }
    }

    pub(crate) fn open_model_picker(data: &ModelPickerData) -> Self {
        let mut panel_items: Vec<PanelItem> = Vec::new();

        // Level 1 layout: Recent (optional, separated top region) + Providers.
        // The status bar already shows the active model, so there is no
        // separate "Current" group. Recent items direct-switch (using their
        // recorded variant); provider rows enter Level 2.
        if !data.recent.is_empty() {
            panel_items.push(PanelItem {
                label: "Recent".into(),
                description: String::new(),
                action: PanelItemAction::Header,
                is_current: false,
            });
            panel_items.extend(data.recent.iter().map(|m| PanelItem {
                label: m.label.clone(),
                description: m.provider.clone(),
                action: PanelItemAction::SwitchModel {
                    provider: m.provider.clone(),
                    model_id: m.model_id.clone(),
                    variant: m.variant.clone(),
                },
                is_current: false,
            }));
        }

        // Deduplicate providers from ready_models. Each provider row enters Level 2.
        let mut providers: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
        for m in &data.ready_models {
            providers.insert(m.provider.as_str());
        }

        if !providers.is_empty() {
            panel_items.push(PanelItem {
                label: "Providers".into(),
                description: String::new(),
                action: PanelItemAction::Header,
                is_current: false,
            });
            panel_items.extend(providers.iter().map(|provider| {
                let count = data
                    .ready_models
                    .iter()
                    .filter(|m| m.provider == *provider)
                    .count();
                PanelItem {
                    label: (*provider).to_string(),
                    description: format!("{} model{}", count, if count == 1 { "" } else { "s" }),
                    action: PanelItemAction::OpenModelList {
                        provider: (*provider).to_string(),
                    },
                    is_current: false,
                }
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
            .position(|i| i.action != PanelItemAction::Header)
            .unwrap_or(0);
        Self {
            is_open: true,
            kind: Some(PanelKind::ModelPicker),
            items: panel_items,
            selected_index: initial_index,
            credential_buffer: String::new(),
            base_url_buffer: String::new(),
            credential_field: CredentialField::ApiKey,
            model_picker_data: Some(data.clone()),
        }
    }

    /// Level 2: build a picker of models scoped to a single provider. Items
    /// either direct-switch (no variants) or open Level 3 (has variants).
    /// Requires `model_picker_data` to be populated by a prior `open_model_picker`.
    pub(crate) fn open_model_list(provider: &str, data: &ModelPickerData) -> Self {
        let models: Vec<&ModelPickerItem> = data
            .ready_models
            .iter()
            .filter(|m| m.provider == provider)
            .collect();

        let panel_items: Vec<PanelItem> = models
            .iter()
            .map(|m| {
                let action = if m.variants.is_empty() {
                    PanelItemAction::SwitchModel {
                        provider: m.provider.clone(),
                        model_id: m.model_id.clone(),
                        variant: None,
                    }
                } else {
                    PanelItemAction::OpenVariantPicker {
                        provider: m.provider.clone(),
                        model_id: m.model_id.clone(),
                        variants: m.variants.clone(),
                    }
                };
                PanelItem {
                    label: m.label.clone(),
                    description: m.pricing.clone().unwrap_or_default(),
                    action,
                    is_current: m.is_current,
                }
            })
            .collect();

        let initial_index = panel_items
            .iter()
            .position(|i| !i.is_current)
            .or_else(|| {
                panel_items
                    .iter()
                    .position(|i| i.action != PanelItemAction::Header)
            })
            .unwrap_or(0);

        Self {
            is_open: true,
            kind: Some(PanelKind::ModelList {
                provider: provider.to_string(),
            }),
            items: panel_items,
            selected_index: initial_index,
            credential_buffer: String::new(),
            base_url_buffer: String::new(),
            credential_field: CredentialField::ApiKey,
            model_picker_data: Some(data.clone()),
        }
    }

    pub(crate) fn open_connect_picker(data: &talos_conversation::ConnectPickerData) -> Self {
        let mut panel_items: Vec<PanelItem> = Vec::new();

        panel_items.push(PanelItem {
            label: "Add custom provider".into(),
            description: "Register an OpenAI-compatible or Anthropic-compatible gateway".into(),
            action: PanelItemAction::OpenWizard,
            is_current: false,
        });

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
                    action: PanelItemAction::ConnectSelect {
                        provider: p.provider.clone(),
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
                    action: PanelItemAction::ConnectSelect {
                        provider: p.provider.clone(),
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
            model_picker_data: None,
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
                model_id: model_id.map(str::to_string),
                connect_mode,
                default_base_url,
            }),
            items: Vec::new(),
            selected_index: 0,
            credential_buffer: String::new(),
            base_url_buffer: String::new(),
            credential_field: CredentialField::ApiKey,
            model_picker_data: None,
        }
    }

    pub(crate) fn open_variant_picker(
        provider: String,
        model_id: String,
        variants: Vec<talos_conversation::ModelPickerVariantItem>,
        data: Option<ModelPickerData>,
    ) -> Self {
        let items = variants
            .iter()
            .map(|v| PanelItem {
                label: v.label.clone(),
                description: v.variant_id.clone(),
                action: PanelItemAction::SwitchModel {
                    provider: provider.clone(),
                    model_id: model_id.clone(),
                    variant: Some(v.variant_id.clone()),
                },
                is_current: false,
            })
            .collect();
        Self {
            is_open: true,
            kind: Some(PanelKind::VariantPicker {
                provider,
                model_id,
                variants,
            }),
            items,
            selected_index: 0,
            credential_buffer: String::new(),
            base_url_buffer: String::new(),
            credential_field: CredentialField::ApiKey,
            model_picker_data: data,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn open_provider_wizard() -> Self {
        Self {
            is_open: true,
            kind: Some(PanelKind::ProviderWizard {
                step: WizardStep::Name,
                name: String::new(),
                protocol: String::new(),
                base_url: String::new(),
                api_key: String::new(),
                is_update: false,
            }),
            items: Vec::new(),
            selected_index: 0,
            credential_buffer: String::new(),
            base_url_buffer: String::new(),
            credential_field: CredentialField::ApiKey,
            model_picker_data: None,
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
                | Some(PanelKind::ModelList { .. })
                | Some(PanelKind::VariantPicker { .. })
                | Some(PanelKind::ConnectPicker)
        )
    }

    pub(crate) fn is_approval(&self) -> bool {
        matches!(self.kind, Some(PanelKind::Approval { .. }))
    }

    pub(crate) fn is_credential_input(&self) -> bool {
        matches!(self.kind, Some(PanelKind::CredentialInput { .. }))
    }

    pub(crate) fn is_provider_wizard(&self) -> bool {
        matches!(self.kind, Some(PanelKind::ProviderWizard { .. }))
    }

    pub(crate) fn is_variant_picker(&self) -> bool {
        matches!(self.kind, Some(PanelKind::VariantPicker { .. }))
    }

    pub(crate) fn is_model_list(&self) -> bool {
        matches!(self.kind, Some(PanelKind::ModelList { .. }))
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
            model_picker_data: None,
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
