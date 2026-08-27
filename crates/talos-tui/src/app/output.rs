use super::*;
use talos_conversation::MessageSource;

const TOOL_PLACEHOLDER_MARKERS: [&str; 2] = ["Calling tools…", "Calling tools..."];

/// Holds only a possible standalone compatibility marker until the ordered response
/// proves whether a structured tool call follows.
#[derive(Default)]
pub(super) struct ToolPlaceholderGate {
    enabled: bool,
    at_line_start: bool,
    line_candidate: String,
    held_marker: String,
}

impl ToolPlaceholderGate {
    pub(super) fn start(&mut self, source: &MessageSource) {
        self.enabled = matches!(source, MessageSource::Assistant);
        self.at_line_start = true;
        self.line_candidate.clear();
        self.held_marker.clear();
    }

    pub(super) fn push(&mut self, text: &str) -> String {
        if !self.enabled {
            return text.to_string();
        }

        let mut output = String::new();
        for ch in text.chars() {
            if !self.held_marker.is_empty() {
                if ch.is_whitespace() {
                    self.held_marker.push(ch);
                    self.at_line_start = ch == '\n' || self.at_line_start;
                    continue;
                }
                output.push_str(&std::mem::take(&mut self.held_marker));
            }

            if self.at_line_start {
                self.line_candidate.push(ch);
                if is_placeholder_prefix(&self.line_candidate) {
                    if ch == '\n' && is_exact_placeholder(&self.line_candidate) {
                        self.held_marker
                            .push_str(&std::mem::take(&mut self.line_candidate));
                        self.at_line_start = true;
                    }
                    continue;
                }

                let candidate = std::mem::take(&mut self.line_candidate);
                self.at_line_start = candidate.ends_with('\n');
                output.push_str(&candidate);
            } else {
                output.push(ch);
                if ch == '\n' {
                    self.at_line_start = true;
                }
            }
        }
        output
    }

    pub(super) fn finish_content(&mut self) -> String {
        let candidate = std::mem::take(&mut self.line_candidate);
        if is_exact_placeholder(&candidate) {
            self.held_marker.push_str(&candidate);
            String::new()
        } else {
            candidate
        }
    }

    pub(super) fn confirm_tool_call(&mut self) -> String {
        let candidate = std::mem::take(&mut self.line_candidate);
        let output = if is_exact_placeholder(&candidate) {
            String::new()
        } else {
            candidate
        };
        self.held_marker.clear();
        self.enabled = false;
        output
    }

    pub(super) fn is_pending(&self) -> bool {
        !self.held_marker.is_empty()
    }

    pub(super) fn flush(&mut self) -> String {
        self.enabled = false;
        let mut output = std::mem::take(&mut self.held_marker);
        output.push_str(&std::mem::take(&mut self.line_candidate));
        output
    }
}

fn is_placeholder_prefix(text: &str) -> bool {
    let trimmed = text.trim_start();
    if trimmed.is_empty() {
        return true;
    }

    TOOL_PLACEHOLDER_MARKERS.iter().any(|marker| {
        marker.starts_with(trimmed)
            || (trimmed.starts_with(marker) && trimmed[marker.len()..].trim().is_empty())
    })
}

