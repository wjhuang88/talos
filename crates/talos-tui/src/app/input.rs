use super::*;

impl Tui {
    pub(super) fn dispatch_panel_action(&mut self, action: PanelAction) {
        match action {
            PanelAction::SendMessage(msg) => {
                if let Some(ref tx) = self.user_input_tx {
                    let _ = tx.send(UserInput::Message(msg));
                }
            }
            PanelAction::ProviderSetup(provider) => {
                self.state
                    .open_credential_input(&provider, None, false, None);
                self.state.input_clear();
            }
            PanelAction::SwitchModel {
                provider,
                model_id,
                variant,
            } => {
                if let Some(ref tx) = self.user_input_tx {
                    let _ = tx.send(UserInput::SwitchModel {
                        provider,
                        model_id,
                        variant,
                    });
                }
            }
            PanelAction::ConnectSelect { provider } => {
                if let Some(ref tx) = self.user_input_tx {
                    let _ = tx.send(UserInput::ConnectSelect { provider });
                }
            }
            PanelAction::RegisterCustomProvider {
                name,
                protocol,
                base_url,
                api_key,
            } => {
                if let Some(ref tx) = self.user_input_tx {
                    let _ = tx.send(UserInput::RegisterCustomProvider {
                        name,
                        protocol,
                        base_url,
                        api_key,
                    });
                }
            }
            PanelAction::None => {}
        }
    }

