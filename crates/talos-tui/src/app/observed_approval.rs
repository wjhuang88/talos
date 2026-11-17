use super::*;

// Dropping the running future must release approvals even when Tui is retained.
pub(super) struct ApprovalRunGuard<'a>(pub(super) &'a mut Tui);

impl Drop for ApprovalRunGuard<'_> {
    fn drop(&mut self) {
        self.0.retire_all_approvals();
    }
}

impl Tui {
    pub(super) fn drop_active_approval(&mut self) {
        self.observed_approval = None;
        self.state.pending_approval_response = None;
        self.hide_approval();
        if self.state.slash_menu.is_approval() {
            self.state.slash_menu.close();
        }
    }

    pub(super) fn retire_observed_approvals(&mut self) {
        if self.observed_approval.take().is_some() {
            self.hide_approval();
            if self.state.slash_menu.is_approval() {
                self.state.slash_menu.close();
            }
        }
        self.queued_outputs
            .retain(|output| !matches!(output, PresentedOutput::Observed(_)));
    }

    pub(super) fn retire_legacy_approvals(&mut self) {
        if self.state.pending_approval_response.take().is_some() && self.observed_approval.is_none()
        {
            self.hide_approval();
            if self.state.slash_menu.is_approval() {
                self.state.slash_menu.close();
            }
        }
        self.queued_outputs.retain(|output| {
            !matches!(
                output,
                PresentedOutput::Legacy(UiOutput::ToolApprovalRequest { .. })
            )
        });
    }

    pub(super) fn retire_all_approvals(&mut self) {
        self.drop_active_approval();
        self.queued_outputs.retain(|output| matches!(output, PresentedOutput::Legacy(output) if !matches!(output, UiOutput::ToolApprovalRequest { .. })));
        self.observed_approval_rx = None;
    }

    pub(super) fn handle_observed_approval(&mut self, request: crate::ObservedApprovalRequest) {
        if request.responder.expired() {
            return;
        }
        self.drop_active_approval();
        let args = serde_json::to_string_pretty(&request.arguments)
            .unwrap_or_else(|_| request.arguments.to_string());
        let summary = crate::tool_display::summarize_tool_args(
            &request.tool_name,
            &args,
            &request.summary_fields,
        );
        self.show_approval_with_preview(
            &request.tool_name,
            &summary,
            request.preview.filter(|value| !value.is_empty()),
        );
        self.state.slash_menu.approval_view.presentation = request.presentation;
        self.observed_approval = Some(request.responder);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ApprovalBinding, ApprovalResponseSource};

    fn make_request(
        binding: &ApprovalBinding,
    ) -> (crate::ObservedApprovalRequest, crate::ApprovalWaiter) {
        binding
            .request(
                "same-tool".into(),
                serde_json::json!({"path":"same"}),
                vec![],
                None,
                None,
            )
            .expect("request")
    }

    #[tokio::test]
    async fn actual_keys_distinguish_human_deny_from_explicit_cancellation() {
        for (key, choice, source) in [
            (
                KeyCode::Char('1'),
                ApprovalChoice::ApproveOnce,
                ApprovalResponseSource::Human,
            ),
            (
                KeyCode::Char('3'),
                ApprovalChoice::Deny,
                ApprovalResponseSource::Human,
            ),
            (
                KeyCode::Esc,
                ApprovalChoice::Deny,
                ApprovalResponseSource::UserCancellation,
            ),
        ] {
            let mut tui = Tui::for_test(TuiState::new(), None);
            let (request, waiter) = make_request(&ApprovalBinding::default());
            tui.handle_observed_approval(request);
            assert!(tui.state.pending_approval_response.is_none());
            tui.handle_pending_approval_input(key);
            let response = waiter.receive().await;
            assert_eq!(response.choice(), &choice);
            assert_eq!(response.source(), source);
        }
    }

