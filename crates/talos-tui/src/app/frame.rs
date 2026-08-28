use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FrameHistoryScrollBounds {
    current_start: usize,
    max_start: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FrameHistoryScrollOutcome {
    Noop,
    Anchored { start: usize },
    FollowTail,
}

impl FrameHistoryScrollBounds {
    fn outcome_for_target(self, target: usize) -> FrameHistoryScrollOutcome {
        if target == self.current_start {
            FrameHistoryScrollOutcome::Noop
        } else if target == self.max_start {
            FrameHistoryScrollOutcome::FollowTail
        } else {
            FrameHistoryScrollOutcome::Anchored { start: target }
        }
    }

    fn scroll_up(self, rows: usize) -> FrameHistoryScrollOutcome {
        self.outcome_for_target(self.current_start.saturating_sub(rows))
    }

    fn scroll_down(self, rows: usize) -> FrameHistoryScrollOutcome {
        let target = self.current_start.saturating_add(rows).min(self.max_start);
        self.outcome_for_target(target)
    }
}

impl Tui {
    pub(super) fn is_startup_mode(&self) -> bool {
        self.transcript.entries().is_empty()
            && !self.first_message_dispatched
            && !self.state.slash_menu.is_open
            && matches!(self.state.approval_state, ApprovalState::Hidden)
    }

    pub(super) fn draw_frame(&mut self) -> io::Result<()> {
        let state = &self.state;
        let status = &state.status;

        let (preview_padding, spinner_color) = if status.is_processing {
            let (padding, color_idx) =
                crate::scrollback::preview_spinner_padding(self.processing_frame);
            (padding, Some(semantic::PROCESSING_SPINNER[color_idx]))
        } else {
            self.processing_frame = 0;
            ("   ".to_string(), None)
        };
        let hold_status = self.stream_render.hold_status().cloned();
        let preview_text = preview_text_for_state(
            hold_status.as_ref(),
            status.phase.as_ref(),
            self.state.thinking_preview.as_deref(),
            status.is_processing,
            self.stream_render.preview(),
            self.processing_frame,
        );
        let preview_text_color = hold_status
            .as_ref()
            .map(|_| crate::scrollback::hold_preview_color(self.processing_frame));
        let thinking_label_frame = self
            .state
            .thinking_preview
            .as_ref()
            .filter(|_| status.is_processing && hold_status.is_none())
            .map(|_| self.processing_frame);
        let mut preview = crate::scrollback::PreviewComponent {
            padding: &preview_padding,
            text: &preview_text,
            spinner_color,
            text_color: preview_text_color,
            thinking_label_frame,
            max_height: crate::scrollback::MAX_PREVIEW_LINES,
        };
        let tips = crate::scrollback::TipsComponent {
            tip: state.tip.as_ref(),
        };
        let input_pad_top = crate::scrollback::InputPadComponent;
        let input_pad_bot = crate::scrollback::InputPadComponent;
        let query_for_panel = state.panel_query();
        let mut bottom_panel = crate::scrollback::BottomPanelComponent {
            menu: &state.slash_menu,
            query: query_for_panel,
            max_height: u16::MAX,
        };

        let screen_size = self.terminal.size()?;
        let width = screen_size.width;
        let is_startup = self.is_startup_mode();
        let approval_startup = self
            .approval_viewport_snapshot
            .as_ref()
            .is_some_and(|snapshot| snapshot.startup_mode);
        let approval_preserves_follow_tail =
            self.approval_viewport_snapshot
                .as_ref()
                .is_some_and(|snapshot| {
                    matches!(
                        snapshot.history_scroll.mode,
                        crate::history_projection::HistoryScrollMode::FollowTail
                    )
                });
        let natural_startup_flow = is_startup || approval_startup;
        let splash = crate::splash::viewport_splash_lines_with_dashboard(
            width,
            self.dashboard_availability.as_ref(),
        );
        let splash_rows = splash.len();
        let startup_spacer_rows: usize = if natural_startup_flow { 1 } else { 0 };
        let follows_tail = matches!(
            self.history_scroll.mode,
            crate::history_projection::HistoryScrollMode::FollowTail
        );
        // A short FollowTail conversation is a single vertical flow: Logo,
        // transcript, then the input frame.  `history_cap` makes AppLayout
        // allocate only that natural history height.  Once it cannot fit, the
        // normal allocation clamps it to the remaining frame and the composer
        // naturally becomes bottom-fixed.  Anchored history deliberately uses
        // the regular viewport so scrolling never moves the input frame.
        let history_cap = if natural_startup_flow {
            Some((splash_rows + startup_spacer_rows) as u16)
        } else if (follows_tail || approval_preserves_follow_tail)
            && (!self.state.slash_menu.is_open
                || !matches!(self.state.approval_state, ApprovalState::Hidden))
        {
            let natural_rows = self
                .history_projection_cache
                .project(
                    &self.transcript,
                    screen_size.width,
                    screen_size.height,
                    &self.history_scroll,
                )
                .total_rows;
            Some(
                splash_rows
                    .saturating_add(natural_rows)
                    .min(u16::MAX as usize) as u16,
            )
        } else {
            None
        };
        let status_comp = crate::scrollback::StatusComponent { status, width };

        let preview_natural = if is_startup {
            0
        } else {
            preview.height_hint(width)
        };
        // Tips remain visible during startup for failures and other transient
        // notices. Successful Dashboard availability belongs to the Logo prefix.
        let tips_h = tips.height_hint(width);

        let input_natural = crate::scrollback::InputComponent {
            state,
            max_height: crate::scrollback::MAX_COMPOSER_LINES,
        }
        .height_hint(width);
        let modal_natural = bottom_panel.height_hint(width);
        let fixed_heights = tips_h
            + input_pad_top.height_hint(width)
            + input_pad_bot.height_hint(width)
            + status_comp.height_hint(width);
        let queue_natural = crate::scrollback::QueuePreviewComponent {
            snapshot: state.steering_queue_snapshot.as_ref(),
            followup_count: status.followup_count,
            max_rows: 6,
        }
        .height_hint(width);

        let content_budget = screen_size.height.saturating_sub(fixed_heights);
        let compressed = crate::scrollback::compress_layout(
            content_budget,
            modal_natural,
            input_natural,
            preview_natural,
            queue_natural,
        );
        bottom_panel.max_height = compressed.panel_max_height;
        preview.max_height = compressed.preview_max_height;
        self.approval_preview_fully_visible = match &state.slash_menu.kind {
            Some(crate::state::PanelKind::Approval { preview, .. }) => {
                crate::scrollback::approval_preview_fully_visible(
                    width,
                    compressed.panel_max_height,
                    preview.as_deref(),
                )
            }
            _ => true,
        };

        let input = crate::scrollback::InputComponent {
            state,
            max_height: compressed.input_max_height,
        };
        let queue = crate::scrollback::QueuePreviewComponent {
            snapshot: state.steering_queue_snapshot.as_ref(),
            followup_count: status.followup_count,
            max_rows: compressed.queue_max_rows,
        };

        let actual_input_h = input.height_hint(width);
        let preview_h = preview.height_hint(width);
        // `bottom_panel_placement` adds its third argument itself, so this base
        // deliberately excludes the panel height. Preview is also excluded so
        // token-by-token growth cannot flip an open panel between placements.
        let base_height = fixed_heights + actual_input_h + queue.height_hint(width);
        let menu_placement = crate::scrollback::bottom_panel_placement(
            screen_size.height,
            base_height,
            modal_natural,
        );

        let app_layout = compute_app_layout(
            screen_size,
            ComponentMetrics {
                preview: preview_h,
                queue: queue.height_hint(width),
                tips: tips_h,
                panel_required: if state.slash_menu.is_credential_input()
                    || state.slash_menu.is_provider_wizard()
                    || !matches!(state.approval_state, ApprovalState::Hidden)
                {
                    bottom_panel.height_hint(width).min(4)
                } else {
                    0
                },
                panel_preferred: bottom_panel.height_hint(width),
                composer: actual_input_h,
                history_cap,
            },
            menu_placement,
        );
        let history_height = app_layout.history.map_or(0, |rect| rect.height);
        self.last_history_viewport_height = history_height;
        self.last_history_area = app_layout.history;
        let history = self.history_projection_cache.project(
            &self.transcript,
            screen_size.width,
            history_height,
            &self.history_scroll,
        );
        if !follows_tail
            && splash_rows.saturating_add(history.total_rows) <= usize::from(history_height)
        {
            self.history_prefix_start = None;
            self.history_scroll.jump_to_end();
            // FollowTail restores the natural history cap. Replan this frame once so resize/reflow
            // cannot leave the composer in the stale anchored layout until another render tick.
            return self.draw_frame();
        }
        self.last_history_projection = history.clone();
        if let (Some(area), Some(selection)) = (app_layout.history, self.selection.as_mut())
            && selection.dragging
            && selection.history_anchor.is_some()
            && selection.edge != 0
        {
            let row = if selection.edge < 0 {
                history.visible_start
            } else {
                history
                    .visible_start
                    .saturating_add(history.rows.len().saturating_sub(1))
            };
            if let Some(point) =
                history.selection_point(row, selection.focus.0.saturating_sub(area.x))
            {
                selection.history_focus = Some(point);
            }
        }
        if let Some(prefix_start) = self.history_prefix_start.as_mut() {
            *prefix_start = (*prefix_start).min(splash_rows.saturating_sub(1));
        }
        let follow_tail = follows_tail;
        let natural_start = if history.total_rows == 0 {
            0
        } else if follow_tail {
            splash_rows
                .saturating_add(history.total_rows)
                .saturating_sub(usize::from(history_height))
        } else {
            splash_rows.saturating_add(history.visible_start)
        };
        let frame_history_start = self.history_prefix_start.unwrap_or(natural_start);
        self.last_frame_history_start = frame_history_start;
        self.last_splash_row_count = splash_rows;
        self.last_history_prefix_row_count = splash_rows.saturating_add(startup_spacer_rows);
        let selection = self.selection.and_then(|selection| {
            match (
                selection.history_anchor,
                selection.history_focus,
                app_layout.history,
            ) {
                (Some(start), Some(end), Some(area)) => history.visible_selection(
                    start,
                    end,
                    frame_history_start,
                    self.last_history_prefix_row_count,
                    area,
                ),
                _ => Some(selection.points()),
            }
        });

        self.terminal.draw(screen_size, |frame| {
            if let Some(area) = app_layout.history {
                let history_text = frame_history_lines(
                    &history,
                    &splash,
                    frame_history_start,
                    startup_spacer_rows,
                );
                frame.render_widget(Paragraph::new(history_text), area);
            }
            if let Some(area) = app_layout.preview {
                preview.render(frame, area);
            }
            if let Some(area) = app_layout.queue {
                queue.render(frame, area);
            }
            if let Some(area) = app_layout.tips {
                tips.render(frame, area);
            }
            if let Some(area) = app_layout.panel {
                bottom_panel.render(frame, area);
            }
            if let Some(area) = app_layout.composer_top_pad {
                input_pad_top.render(frame, area);
            }
            if let Some(area) = app_layout.composer {
                input.render(frame, area);
            }
            if let Some(area) = app_layout.composer_bottom_pad {
                input_pad_bot.render(frame, area);
            }
            if let Some(area) = app_layout.status {
                status_comp.render(frame, area);
            }
            if let Some((start, end)) = selection {
                frame.highlight_selection(start, end);
            }
        })?;

        {
            let screen_w = screen_size.width;
            if self.state.slash_menu.is_credential_input() {
                let (Some(panel), Some(local)) = (
                    app_layout.panel,
                    crate::scrollback::credential_cursor_position(&self.state.slash_menu),
                ) else {
                    self.terminal.hide_cursor()?;
                    return Ok(());
                };
                self.terminal
                    .set_cursor_if_visible_in_rect(panel, local.col, local.row)?;
            } else if self.state.slash_menu.is_provider_wizard() {
                let (Some(panel), Some(local)) = (
                    app_layout.panel,
                    crate::scrollback::provider_wizard_local_cursor_position(
                        &self.state.slash_menu,
                    ),
                ) else {
                    self.terminal.hide_cursor()?;
                    return Ok(());
                };
                self.terminal
                    .set_cursor_if_visible_in_rect(panel, local.col, local.row)?;
            } else {
                let Some(composer_rect) = app_layout.composer else {
                    self.terminal.hide_cursor()?;
                    return Ok(());
                };
                let byte_pos = self.state.cursor_byte_pos();
                let content_w = crate::scrollback::composer_content_width(screen_w);
                let (cursor_row_offset, cursor_col_offset) =
                    crate::scrollback::cursor_line_col_with_width(
                        &self.state.input_buffer[..byte_pos],
                        content_w,
                    );
                let scroll_offset = crate::scrollback::composer_scroll_offset(
                    &self.state.input_buffer[..byte_pos],
                    &self.state.input_buffer,
                    content_w,
                    composer_rect.height,
                );
                let local_row = cursor_row_offset.saturating_sub(scroll_offset);
                let cursor_col = crate::scrollback::COMPOSER_LEFT_PAD + cursor_col_offset;
                self.terminal
                    .set_cursor_in_rect(composer_rect, cursor_col, local_row)?;
            }
        }

        Ok(())
    }

    pub(super) fn anchor_frame_history_start(&mut self, frame_start: usize) {
        if frame_start < self.last_splash_row_count {
            self.history_prefix_start = Some(frame_start);
            if let Some(anchor) = self.last_history_projection.first_anchor() {
                self.history_scroll.anchor(anchor, 0);
            } else {
                self.history_scroll.jump_to_end();
            }
            return;
        }

        self.history_prefix_start = None;
        let transcript_index = frame_start.saturating_sub(self.last_splash_row_count);
        if let Some(anchor) = self.last_history_projection.anchor_at(transcript_index) {
            self.history_scroll.anchor(anchor, 0);
        }
    }

    fn frame_history_scroll_bounds(&self, height: u16) -> FrameHistoryScrollBounds {
        let total_rows = self
            .last_splash_row_count
            .saturating_add(self.last_history_projection.total_rows);
        let max_start = total_rows.saturating_sub(usize::from(height));
        FrameHistoryScrollBounds {
            current_start: self.last_frame_history_start.min(max_start),
            max_start,
        }
    }

    pub(super) fn scroll_frame_history_up(&mut self, rows: usize) {
        let bounds = self.frame_history_scroll_bounds(self.last_history_viewport_height);
        self.apply_frame_history_scroll(bounds.scroll_up(rows));
    }

    pub(super) fn scroll_frame_history_down(&mut self, rows: usize, height: u16) {
        let bounds = self.frame_history_scroll_bounds(height);
        self.apply_frame_history_scroll(bounds.scroll_down(rows));
    }

    fn apply_frame_history_scroll(&mut self, outcome: FrameHistoryScrollOutcome) {
        match outcome {
            FrameHistoryScrollOutcome::Noop => {}
            FrameHistoryScrollOutcome::Anchored { start } => {
                self.anchor_frame_history_start(start);
            }
            FrameHistoryScrollOutcome::FollowTail => {
                self.history_prefix_start = None;
                self.history_scroll.jump_to_end();
            }
        }
    }
}
