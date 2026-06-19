//! TUI state machine — pure UI state only.
//!
//! Business logic (messages, streaming, queues) lives in `talos-conversation`.
//! This module owns only input handling, approval overlay, and display state.

use std::time::{Duration, Instant};

use talos_conversation::{StatusSnapshot, TipKind};
use talos_core::ApprovalChoice;

pub(crate) const DOUBLE_CTRL_C_WINDOW: Duration = Duration::from_secs(2);
pub(crate) const SLASH_MENU_MAX_VISIBLE: usize = 8;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SlashMenuItem {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) arg_hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct SlashMenuState {
    pub(crate) is_open: bool,
    pub(crate) items: Vec<SlashMenuItem>,
    pub(crate) selected_index: usize,
    pub(crate) filter_text: String,
}

impl SlashMenuState {
    pub(crate) fn open(registry: &talos_conversation::CommandRegistry) -> Self {
        let items = registry
            .available_commands()
            .into_iter()
            .map(|cmd| SlashMenuItem {
                name: cmd.name.to_string(),
                description: cmd.description.to_string(),
                arg_hint: cmd.arg_hint.map(|h| h.to_string()),
            })
            .collect();
        Self {
            is_open: true,
            items,
            selected_index: 0,
            filter_text: String::new(),
        }
    }

    pub(crate) fn filtered_items(&self) -> Vec<&SlashMenuItem> {
        if self.filter_text.is_empty() {
            return self.items.iter().collect();
        }
        let lower = self.filter_text.to_lowercase();
        self.items
            .iter()
            .filter(|item| {
                item.name[1..].to_lowercase().contains(&lower)
                    || item.description.to_lowercase().contains(&lower)
            })
            .collect()
    }

    pub(crate) fn selected_command(&self) -> Option<String> {
        let filtered = self.filtered_items();
        if filtered.is_empty() {
            return None;
        }
        let idx = self.selected_index.min(filtered.len() - 1);
        Some(filtered[idx].name.clone())
    }

    pub(crate) fn close(&mut self) {
        self.is_open = false;
        self.items.clear();
        self.selected_index = 0;
        self.filter_text.clear();
    }

    pub(crate) fn select_next(&mut self) {
        let len = self.filtered_items().len();
        if len == 0 {
            return;
        }
        self.selected_index = (self.selected_index + 1) % len;
    }

    pub(crate) fn select_prev(&mut self) {
        let len = self.filtered_items().len();
        if len == 0 {
            return;
        }
        if self.selected_index == 0 {
            self.selected_index = len - 1;
        } else {
            self.selected_index -= 1;
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
    pub slash_menu: SlashMenuState,
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
