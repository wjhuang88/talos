use super::*;

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

        if !filter_out.text.is_empty() {
            if self.stream_opening_pending {
                let opening = crate::scrollback::stream_opening_lines(
                    self.stream_count,
                    std::mem::take(&mut self.pending_stream_opening),
                );
                self.append_styled_lines(opening);
                self.stream_opening_pending = false;
                self.stream_count += 1;
            }
            let lines = self.stream_render.push_chunk(&filter_out.text);
            self.append_styled_lines(lines);
        }
    }

    pub(super) fn handle_ui_output(&mut self, output: UiOutput) -> bool {
        match output {
            UiOutput::Content(content) => match content {
                ContentOutput::Start { source } => {
                    if self.active_stream.is_some() {
                        self.finalize_active_stream();
                    }
                    self.finalize_ordered_content();
                    self.pending_stream_opening = self.stream_render.start(source);
                    self.stream_opening_pending = true;
                    self.ordered_content_open = true;
                }
                ContentOutput::Delta { text } => {
                    if self.ordered_content_open {
                        self.consume_stream_chunk(&text);
                    }
                }
                ContentOutput::End => self.finalize_ordered_content(),
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
                self.finalize_ordered_content();
                if self.active_stream.is_some() {
                    self.finalize_active_stream();
                }
            }
            UiOutput::ToolCall(display) => {
                self.pending_transcript
                    .push(TranscriptBlock::ToolCall(display));
            }
            UiOutput::ToolResult(display) => {
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
                self.show_approval(&tool_name, &summary);
            }
            UiOutput::Status(snapshot) => {
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
    }
}