    pub(super) fn handle_pending_approval_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Up => {
                self.state.slash_menu.select_prev("");
            }
            KeyCode::Down => {
                self.state.slash_menu.select_next("");
            }
            KeyCode::Enter => {
                let idx = self.state.slash_menu.selected_index;
                let choice = match idx {
                    0 => ApprovalChoice::ApproveOnce,
                    1 => ApprovalChoice::AlwaysApprove,
                    _ => ApprovalChoice::Deny,
                };
                self.resolve_approval(choice);
            }
            KeyCode::Esc => {
                self.resolve_approval(ApprovalChoice::Deny);
            }
            KeyCode::Char(c) => {
                if let Some(choice) = self.handle_approval_key(c) {
                    self.resolve_approval(choice);
                }
            }
            _ => {}
        }
    }

    pub(super) fn resolve_approval(&mut self, choice: ApprovalChoice) {
        let (icon, color, msg) = match &choice {
            ApprovalChoice::ApproveOnce => (
                "\u{2713}",
                to_crossterm_color(semantic::TEXT_SUCCESS),
                "approved",
            ),
            ApprovalChoice::AlwaysApprove => (
                "\u{2713}",
                to_crossterm_color(semantic::TEXT_SUCCESS),
                "always approved",
            ),
            ApprovalChoice::Deny => (
                "\u{2717}",
                to_crossterm_color(semantic::TEXT_ERROR),
                "denied",
            ),
        };
        self.pending_transcript
            .push(TranscriptBlock::StyledLine(ScrollbackLine::styled(
                vec![HistorySegment::styled(
                    format!("   {icon} {msg}"),
                    color,
                    HistoryAttrs::default(),
                )],
                None,
            )));
        let _ = self.commit_pending_transcript();

        if let Some(response_tx) = self.state.pending_approval_response.take() {
            let _ = response_tx.send(choice);
        }
        self.hide_approval();
        self.state.slash_menu.close();
        self.state.tip = Some(Tip {
            kind: TipKind::ApprovalResult,
            text: format!("Tool call {msg}"),
            ttl: Duration::from_secs(2),
            created_at: Instant::now(),
        });
    }

    pub(super) fn handle_input_event(&mut self, event: &Event) -> bool {
        match event {
            Event::Key(key) => {
                if key.kind != KeyEventKind::Press {
                    return false;
                }
                match key.code {
                    KeyCode::PageUp => {
                        self.history_prefix_start = None;
                        let height = self.last_history_viewport_height;
                        if height == 0 {
                            return false;
                        }
                        if let Some(anchor) = self.last_history_projection.page_up(height) {
                            self.history_scroll.anchor(anchor, 0);
                        }
                        return false;
                    }
                    KeyCode::PageDown => {
                        self.history_prefix_start = None;
                        let height = self.last_history_viewport_height;
                        if height == 0 {
                            return false;
                        }
                        if self.last_history_projection.page_down_reaches_tail(height) {
                            self.history_scroll.jump_to_end();
                        } else if let Some(anchor) = self.last_history_projection.page_down(height)
                        {
                            self.history_scroll.anchor(anchor, 0);
                        }
                        return false;
                    }
                    KeyCode::Home if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        self.history_prefix_start = None;
                        if let Some(anchor) = self.last_history_projection.first_anchor() {
                            self.history_scroll.anchor(anchor, 0);
                        }
                        return false;
                    }
                    KeyCode::End if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        self.history_prefix_start = None;
                        self.history_scroll.jump_to_end();
                        return false;
                    }
                    _ => {}
                }
                if key.code == KeyCode::Char('c')
                    && key.modifiers.contains(event::KeyModifiers::CONTROL)
                {
                    if !matches!(self.state.approval_state, ApprovalState::Hidden) {
                        self.resolve_approval(ApprovalChoice::Deny);
                        return false;
                    }
                    if self.state.slash_menu.is_credential_input() {
                        self.state.credential_cancel();
                        self.state.input_clear();
                        return false;
                    }
                    if self.state.slash_menu.is_provider_wizard() {
                        self.state.wizard_cancel();
                        self.state.input_clear();
                        return false;
                    }
                    if self.state.slash_menu.is_open {
                        self.state.slash_menu.close();
                        self.state.input_clear();
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        return false;
                    }
                    if !self.state.input_buffer.is_empty() {
                        self.state.input_clear();
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        self.state.tip = Some(Tip {
                            kind: TipKind::ExitHint,
                            text: "Input cleared. Press Ctrl+C twice to exit.".to_string(),
                            ttl: Duration::from_secs(2),
                            created_at: Instant::now(),
                        });
                        return false;
                    }
                    if self.state.status.is_processing {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        self.state.tip = Some(Tip {
                            kind: TipKind::ExitHint,
                            text: "Press Esc to interrupt the current turn.".to_string(),
                            ttl: Duration::from_secs(2),
                            created_at: Instant::now(),
                        });
                        return false;
                    }
                    return self.state.handle_ctrl_c();
                }
                if !matches!(self.state.approval_state, ApprovalState::Hidden) {
                    self.handle_pending_approval_input(key.code);
                    return false;
                }
                if self.state.slash_menu.is_credential_input() {
                    match key.code {
                        KeyCode::Enter => {
                            if let Some(t) = self.last_char_time
                                && t.elapsed() < IME_ENTER_WINDOW
                            {
                                return false;
                            }
                            if let Some(resp) = self.state.credential_submit()
                                && let Some(ref tx) = self.user_input_tx
                            {
                                let _ = tx.send(UserInput::Credential(resp));
                            }
                            self.state.input_clear();
                        }
                        KeyCode::Esc => {
                            self.state.credential_cancel();
                            self.state.input_clear();
                        }
                        KeyCode::Backspace => {
                            self.state.credential_backspace();
                        }
                        KeyCode::Char(c) => {
                            self.last_char_time = Some(Instant::now());
                            self.state.credential_append_char(c);
                        }
                        _ => {}
                    }
                    return false;
                }
                if self.state.slash_menu.is_provider_wizard() {
                    match key.code {
                        KeyCode::Enter => {
                            if let Some(t) = self.last_char_time
                                && t.elapsed() < IME_ENTER_WINDOW
                            {
                                return false;
                            }
                            if let Some(action) = self.state.wizard_advance() {
                                self.dispatch_panel_action(action);
                            }
                            self.state.input_clear();
                        }
                        KeyCode::Esc => {
                            self.state.wizard_cancel();
                            self.state.input_clear();
                        }
                        KeyCode::Backspace => {
                            self.state.wizard_backspace();
                        }
                        KeyCode::Up | KeyCode::Down => {
                            self.state.wizard_cycle_protocol();
                        }
                        KeyCode::Char(c) => {
                            self.last_char_time = Some(Instant::now());
                            self.state.wizard_append_char(c);
                        }
                        _ => {}
                    }
                    return false;
                }
                match key.code {
                    KeyCode::Char('a') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        self.state.slash_menu.close();
                        self.state.input_cursor_to_line_start();
                    }
                    KeyCode::Char('e') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        self.state.slash_menu.close();
                        self.state.input_cursor_to_line_end();
                    }
                    KeyCode::Char('g') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        self.state.slash_menu.close();
                        self.toggle_evolution_panel();
                    }
                    KeyCode::Up if self.state.slash_menu.is_open => {
                        let query = self.state.panel_query().to_string();
                        self.state.slash_menu.select_prev(&query);
                    }
                    KeyCode::Down if self.state.slash_menu.is_open => {
                        let query = self.state.panel_query().to_string();
                        self.state.slash_menu.select_next(&query);
                    }
                    KeyCode::Up if !self.state.slash_menu.is_open => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        self.state.history_prev();
                    }
                    KeyCode::Down if !self.state.slash_menu.is_open => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        self.state.history_next();
                    }
                    KeyCode::Tab if self.state.slash_menu.is_open => {
                        let action = self.state.complete_selected_panel_item();
                        self.dispatch_panel_action(action);
                    }
                    KeyCode::Enter if self.state.slash_menu.is_open => {
                        if let Some(t) = self.last_char_time
                            && t.elapsed() < IME_ENTER_WINDOW
                        {
                            return false;
                        }
                        let action = self.state.accept_selected_panel_item();
                        self.dispatch_panel_action(action);
                    }
                    KeyCode::Esc if self.state.slash_menu.is_open => {
                        self.state.slash_menu.close();
                    }
                    KeyCode::Char('j')
                        if key.modifiers.contains(event::KeyModifiers::CONTROL)
                            && !self.state.slash_menu.is_open =>
                    {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        self.state.input_append_char('\n');
                    }
                    KeyCode::Char('/') if self.state.input_buffer.is_empty() => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        let registry = talos_conversation::command_registry();
                        self.state.open_slash_menu(registry);
                    }
                    KeyCode::Char(c) => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        self.last_char_time = Some(Instant::now());
                        if self.state.slash_menu.is_open {
                            self.state.append_slash_query_char(c);
                        } else {
                            self.state.input_append_char(c);
                        }
                    }
                    KeyCode::Backspace => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        if self.state.slash_menu.is_open {
                            self.state.backspace_slash_query();
                        } else {
                            self.state.input_backspace();
                        }
                    }
                    KeyCode::Left => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        self.state.input_cursor_left();
                    }
                    KeyCode::Right => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        self.state.input_cursor_right();
                    }
                    KeyCode::Enter => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        if key.modifiers.contains(event::KeyModifiers::SHIFT) {
                            self.state.input_append_char('\n');
                        } else {
                            let sent = submit_input_message(
                                &mut self.state,
                                &mut self.stream_render,
                                self.user_input_tx.as_ref(),
                            );
                            if sent {
                                self.first_message_dispatched = true;
                            }
                        }
                    }
                    KeyCode::Esc => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        if self.state.status.is_processing {
                            if let Some(ref tx) = self.user_input_tx {
                                let _ = tx.send(UserInput::Cancel);
                            }
                            self.state.tip = Some(Tip {
                                kind: TipKind::ExitHint,
                                text: "Turn cancellation requested.".to_string(),
                                ttl: Duration::from_secs(2),
                                created_at: Instant::now(),
                            });
                        }
                    }
                    _ => {}
                }
            }
            Event::Paste(text) => {
                self.state.input_paste(text);
            }
            Event::Mouse(mouse) => {
                let height = self.last_history_viewport_height;
                if height == 0 {
                    return false;
                }
                match mouse.kind {
                    MouseEventKind::ScrollUp => {
                        self.scroll_frame_history_up(MOUSE_HISTORY_SCROLL_ROWS);
                    }
                    MouseEventKind::ScrollDown => {
                        self.scroll_frame_history_down(MOUSE_HISTORY_SCROLL_ROWS, height);
                    }
                    _ => {}
                }
            }
            Event::Resize(_, _) => {}
            _ => {}
        }
        false
    }
}