fn is_exact_placeholder(text: &str) -> bool {
    TOOL_PLACEHOLDER_MARKERS.contains(&text.trim())
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    use super::*;
    use talos_conversation::{ContentOutput, ToolCallDisplay, ToolResultDisplay, UiOutput};
    use talos_core::tool::ToolProvenance;

    fn assistant_gate() -> ToolPlaceholderGate {
        let mut gate = ToolPlaceholderGate::default();
        gate.start(&MessageSource::Assistant);
        gate
    }

    #[test]
    fn holds_split_unicode_marker_until_tool_call_decision() {
        let mut gate = assistant_gate();
        assert_eq!(gate.push("Calling tools"), "");
        assert_eq!(gate.push("…"), "");
        assert_eq!(gate.finish_content(), "");
        assert_eq!(gate.confirm_tool_call(), "");
        assert_eq!(gate.flush(), "");
    }

    #[test]
    fn flushes_marker_when_followed_by_normal_text() {
        let mut gate = assistant_gate();
        assert_eq!(gate.push("Calling tools..."), "");
        assert_eq!(
            gate.push(" for this task"),
            "Calling tools... for this task"
        );
    }

    #[test]
    fn flushes_marker_at_end_without_tool_call() {
        let mut gate = assistant_gate();
        assert_eq!(gate.push("Calling tools…\n"), "");
        assert_eq!(gate.flush(), "Calling tools…\n");
    }

    #[test]
    fn does_not_filter_marker_inside_larger_text() {
        let mut gate = assistant_gate();
        assert_eq!(gate.push("I said Calling tools…"), "I said Calling tools…");
    }

    #[test]
    fn non_assistant_content_is_not_gated() {
        let mut gate = ToolPlaceholderGate::default();
        gate.start(&MessageSource::User);
        assert_eq!(gate.push("Calling tools…"), "Calling tools…");
    }

    #[test]
    fn preserves_text_before_a_standalone_marker() {
        let mut gate = assistant_gate();
        assert_eq!(gate.push("First line\nCalling "), "First line\n");
        assert_eq!(gate.push("tools...\n"), "");
        assert_eq!(gate.finish_content(), "");
        assert_eq!(gate.confirm_tool_call(), "");
    }

    #[test]
    fn preserves_incomplete_marker_prefix_before_tool_call() {
        let mut gate = assistant_gate();
        assert_eq!(gate.push("Calling too"), "");
        assert_eq!(gate.finish_content(), "Calling too");
        assert_eq!(gate.confirm_tool_call(), "");
    }

    fn tool_call_named(name: &str) -> ToolCallDisplay {
        ToolCallDisplay {
            tool_name: name.into(),
            arguments: serde_json::json!({"path": "README.md"}),
            provenance: ToolProvenance::Native,
            summary_fields: vec!["path".into()],
        }
    }

    fn tool_call() -> ToolCallDisplay {
        tool_call_named("read_file")
    }

    fn pending_text(tui: &Tui) -> String {
        tui.pending_transcript
            .iter()
            .filter_map(|block| match block {
                TranscriptBlock::StyledLine(line) => Some(line.text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn structured_tool_call_suppresses_split_marker_without_blank_row() {
        let mut tui = Tui::for_test(TuiState::new(), None);
        tui.handle_ui_output(UiOutput::Content(ContentOutput::Start {
            source: MessageSource::Assistant,
        }));
        tui.handle_ui_output(UiOutput::Content(ContentOutput::Delta {
            text: "Calling ".into(),
        }));
        tui.handle_ui_output(UiOutput::Content(ContentOutput::Delta {
            text: "tools…\n".into(),
        }));
        tui.handle_ui_output(UiOutput::Content(ContentOutput::End));
        tui.handle_ui_output(UiOutput::ToolCall(tool_call()));

        assert_eq!(pending_text(&tui), "");
        assert_eq!(tui.pending_transcript.len(), 1);
        assert!(matches!(
            tui.pending_transcript[0],
            TranscriptBlock::ToolCall(_)
        ));
        assert!(!tui.ordered_content_open);
    }

    #[test]
    fn tool_call_started_without_confirmed_call_does_not_suppress_marker() {
        let mut tui = Tui::for_test(TuiState::new(), None);
        tui.handle_ui_output(UiOutput::Content(ContentOutput::Start {
            source: MessageSource::Assistant,
        }));
        tui.handle_ui_output(UiOutput::Content(ContentOutput::Delta {
            text: "Calling tools…".into(),
        }));
        tui.handle_ui_output(UiOutput::Content(ContentOutput::End));
        tui.handle_ui_output(UiOutput::ToolCallStarted {
            name: "read_file".into(),
        });
        tui.handle_ui_output(UiOutput::Status(Default::default()));

        assert!(pending_text(&tui).contains("Calling tools…"));
        assert!(!tui.ordered_content_open);
    }

    #[test]
    fn terminal_completion_flushes_standalone_marker() {
        let mut tui = Tui::for_test(TuiState::new(), None);
        tui.handle_ui_output(UiOutput::Content(ContentOutput::Start {
            source: MessageSource::Assistant,
        }));
        tui.handle_ui_output(UiOutput::Content(ContentOutput::Delta {
            text: "Calling tools...".into(),
        }));
        tui.handle_ui_output(UiOutput::Content(ContentOutput::End));
        tui.handle_ui_output(UiOutput::Status(Default::default()));

        assert!(pending_text(&tui).contains("Calling tools..."));
    }

    #[test]
    fn larger_sentence_is_visible_before_later_tool_call() {
        let mut tui = Tui::for_test(TuiState::new(), None);
        tui.handle_ui_output(UiOutput::Content(ContentOutput::Start {
            source: MessageSource::Assistant,
        }));
        tui.handle_ui_output(UiOutput::Content(ContentOutput::Delta {
            text: "Calling tools... can take a moment.\n".into(),
        }));
        tui.handle_ui_output(UiOutput::Content(ContentOutput::End));
        tui.handle_ui_output(UiOutput::ToolCall(tool_call()));

        assert!(pending_text(&tui).contains("Calling tools... can take a moment."));
        assert!(matches!(
            tui.pending_transcript.last(),
            Some(TranscriptBlock::ToolCall(_))
        ));
    }

    #[test]
    fn leading_whitespace_three_dot_marker_is_suppressed_for_multiple_tools() {
        let mut tui = Tui::for_test(TuiState::new(), None);
        tui.handle_ui_output(UiOutput::Content(ContentOutput::Start {
            source: MessageSource::Assistant,
        }));
        tui.handle_ui_output(UiOutput::Content(ContentOutput::Delta {
            text: "  Calling tools...  \n".into(),
        }));
        tui.handle_ui_output(UiOutput::Content(ContentOutput::End));
        tui.handle_ui_output(UiOutput::ToolCall(tool_call_named("read_file")));
        tui.handle_ui_output(UiOutput::ToolCall(tool_call_named("list_files")));

        assert_eq!(pending_text(&tui), "");
        assert_eq!(
            tui.pending_transcript
                .iter()
                .filter(|block| matches!(block, TranscriptBlock::ToolCall(_)))
                .count(),
            2
        );
    }

    #[test]
    fn tool_result_without_confirmed_call_flushes_marker() {
        let mut tui = Tui::for_test(TuiState::new(), None);
        tui.handle_ui_output(UiOutput::Content(ContentOutput::Start {
            source: MessageSource::Assistant,
        }));
        tui.handle_ui_output(UiOutput::Content(ContentOutput::Delta {
            text: "Calling tools…".into(),
        }));
        tui.handle_ui_output(UiOutput::Content(ContentOutput::End));
        tui.handle_ui_output(UiOutput::ToolResult(ToolResultDisplay {
            tool_name: Some("read_file".into()),
            is_error: false,
            content: "done".into(),
        }));

        assert!(pending_text(&tui).contains("Calling tools…"));
        assert!(matches!(
            tui.pending_transcript.last(),
            Some(TranscriptBlock::ToolResult(_))
        ));
    }

    #[test]
    fn approval_wait_keeps_marker_pending_until_call_is_confirmed() {
        let mut tui = Tui::for_test(TuiState::new(), None);
        tui.handle_ui_output(UiOutput::Content(ContentOutput::Start {
            source: MessageSource::Assistant,
        }));
        tui.handle_ui_output(UiOutput::Content(ContentOutput::Delta {
            text: "Calling tools...".into(),
        }));
        tui.handle_ui_output(UiOutput::Content(ContentOutput::End));
        let (response, _receiver) = tokio::sync::oneshot::channel();
        tui.handle_ui_output(UiOutput::ToolApprovalRequest {
            tool_name: "write_file".into(),
            arguments: serde_json::json!({"path": "README.md"}),
            summary_fields: vec!["path".into()],
            preview: None,
            response,
        });

        assert!(!pending_text(&tui).contains("Calling tools..."));
        assert!(tui.tool_placeholder_gate.is_pending());
        assert!(tui.state.pending_approval_response.is_some());

        tui.handle_ui_output(UiOutput::ToolCall(tool_call_named("write_file")));
        assert!(!pending_text(&tui).contains("Calling tools..."));
        assert!(matches!(
            tui.pending_transcript.last(),
            Some(TranscriptBlock::ToolCall(_))
        ));
    }

    #[test]
    fn approval_without_confirmed_call_flushes_marker_on_direct_result() {
        let mut tui = Tui::for_test(TuiState::new(), None);
        tui.handle_ui_output(UiOutput::Content(ContentOutput::Start {
            source: MessageSource::Assistant,
        }));
        tui.handle_ui_output(UiOutput::Content(ContentOutput::Delta {
            text: "Calling tools...".into(),
        }));
        tui.handle_ui_output(UiOutput::Content(ContentOutput::End));
        let (response, _receiver) = tokio::sync::oneshot::channel();
        tui.handle_ui_output(UiOutput::ToolApprovalRequest {
            tool_name: "write_file".into(),
            arguments: serde_json::json!({"path": "README.md"}),
            summary_fields: vec!["path".into()],
            preview: None,
            response,
        });
        tui.handle_ui_output(UiOutput::ToolResult(ToolResultDisplay {
            tool_name: None,
            is_error: true,
            content: "permission denied".into(),
        }));

        assert!(pending_text(&tui).contains("Calling tools..."));
        assert!(matches!(
            tui.pending_transcript.last(),
            Some(TranscriptBlock::ToolResult(_))
        ));
    }

    #[test]
    fn approval_outcome_keeps_the_correlated_tool_name() {
        let mut tui = Tui::for_test(TuiState::new(), None);
        let (response, _receiver) = tokio::sync::oneshot::channel();
        tui.handle_ui_output(UiOutput::ToolApprovalRequest {
            tool_name: "write_file".into(),
            arguments: serde_json::json!({"path": "README.md"}),
            summary_fields: vec!["path".into()],
            preview: None,
            response,
        });

        tui.resolve_approval(talos_core::ApprovalChoice::ApproveOnce);

        assert!(matches!(
            tui.transcript.entries().last().map(|entry| &entry.block),
            Some(TranscriptBlock::StyledLine(line)) if line.text.contains("approved: write_file")
        ));
    }
}

impl Tui {
    pub(super) async fn next_stream_chunk(&mut self) -> Option<String> {
        match self.active_stream.as_mut() {
            Some(stream) => {
                let chunk = stream.next().await;
                if chunk.is_none() {
                    self.finalize_active_stream();
                }
                chunk
            }
            None => std::future::pending().await,
        }
    }

    pub(super) fn finalize_active_stream(&mut self) {
        let lines = self.stream_render.finish();
        if self.stream_opening_pending {
            self.stream_opening_pending = false;
            self.pending_stream_opening.clear();
        } else {
            self.append_styled_lines(lines);
        }
        self.active_stream = None;
    }

    pub(super) fn finalize_ordered_content(&mut self) {
        if !self.ordered_content_open {
            return;
        }
        self.flush_tool_placeholder();
        let lines = self.stream_render.finish();
        if self.stream_opening_pending {
            self.stream_opening_pending = false;
            self.pending_stream_opening.clear();
        } else {
            self.append_styled_lines(lines);
        }
        self.ordered_content_open = false;
    }

    pub(super) fn consume_stream_chunk(&mut self, chunk: &str) {
        let filter_out = self.text_filter.push_chunk(chunk);

        if filter_out.tool_call_started && self.active_stream.is_some() {
            self.finalize_active_stream();
        }

        let text = self.tool_placeholder_gate.push(&filter_out.text);
        self.append_visible_text(&text);
    }

    fn append_visible_text(&mut self, text: &str) {
        if !text.is_empty() {
            if self.stream_opening_pending {
                let opening = crate::scrollback::stream_opening_lines(
                    self.stream_count,
                    std::mem::take(&mut self.pending_stream_opening),
                );
                self.append_styled_lines(opening);
                self.stream_opening_pending = false;
                self.stream_count += 1;
            }
            let lines = self.stream_render.push_chunk(text);
            self.append_styled_lines(lines);
        }
    }

    fn flush_tool_placeholder(&mut self) {
        let text = self.tool_placeholder_gate.flush();
        self.append_visible_text(&text);
    }

    pub(super) fn handle_ui_output(&mut self, output: UiOutput) -> bool {
        match output {
            UiOutput::Content(content) => match content {
                ContentOutput::Start { source } => {
                    if self.active_stream.is_some() {
                        self.finalize_active_stream();
                    }
                    self.finalize_ordered_content();
                    self.tool_placeholder_gate.start(&source);
                    self.pending_stream_opening = self.stream_render.start(source);
                    self.stream_opening_pending = true;
                    self.ordered_content_open = true;
                }
                ContentOutput::Delta { text } => {
                    if self.ordered_content_open {
                        self.consume_stream_chunk(&text);
                    }
                }
                ContentOutput::End => {
                    let text = self.tool_placeholder_gate.finish_content();
                    self.append_visible_text(&text);
                    if !self.tool_placeholder_gate.is_pending() {
                        self.finalize_ordered_content();
                    }
                }
                ContentOutput::Block { source, text } => {
                    if self.active_stream.is_some() {
                        self.finalize_active_stream();
                    }
                    self.finalize_ordered_content();
                    let lines = crate::scrollback::render_history_message(
                        &mut self.stream_count,
                        source,
                        &text,
                    );
                    self.append_styled_lines(lines);
                }
            },
            UiOutput::Stream(msg) => {
                self.finalize_ordered_content();
                if self.active_stream.is_some() {
                    self.finalize_active_stream();
                }
                self.pending_stream_opening = self.stream_render.start(msg.source.clone());
                self.stream_opening_pending = true;
                self.active_stream = Some(msg.stream);
            }
            UiOutput::Reasoning(text) => {
                let lines = crate::scrollback::render_history_message(
                    &mut self.stream_count,
                    talos_conversation::MessageSource::Reasoning,
                    &text,
                );
                self.append_styled_lines(lines);
            }
            UiOutput::ToolCallStarted { .. } => {
                if !self.tool_placeholder_gate.is_pending() {
                    self.finalize_ordered_content();
                }
                if self.active_stream.is_some() {
                    self.finalize_active_stream();
                }
            }
            UiOutput::ToolCall(display) => {
                let text = self.tool_placeholder_gate.confirm_tool_call();
                self.append_visible_text(&text);
                self.finalize_ordered_content();
                self.pending_transcript
                    .push(TranscriptBlock::ToolCall(display));
            }
            UiOutput::ToolResult(display) => {
                self.finalize_ordered_content();
                let icon = if display.is_error { "✗" } else { "" };
                let color = if display.is_error {
                    to_crossterm_color(semantic::TEXT_ERROR)
                } else {
                    to_crossterm_color(semantic::TEXT_SUCCESS)
                };
                let _ = (icon, color);
                self.pending_transcript
                    .push(TranscriptBlock::ToolResult(display));
            }
            UiOutput::TodoPanel(data) => {
                self.append_styled_lines(build_todo_panel_lines(&data));
            }
            UiOutput::ThinkingPreview { text } => {
                self.state.thinking_preview = text;
            }
            UiOutput::ToolApprovalRequest {
                tool_name,
                arguments,
                summary_fields,
                preview,
                response,
            } => {
                self.state.pending_approval_response = Some(response);
                let args_str = serde_json::to_string_pretty(&arguments)
                    .unwrap_or_else(|_| arguments.to_string());
                let summary = crate::tool_display::summarize_tool_args(
                    &tool_name,
                    &args_str,
                    &summary_fields,
                );
                self.show_approval_with_preview(
                    &tool_name,
                    &summary,
                    preview.filter(|value| !value.is_empty()),
                );
            }
            UiOutput::Status(snapshot) => {
                if !snapshot.is_processing {
                    self.finalize_ordered_content();
                }
                let workspace_path = std::mem::take(&mut self.state.status.workspace_path);
                self.state.status = snapshot;
                if self.state.status.workspace_path.is_empty() {
                    self.state.status.workspace_path = workspace_path;
                }
            }
            UiOutput::SessionIdentity { id } => {
                self.session_id = Some(id);
            }
            UiOutput::Tip { text, kind } => {
                self.state.tip = Some(Tip {
                    ttl: tip_ttl(&kind),
                    kind,
                    text,
                    created_at: Instant::now(),
                });
            }
            UiOutput::CopyToClipboard { text, scope } => {
                let label = match scope {
                    CopyScope::Last => "last message",
                    CopyScope::All => "transcript",
                };
                match crate::clipboard::copy_text(&text) {
                    Ok(backend) => {
                        self.state.tip = Some(Tip {
                            kind: TipKind::Info,
                            text: format!("Copied {label} to clipboard (via {backend:?})",),
                            ttl: Duration::from_secs(3),
                            created_at: Instant::now(),
                        });
                    }
                    Err(e) => {
                        self.state.tip = Some(Tip {
                            kind: TipKind::Error,
                            text: format!("Failed to copy {label}: {e:?}"),
                            ttl: Duration::from_secs(4),
                            created_at: Instant::now(),
                        });
                    }
                }
            }
            UiOutput::ExportToFile { path, content } => {
                let engine = talos_permission::PermissionEngine::default();
                match crate::export::export_transcript(&engine, &path, &content) {
                    Ok(()) => {
                        self.state.tip = Some(Tip {
                            kind: TipKind::Info,
                            text: format!("Exported transcript to {}", path.display()),
                            ttl: Duration::from_secs(3),
                            created_at: Instant::now(),
                        });
                    }
                    Err(crate::export::ExportError::PermissionDenied(reason)) => {
                        self.state.tip = Some(Tip {
                            kind: TipKind::Error,
                            text: format!("Export denied: {reason}"),
                            ttl: Duration::from_secs(4),
                            created_at: Instant::now(),
                        });
                    }
                    Err(crate::export::ExportError::WriteFailed(reason)) => {
                        self.state.tip = Some(Tip {
                            kind: TipKind::Error,
                            text: format!("Export failed: {reason}"),
                            ttl: Duration::from_secs(4),
                            created_at: Instant::now(),
                        });
                    }
                }
            }
            UiOutput::Exit => {
                self.finalize_ordered_content();
                self.state.should_exit = true;
                return true;
            }
            UiOutput::SessionNew(_)
            | UiOutput::SessionResume(_)
            | UiOutput::SessionFork(_)
            | UiOutput::SessionDelete(_)
            | UiOutput::TodoCommand(_)
            | UiOutput::ModelSwitchRequest(_)
            | UiOutput::SkillCommand(_)
            | UiOutput::CredentialResponse(_) => {
                // Handled by the bridge → mode runner lifecycle handler.
                // Should not reach the TUI directly.
            }
            UiOutput::SessionPicker(sessions) => {
                self.state.open_session_picker(&sessions);
            }
            UiOutput::ModelPicker(data) => {
                self.state.open_model_picker(&data);
            }
            UiOutput::ConnectPicker(data) => {
                self.state.open_connect_picker(&data);
            }
            UiOutput::ConnectProviderRequest { .. } => {}
            UiOutput::SteeringQueueSnapshot(snapshot) => {
                self.state.steering_queue_snapshot = Some(snapshot);
            }
            UiOutput::CredentialRequest(req) => {
                self.state.open_credential_input(
                    &req.provider,
                    req.model_id.as_deref(),
                    req.connect_mode,
                    req.default_base_url.clone(),
                );
            }
            UiOutput::HydrateHistory(messages) => {
                self.finalize_ordered_content();
                self.finalize_active_stream();
                self.commit_pending_transcript().ok();
                self.hydrate_history(&messages);
                self.commit_pending_transcript().ok();
            }
            UiOutput::AttachImageRequest { .. } => {
                // Handled by the bridge — should not reach the TUI directly.
            }
        }
        false
    }

    pub(super) fn append_styled_lines(&mut self, lines: impl IntoIterator<Item = ScrollbackLine>) {
        self.pending_transcript
            .extend(lines.into_iter().map(TranscriptBlock::StyledLine));
    }

    pub(super) fn commit_pending_transcript(&mut self) -> io::Result<()> {
        for block in std::mem::take(&mut self.pending_transcript) {
            self.transcript.append(block);
        }
        Ok(())
    }

    pub(super) fn advance_processing_frame(&mut self) {
        self.processing_frame =
            next_processing_frame(self.state.status.is_processing, self.processing_frame);
        let Some(selection) = self.selection else {
            return;
        };
        if !selection.dragging
            || selection.edge == 0
            || selection.history_anchor.is_none()
            || self.last_history_viewport_height == 0
        {
            return;
        }
        if selection.edge < 0 {
            self.scroll_frame_history_up(1);
        } else {
            self.scroll_frame_history_down(1, self.last_history_viewport_height);
        }
    }
}