    #[tokio::test]
    async fn invalidation_retires_active_queued_and_late_responses_without_reusing_ids() {
        let mut tui = Tui::for_test(TuiState::new(), None);
        let binding = ApprovalBinding::default();
        let (active, first) = make_request(&binding);
        let (queued, second) = make_request(&binding);
        assert_ne!(first.request_id(), second.request_id());
        tui.handle_observed_approval(active);
        tui.queued_outputs.push(PresentedOutput::Observed(queued));
        binding.invalidate();
        tui.handle_pending_approval_input(KeyCode::Char('1'));
        assert!(matches!(tui.state.approval_state, ApprovalState::Hidden));
        assert_eq!(
            first.receive().await.source(),
            ApprovalResponseSource::Unavailable
        );
        let PresentedOutput::Observed(queued) = tui.queued_outputs.remove(0) else {
            panic!("typed");
        };
        tui.handle_observed_approval(queued);
        assert!(tui.observed_approval.is_none());
        assert_eq!(
            second.receive().await.source(),
            ApprovalResponseSource::Unavailable
        );
        let (next, waiter) = make_request(&binding);
        tui.handle_observed_approval(next);
        tui.handle_pending_approval_input(KeyCode::Char('1'));
        // Even a response already in its oneshot is rejected if publication changed.
        binding.invalidate();
        assert_eq!(
            waiter.receive().await.source(),
            ApprovalResponseSource::Unavailable
        );
    }

    #[tokio::test]
    async fn exit_releases_both_transports_and_request_drop_is_not_human_denial() {
        let mut tui = Tui::for_test(TuiState::new(), None);
        let binding = ApprovalBinding::default();
        let (active, first) = make_request(&binding);
        let (queued, second) = make_request(&binding);
        tui.handle_observed_approval(active);
        tui.queued_outputs.push(PresentedOutput::Observed(queued));
        let (tx, rx) = tokio::sync::oneshot::channel();
        tui.queued_outputs
            .push(PresentedOutput::Legacy(UiOutput::ToolApprovalRequest {
                tool_name: "legacy".into(),
                arguments: serde_json::json!({}),
                summary_fields: vec![],
                presentation: None,
                preview: None,
                response: tx,
            }));
        tui.handle_ui_output(UiOutput::Exit);
        assert_eq!(
            first.receive().await.source(),
            ApprovalResponseSource::Unavailable
        );
        assert_eq!(
            second.receive().await.source(),
            ApprovalResponseSource::Unavailable
        );
        assert!(rx.await.is_err());
        let (request, waiter) = make_request(&binding);
        drop(request);
        assert_eq!(
            waiter.receive().await.source(),
            ApprovalResponseSource::Unavailable
        );
    }

    #[tokio::test]
    async fn typed_choices_preserve_once_only_and_full_reusable_scope_rules() {
        let binding = ApprovalBinding::default();
        let mut tui = Tui::for_test(TuiState::new(), None);
        let (mut request, waiter) = make_request(&binding);
        request.presentation = Some(talos_conversation::ApprovalPresentation {
            summary: vec!["one operation".into()],
            once_only: true,
        });
        tui.handle_observed_approval(request);
        tui.handle_pending_approval_input(KeyCode::Char('2'));
        assert_eq!(
            waiter.receive().await.choice(),
            &ApprovalChoice::ApproveOnce
        );
        let (mut request, waiter) = make_request(&binding);
        request.preview = Some("full reusable scope requiring review".into());
        tui.handle_observed_approval(request);
        tui.handle_pending_approval_input(KeyCode::Char('2'));
        assert!(
            tui.observed_approval.is_some(),
            "hidden scope must withhold reusable approval"
        );
        tui.handle_pending_approval_input(KeyCode::Char('1'));
        assert_eq!(
            waiter.receive().await.choice(),
            &ApprovalChoice::ApproveOnce
        );
    }

