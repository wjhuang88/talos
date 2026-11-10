use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RepeatFocus {
    panel: u64,
    open: bool,
    approval: bool,
    wizard: Option<crate::state::WizardStep>,
    credential: crate::state::CredentialField,
}
pub(super) async fn ready_input_batch(
    first: Event,
    stream: &mut (impl Stream<Item = io::Result<Event>> + Unpin),
) -> Vec<Event> {
    let mut batch = vec![first];
    futures::future::poll_fn(|cx| {
        // Use the live task waker even when draining stops at Pending.
        for _ in 1..1024 {
            match stream.poll_next_unpin(cx) {
                std::task::Poll::Ready(Some(Ok(next))) => batch.push(next),
                _ => break,
            }
        }
        std::task::Poll::Ready(coalesce_key_repeats(std::mem::take(&mut batch)))
    })
    .await
}

// Coalesce only protocol-labelled repeats. Press events (including terminals
// without release reporting), paste, resize and unrelated keys retain their order.
pub(super) fn coalesce_key_repeats(events: Vec<Event>) -> Vec<Event> {
    let mut pending = Vec::with_capacity(events.len());
    for event in events {
        if let Event::Key(key) = &event {
            if key.kind == KeyEventKind::Release {
                pending.retain(|queued| {
                    !matches!(queued, Event::Key(previous)
                    if previous.kind == KeyEventKind::Repeat
                        && previous.code == key.code && previous.modifiers == key.modifiers)
                });
            } else if key.kind == KeyEventKind::Repeat
                && matches!(pending.last(), Some(Event::Key(previous))
                    if previous.kind == KeyEventKind::Repeat
                        && previous.code == key.code && previous.modifiers == key.modifiers)
            {
                continue;
            }
        }
        pending.push(event);
    }
    pending
}

impl Tui {
    fn repeat_focus(&self) -> RepeatFocus {
        RepeatFocus {
            panel: self.state.slash_menu.input_id,
            open: self.state.slash_menu.is_open,
            approval: !matches!(self.state.approval_state, ApprovalState::Hidden),
            wizard: match &self.state.slash_menu.kind {
                Some(crate::state::PanelKind::ProviderWizard { step, .. }) => Some(*step),
                _ => None,
            },
            credential: self.state.slash_menu.credential_field,
        }
    }
    fn admit_key_event(&mut self, key: &event::KeyEvent) -> bool {
        let focus = self.repeat_focus();
        let safe = matches!(
            key.code,
            KeyCode::Backspace
                | KeyCode::Delete
                | KeyCode::Left
                | KeyCode::Right
                | KeyCode::Up
                | KeyCode::Down
                | KeyCode::PageUp
                | KeyCode::PageDown
                | KeyCode::Home
                | KeyCode::End
        );
        let identity = (key.code, key.modifiers, focus);
        match key.kind {
            KeyEventKind::Press => {
                self.repeat_key = safe.then_some(identity);
                true
            }
            KeyEventKind::Repeat => {
                let accepted = safe && self.repeat_key == Some(identity);
                if !accepted {
                    self.repeat_key = None;
                }
                accepted
            }
            KeyEventKind::Release => {
                if self.repeat_key.is_some_and(|(code, _, _)| code == key.code) {
                    self.repeat_key = None;
                }
                false
            }
        }
    }

    fn toggle_expansion_at_screen(&mut self, column: u16, row: u16) -> bool {
        let Some(area) = self.last_history_area else {
            return false;
        };
        if !area.contains(ratatui::layout::Position::new(column, row)) {
            return false;
        }
        let frame_row = self
            .last_frame_history_start
            .saturating_add(usize::from(row.saturating_sub(area.y)));
        let Some(history_row) = frame_row.checked_sub(self.last_history_prefix_row_count) else {
            return false;
        };
        let Some(id) = self
            .last_history_projection
            .expandable_header_at(history_row)
        else {
            return false;
        };
        self.transcript.toggle_expansion(id)
    }

    pub(super) fn history_selection_point_at_screen(
        &self,
        column: u16,
        row: u16,
    ) -> Option<HistorySelectionPoint> {
        let area = self.last_history_area?;
        if !area.contains(ratatui::layout::Position::new(column, row)) {
            return None;
        }
        let frame_row = self
            .last_frame_history_start
            .saturating_add(usize::from(row.saturating_sub(area.y)));
        let history_row = frame_row.checked_sub(self.last_history_prefix_row_count)?;
        self.last_history_projection
            .selection_point(history_row, column.saturating_sub(area.x))
    }