    #[tokio::test]
    async fn event_loop_wakes_on_binding_invalidation_and_exits_with_queued_requests() {
        let mut tui = Tui::for_test(TuiState::new(), None);
        let (output_tx, output_rx) = mpsc::unbounded_channel();
        let (approval_tx, approval_rx) = mpsc::unbounded_channel();
        tui.set_ui_output_rx(output_rx);
        tui.set_observed_approval_rx(approval_rx);
        let mut frames = tui.terminal.observe_test_frames();
        let binding = ApprovalBinding::default();
        let (request, waiter) = make_request(&binding);
        approval_tx.send(request).expect("send");
        let driver = async {
            loop {
                if frames
                    .recv()
                    .await
                    .expect("displayed frame")
                    .contains("same-tool")
                {
                    break;
                }
            }
            binding.invalidate();
            let response = waiter.receive().await;
            assert_eq!(response.source(), ApprovalResponseSource::Unavailable);
            loop {
                if !frames
                    .recv()
                    .await
                    .expect("retired frame")
                    .contains("same-tool")
                {
                    break;
                }
            }
            output_tx.send(UiOutput::Exit).expect("exit");
        };
        let (result, ()) = tokio::time::timeout(Duration::from_secs(2), async {
            tokio::join!(
                tui.run_with_scanner(futures::stream::pending(), |_, _| Ok(None)),
                driver
            )
        })
        .await
        .expect("event loop must wake");
        result.expect("loop");
        assert!(tui.observed_approval.is_none());
        assert!(matches!(tui.state.approval_state, ApprovalState::Hidden));
    }
    #[tokio::test]
    async fn dropping_running_future_releases_displayed_and_queued_approvals() {
        let mut tui = Tui::for_test(TuiState::new(), None);
        let (_output_tx, output_rx) = mpsc::unbounded_channel();
        tui.set_ui_output_rx(output_rx);
        let binding = ApprovalBinding::default();
        let (request, active) = make_request(&binding);
        tui.handle_observed_approval(request);
        let (request, queued) = make_request(&binding);
        tui.queued_outputs.push(PresentedOutput::Observed(request));
        let mut frames = tui.terminal.observe_test_frames();
        {
            let run = tui.run_with_scanner(futures::stream::pending(), |_, _| Ok(None));
            tokio::pin!(run);
            tokio::time::timeout(Duration::from_secs(2), async {
                loop {
                    tokio::select! {
                        result = &mut run => panic!("unexpected exit: {result:?}"),
                        frame = frames.recv() => {
                            if frame.expect("frame").contains("same-tool") {
                                break;
                            }
                        }
                    }
                }
            })
            .await
            .expect("modal displayed");
        }
        for waiter in [active, queued] {
            assert_eq!(
                tokio::time::timeout(Duration::from_secs(1), waiter.receive())
                    .await
                    .expect("cancelled run releases waiter")
                    .source(),
                ApprovalResponseSource::Unavailable
            );
        }
        assert!(tui.observed_approval.is_none());
        assert!(tui.queued_outputs.is_empty());
        tui.handle_pending_approval_input(KeyCode::Char('1'));
        assert!(matches!(tui.state.approval_state, ApprovalState::Hidden));
    }

    #[tokio::test]
    async fn typed_channel_eof_retires_modal_and_queue_while_loop_stays_running() {
        let mut tui = Tui::for_test(TuiState::new(), None);
        let (output_tx, output_rx) = mpsc::unbounded_channel();
        let (approval_tx, approval_rx) = mpsc::unbounded_channel();
        tui.set_ui_output_rx(output_rx);
        tui.set_observed_approval_rx(approval_rx);
        let binding = ApprovalBinding::default();
        let (request, active) = make_request(&binding);
        tui.handle_observed_approval(request);
        let (request, queued) = make_request(&binding);
        tui.queued_outputs.push(PresentedOutput::Observed(request));
        let mut frames = tui.terminal.observe_test_frames();
        let driver = async {
            while !frames.recv().await.expect("frame").contains("same-tool") {}
            drop(approval_tx);
            for waiter in [active, queued] {
                assert_eq!(
                    waiter.receive().await.source(),
                    ApprovalResponseSource::Unavailable
                );
            }
            while frames
                .recv()
                .await
                .expect("retired frame")
                .contains("same-tool")
            {}
            output_tx.send(UiOutput::Exit).expect("exit");
        };
        let (result, ()) = tokio::time::timeout(Duration::from_secs(2), async {
            tokio::join!(
                tui.run_with_scanner(futures::stream::pending(), |_, _| Ok(None)),
                driver
            )
        })
        .await
        .expect("EOF releases approvals without exiting loop");
        result.expect("run");
    }

    #[tokio::test]
    async fn rendering_failure_releases_responder_while_tui_remains_alive() {
        let mut tui = Tui::for_test(TuiState::new(), None);
        let (_tx, rx) = mpsc::unbounded_channel();
        tui.set_ui_output_rx(rx);
        let (request, waiter) = make_request(&ApprovalBinding::default());
        tui.handle_observed_approval(request);
        tui.terminal.fail_next_test_draw();
        let error = tui
            .run_with_scanner(futures::stream::pending(), |_, _| Ok(None))
            .await
            .expect_err("injected draw error");
        assert!(error.to_string().contains("injected terminal draw failure"));
        assert_eq!(
            waiter.receive().await.source(),
            ApprovalResponseSource::Unavailable
        );
        assert!(tui.observed_approval.is_none());
        tui.handle_pending_approval_input(KeyCode::Char('1'));
        assert!(matches!(tui.state.approval_state, ApprovalState::Hidden));
    }

    #[tokio::test]
    async fn session_change_retires_legacy_without_discarding_current_typed_request() {
        let mut tui = Tui::for_test(TuiState::new(), None);
        let (legacy_tx, legacy_rx) = tokio::sync::oneshot::channel();
        tui.handle_ui_output(UiOutput::ToolApprovalRequest {
            tool_name: "legacy".into(),
            arguments: serde_json::json!({}),
            summary_fields: vec![],
            presentation: None,
            preview: None,
            response: legacy_tx,
        });
        let (queued_tx, queued_rx) = tokio::sync::oneshot::channel();
        tui.queued_outputs
            .push(PresentedOutput::Legacy(UiOutput::ToolApprovalRequest {
                tool_name: "legacy-queued".into(),
                arguments: serde_json::json!({}),
                summary_fields: vec![],
                presentation: None,
                preview: None,
                response: queued_tx,
            }));
        let (request, waiter) = make_request(&ApprovalBinding::default());
        tui.queued_outputs.push(PresentedOutput::Observed(request));
        tui.handle_ui_output(UiOutput::SessionIdentity {
            id: "new-session".into(),
        });
        assert!(legacy_rx.await.is_err());
        assert!(queued_rx.await.is_err());
        assert_eq!(tui.queued_outputs.len(), 1);
        let PresentedOutput::Observed(request) = tui.queued_outputs.remove(0) else {
            panic!("typed retained");
        };
        tui.handle_observed_approval(request);
        tui.handle_pending_approval_input(KeyCode::Char('1'));
        assert_eq!(
            waiter.receive().await.source(),
            ApprovalResponseSource::Human
        );
    }

    #[tokio::test]
    async fn replacing_typed_receiver_retires_active_and_queued_without_closing_legacy() {
        let mut tui = Tui::for_test(TuiState::new(), None);
        let (old_tx, old_rx) = mpsc::unbounded_channel();
        tui.set_observed_approval_rx(old_rx);
        let binding = ApprovalBinding::default();
        let (request, active) = make_request(&binding);
        tui.handle_observed_approval(request);
        let (request, queued) = make_request(&binding);
        tui.queued_outputs.push(PresentedOutput::Observed(request));
        let (_new_tx, new_rx) = mpsc::unbounded_channel();
        tui.set_observed_approval_rx(new_rx);
        assert!(old_tx.is_closed());
        assert_eq!(
            active.receive().await.source(),
            ApprovalResponseSource::Unavailable
        );
        assert_eq!(
            queued.receive().await.source(),
            ApprovalResponseSource::Unavailable
        );
        assert!(tui.queued_outputs.is_empty());
        let (tx, mut rx) = tokio::sync::oneshot::channel();
        tui.handle_ui_output(UiOutput::ToolApprovalRequest {
            tool_name: "legacy".into(),
            arguments: serde_json::json!({}),
            summary_fields: vec![],
            presentation: None,
            preview: None,
            response: tx,
        });
        let (_tx, replacement) = mpsc::unbounded_channel();
        tui.set_observed_approval_rx(replacement);
        assert!(matches!(
            rx.try_recv(),
            Err(tokio::sync::oneshot::error::TryRecvError::Empty)
        ));
        tui.handle_pending_approval_input(KeyCode::Char('1'));
        assert_eq!(
            rx.await.expect("legacy choice"),
            ApprovalChoice::ApproveOnce
        );
    }
}