    pub(super) fn selected_text_for(&self, selection: SelectionState) -> String {
        match (selection.history_anchor, selection.history_focus) {
            (Some(start), Some(end)) => self.last_history_projection.selected_text(start, end),
            _ => {
                let (start, end) = selection.points();
                self.terminal.selected_text(start, end)
            }
        }
    }

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
            KeyCode::Char('d') => {
                let view = &mut self.state.slash_menu.approval_view;
                view.expanded = !view.expanded;
                view.offset = 0;
            }
            KeyCode::PageDown => {
                let view = &mut self.state.slash_menu.approval_view;
                if view.expanded
                    && view.page_size > 0
                    && view.offset.saturating_add(view.page_size) < view.total_rows
                {
                    // Only advance past rows actually rendered, never skip a
                    // hidden page and accidentally enable reusable approval.
                    view.offset = view
                        .offset
                        .saturating_add(view.page_size)
                        .min(view.reviewed)
                        .min(view.total_rows.saturating_sub(1));
                }
            }
            KeyCode::PageUp => {
                let view = &mut self.state.slash_menu.approval_view;
                view.offset = view.offset.saturating_sub(view.page_size);
            }
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
                if let Some(choice) = match c {
                    '1' => Some(ApprovalChoice::ApproveOnce),
                    '2' => Some(ApprovalChoice::AlwaysApprove),
                    '3' => Some(ApprovalChoice::Deny),
                    _ => None,
                } {
                    self.resolve_approval(choice);
                }
            }
            _ => {}
        }
    }

    pub(super) fn resolve_approval(&mut self, choice: ApprovalChoice) {
        if self.retire_closed_approval() {
            return;
        }
        let choice = if matches!(choice, ApprovalChoice::AlwaysApprove)
            && self
                .state
                .slash_menu
                .approval_view
                .presentation
                .as_ref()
                .is_some_and(|p| p.once_only)
        {
            ApprovalChoice::ApproveOnce
        } else {
            choice
        };
        if matches!(choice, ApprovalChoice::AlwaysApprove) && !self.approval_preview_fully_visible {
            self.state.tip = Some(Tip {
                kind: TipKind::Info,
                text: "Press d and PageDown to review the full reusable scope".to_string(),
                ttl: Duration::from_secs(3),
                created_at: Instant::now(),
            });
            return;
        }
        if let Some(response_tx) = self.state.pending_approval_response.take()
            && response_tx.send(choice.clone()).is_err()
        {
            self.hide_approval();
            self.state.slash_menu.close();
            self.append_styled_lines([ScrollbackLine::styled(
                vec![HistorySegment::raw(
                    "Approval expired or was cancelled; no decision was submitted.",
                )],
                None,
            )]);
            return;
        }
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
        let tool_name = match &self.state.approval_state {
            ApprovalState::Visible { tool_name, .. } => tool_name.as_str(),
            ApprovalState::Hidden => "unknown tool",
        };
        self.pending_transcript
            .push(TranscriptBlock::StyledLine(ScrollbackLine::styled(
                vec![HistorySegment::styled(
                    format!("   {icon} {msg}: {tool_name}"),
                    color,
                    HistoryAttrs::default(),
                )],
                None,
            )));
        let _ = self.commit_pending_transcript();

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
        if !matches!(event, Event::Key(_)) {
            self.repeat_key = None;
        }
        match event {
            Event::Key(key) => {
                if !self.admit_key_event(key) {
                    return false;
                }
                // Modal scope navigation owns paging before global history shortcuts.
                if !matches!(self.state.approval_state, ApprovalState::Hidden)
                    && matches!(key.code, KeyCode::PageUp | KeyCode::PageDown)
                {
                    self.handle_pending_approval_input(key.code);
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
                    KeyCode::Down if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
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
                            self.state.wizard_cycle_protocol(key.code == KeyCode::Up);
                        }
                        KeyCode::Char(c) => {
                            self.last_char_time = Some(Instant::now());
                            self.state.wizard_append_char(c);
                        }
                        _ => {}
                    }
                    return false;
                }
                if matches!(key.code, KeyCode::Char('v' | 'V'))
                    && key
                        .modifiers
                        .intersects(event::KeyModifiers::CONTROL | event::KeyModifiers::ALT)
                    && !self.state.slash_menu.is_open
                {
                    if self
                        .last_image_paste
                        .is_none_or(|last| last.elapsed() >= Duration::from_millis(250))
                    {
                        self.last_image_paste = Some(Instant::now());
                        self.clipboard_paste_pending = true;
                        if let Some(tx) = &self.user_input_tx {
                            let _ = tx.send(UserInput::Message("/paste-image".into()));
                        }
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
                    KeyCode::Delete => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        self.state.input_delete();
                        if self.state.slash_menu.is_open {
                            self.state.refresh_slash_menu_after_input();
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
                    KeyCode::Enter if self.clipboard_paste_pending => {}
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
                        if self.state.status.is_processing
                            || self.state.compaction_pending
                            || self.clipboard_paste_pending
                        {
                            if let Some(ref tx) = self.user_input_tx {
                                let _ = tx.send(UserInput::Cancel);
                            }
                            self.state.tip = Some(Tip {
                                kind: TipKind::ExitHint,
                                text: "Cancellation requested.".to_string(),
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
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::Down(event::MouseButton::Left) => {
                    let history_point =
                        self.history_selection_point_at_screen(mouse.column, mouse.row);
                    self.selection = Some(SelectionState {
                        anchor: (mouse.column, mouse.row),
                        focus: (mouse.column, mouse.row),
                        dragging: true,
                        moved: false,
                        edge: 0,
                        history_anchor: history_point,
                        history_focus: history_point,
                    });
                }
                MouseEventKind::Drag(event::MouseButton::Left) => {
                    let edge = self.last_history_area.map_or(0, |area| {
                        if mouse.row <= area.y.saturating_add(1) {
                            -1
                        } else if mouse.row.saturating_add(2) >= area.bottom() {
                            1
                        } else {
                            0
                        }
                    });
                    let history_focus =
                        self.history_selection_point_at_screen(mouse.column, mouse.row);
                    let inside_history = self.last_history_area.is_some_and(|area| {
                        area.contains(ratatui::layout::Position::new(mouse.column, mouse.row))
                    });
                    if let Some(selection) = self.selection.as_mut() {
                        selection.moved = true;
                        selection.focus = (mouse.column, mouse.row);
                        selection.edge = edge;
                        selection.update_history_focus(history_focus, inside_history || edge != 0);
                    }
                }
                MouseEventKind::Up(event::MouseButton::Left) => {
                    let history_focus =
                        self.history_selection_point_at_screen(mouse.column, mouse.row);
                    let inside_history = self.last_history_area.is_some_and(|area| {
                        area.contains(ratatui::layout::Position::new(mouse.column, mouse.row))
                    });
                    if let Some(mut selection) = self.selection {
                        selection.focus = (mouse.column, mouse.row);
                        selection.update_history_focus(
                            history_focus,
                            inside_history || selection.edge != 0,
                        );
                        if selection.anchor == selection.focus {
                            self.selection = None;
                            if !selection.moved {
                                self.toggle_expansion_at_screen(mouse.column, mouse.row);
                            }
                            return false;
                        }
                        selection.dragging = false;
                        selection.edge = 0;
                        self.selection = Some(selection);
                        let text = self.selected_text_for(selection);
                        if !text.is_empty() {
                            match crate::clipboard::copy_text(&text) {
                                Ok(backend) => {
                                    self.state.tip = Some(Tip {
                                        kind: TipKind::Info,
                                        text: format!(
                                            "Copied selection to clipboard (via {backend:?})"
                                        ),
                                        ttl: Duration::from_secs(3),
                                        created_at: Instant::now(),
                                    });
                                }
                                Err(error) => {
                                    self.state.tip = Some(Tip {
                                        kind: TipKind::Error,
                                        text: format!("Failed to copy selection: {error:?}"),
                                        ttl: Duration::from_secs(4),
                                        created_at: Instant::now(),
                                    });
                                }
                            }
                        }
                    }
                }
                MouseEventKind::ScrollUp => {
                    if self.selection.is_some_and(|selection| selection.dragging) {
                        return false;
                    }
                    if self.last_history_viewport_height == 0 {
                        return false;
                    }
                    self.scroll_frame_history_up(MOUSE_HISTORY_SCROLL_ROWS);
                }
                MouseEventKind::ScrollDown => {
                    if self.selection.is_some_and(|selection| selection.dragging) {
                        return false;
                    }
                    let height = self.last_history_viewport_height;
                    if height == 0 {
                        return false;
                    }
                    self.scroll_frame_history_down(MOUSE_HISTORY_SCROLL_ROWS, height);
                }
                _ => {}
            },
            Event::Resize(width, height) => {
                if let Some(selection) = self.selection.as_mut() {
                    let max_x = width.saturating_sub(1);
                    let max_y = height.saturating_sub(1);
                    selection.anchor.0 = selection.anchor.0.min(max_x);
                    selection.anchor.1 = selection.anchor.1.min(max_y);
                    selection.focus.0 = selection.focus.0.min(max_x);
                    selection.focus.1 = selection.focus.1.min(max_y);
                }
            }
            _ => {}
        }
        false
    }
}
