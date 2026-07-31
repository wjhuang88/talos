from pathlib import Path
import re

# 1. Queue the canonical Started event before acknowledging SubmissionStarted.
path = Path('crates/talos-agent/src/session.rs')
text = path.read_text()
old = '''        let turn_id = format!("{}_{}", self.turn_prefix, turn_counter);
        if self
            .eq_tx
            .send(SessionEvent::SubmissionStarted {
                session_id: self.session_id.clone(),
                submission_id: submission.id.clone(),
                sender_generation: submission.sender_generation,
                turn_id: turn_id.clone(),
            })
            .is_err()
        {
            return None;
        }
        if self
            .eq_tx
            .send(SessionEvent::TurnEvent {
                session_id: self.session_id.clone(),
                turn_id: turn_id.clone(),
                sequence: 0,
                payload: TurnEventPayload::Started,
            })
            .is_err()
        {
            return None;
        }
'''
new = '''        let turn_id = format!("{}_{}", self.turn_prefix, turn_counter);
        if self
            .eq_tx
            .send(SessionEvent::TurnEvent {
                session_id: self.session_id.clone(),
                turn_id: turn_id.clone(),
                sequence: 0,
                payload: TurnEventPayload::Started,
            })
            .is_err()
        {
            return None;
        }
        if self
            .eq_tx
            .send(SessionEvent::SubmissionStarted {
                session_id: self.session_id.clone(),
                submission_id: submission.id.clone(),
                sender_generation: submission.sender_generation,
                turn_id: turn_id.clone(),
            })
            .is_err()
        {
            return None;
        }
'''
if old in text:
    text = text.replace(old, new, 1)
elif new not in text:
    raise SystemExit('unexpected session start event block')
path.write_text(text)

# 2. Bridge commits only after the post-Started acknowledgement and resumes at sequence 1.
path = Path('crates/talos-cli/src/tui_bridge.rs')
text = path.read_text()
old_local = '''                        if let Some((submission, cancel_requested)) = local {
                            if !engine.commit_prepared_steering(&submission_id) {
                                continue;
                            }
                            for item in &submission.items {
                                for output in engine.start_user_message(&item.text) {
                                    let _ = ui_tx.send(output);
                                }
                            }
                            let _ = ui_tx.send(UiOutput::SteeringQueueSnapshot(
                                engine.steering_queue_snapshot(),
                            ));
                            turn_state = if cancel_requested {
                                BridgeTurnState::Cancelling {
                                    session_id,
                                    turn_id,
                                    next_sequence: 0,
                                }
                            } else {
                                BridgeTurnState::Running {
                                    session_id,
                                    turn_id,
                                    next_sequence: 0,
                                }
                            };
                            continue;
                        }
'''
new_local = '''                        if let Some((submission, cancel_requested)) = local {
                            if !engine.commit_prepared_steering(&submission_id) {
                                continue;
                            }
                            for item in &submission.items {
                                for output in engine.start_user_message(&item.text) {
                                    let _ = ui_tx.send(output);
                                }
                            }
                            for output in engine.handle_turn_started() {
                                let _ = ui_tx.send(output);
                            }
                            let _ = ui_tx.send(UiOutput::SteeringQueueSnapshot(
                                engine.steering_queue_snapshot(),
                            ));
                            turn_state = if cancel_requested {
                                BridgeTurnState::Cancelling {
                                    session_id,
                                    turn_id,
                                    next_sequence: 1,
                                }
                            } else {
                                BridgeTurnState::Running {
                                    session_id,
                                    turn_id,
                                    next_sequence: 1,
                                }
                            };
                            continue;
                        }
'''
if old_local in text:
    text = text.replace(old_local, new_local, 1)
elif new_local not in text:
    raise SystemExit('unexpected local SubmissionStarted block')

old_external = '''                        for text in external.item_texts {
                            for output in engine.start_user_message(&text) {
                                let _ = ui_tx.send(output);
                            }
                        }
                        external_turn = Some(ExternalTurnState {
                            session_id,
                            turn_id,
                            next_sequence: 0,
                        });
'''
new_external = '''                        for text in external.item_texts {
                            for output in engine.start_user_message(&text) {
                                let _ = ui_tx.send(output);
                            }
                        }
                        for output in engine.handle_turn_started() {
                            let _ = ui_tx.send(output);
                        }
                        external_turn = Some(ExternalTurnState {
                            session_id,
                            turn_id,
                            next_sequence: 1,
                        });
'''
if old_external in text:
    text = text.replace(old_external, new_external, 1)
elif new_external not in text:
    raise SystemExit('unexpected external SubmissionStarted block')

needle = '''                    Some(SessionEvent::TurnEvent {
                        session_id,
                        turn_id,
                        sequence,
                        payload,
                    }) => {
                        let local_matching = match &turn_state {
'''
replacement = '''                    Some(SessionEvent::TurnEvent {
                        session_id,
                        turn_id,
                        sequence,
                        payload,
                    }) => {
                        let awaiting_submission_ack = sequence == 0
                            && matches!(&payload, TurnEventPayload::Started)
                            && (matches!(
                                &turn_state,
                                BridgeTurnState::Submitting {
                                    session_id: expected_session,
                                    ..
                                } if expected_session
                                    .as_ref()
                                    .is_none_or(|expected| expected == &session_id)
                            ) || external_pending
                                .iter()
                                .any(|pending| pending.session_id == session_id));
                        if awaiting_submission_ack {
                            // The actor queues canonical Started before emitting the
                            // correlated SubmissionStarted acknowledgement. Ownership
                            // remains prepared until that acknowledgement arrives.
                            continue;
                        }

                        let local_matching = match &turn_state {
'''
if needle in text:
    text = text.replace(needle, replacement, 1)
elif replacement not in text:
    raise SystemExit('unexpected TurnEvent handler preamble')
path.write_text(text)

# 3. Test helper follows the production event order.
path = Path('crates/talos-cli/src/tests.rs')
text = path.read_text()
old_helper = '''            self.tx
                .send(SessionEvent::SubmissionStarted {
                    session_id: "session_test".into(),
                    submission_id,
                    sender_generation,
                    turn_id: turn_id.into(),
                })
                .unwrap();
            self.tx
                .send(SessionEvent::TurnEvent {
                    session_id: "session_test".into(),
                    turn_id: turn_id.into(),
                    sequence: 0,
                    payload: TurnEventPayload::Started,
                })
                .unwrap();
'''
new_helper = '''            self.tx
                .send(SessionEvent::TurnEvent {
                    session_id: "session_test".into(),
                    turn_id: turn_id.into(),
                    sequence: 0,
                    payload: TurnEventPayload::Started,
                })
                .unwrap();
            self.tx
                .send(SessionEvent::SubmissionStarted {
                    session_id: "session_test".into(),
                    submission_id,
                    sender_generation,
                    turn_id: turn_id.into(),
                })
                .unwrap();
'''
if old_helper in text:
    text = text.replace(old_helper, new_helper, 1)
elif new_helper not in text:
    raise SystemExit('unexpected TestTurnSender start order')
path.write_text(text)

# 4. Add an actor-level deterministic ordering regression.
path = Path('crates/talos-agent/tests/pr68_lifecycle.rs')
text = path.read_text()
test_name = 'authoritative_turn_start_precedes_submission_ack'
if test_name not in text:
    test = r'''

#[tokio::test]
async fn authoritative_turn_start_precedes_submission_ack() {
    let (handle, mut actor) = AppServerSession::new(make_agent(), config(128_000));
    let sq_tx = handle.sq_tx;
    let mut eq_rx = handle.eq_rx;
    let actor_task = tokio::spawn(async move { actor.run().await });

    sq_tx
        .send(SessionOp::SubmitStructured {
            submission: submission("ordered_batch", "ordered_item", SubmissionSource::User),
        })
        .await
        .unwrap();

    let mut saw_canonical_start = false;
    loop {
        let event = tokio::time::timeout(Duration::from_secs(2), eq_rx.recv())
            .await
            .expect("start ordering timeout")
            .expect("session event channel");
        match event {
            SessionEvent::TurnEvent {
                sequence: 0,
                payload: talos_core::session::TurnEventPayload::Started,
                ..
            } => saw_canonical_start = true,
            SessionEvent::SubmissionStarted { submission_id, .. }
                if submission_id == "ordered_batch" =>
            {
                assert!(
                    saw_canonical_start,
                    "canonical Started must be queued before ownership acknowledgement"
                );
                break;
            }
            _ => {}
        }
    }

    sq_tx.send(SessionOp::Shutdown).await.unwrap();
    actor_task.await.unwrap();
}
'''
    text += test
path.write_text(text)

# 5. Remove the destructive joined drain from production API surface.
path = Path('crates/talos-conversation/src/engine.rs')
text = path.read_text()
if '#[cfg(test)]\n    pub(crate) fn drain_steering_queue_batched' not in text:
    pattern = re.compile(
        r'(?m)(?:^    ///.*\n)+'
        r'^    pub fn drain_steering_queue_batched\(&mut self\) -> Option<String> \{'
    )
    replacement = (
        '    /// Test-only projection for validating the retired joined-drain behavior.\n'
        '    /// Production code must use structured prepare/commit/rollback.\n'
        '    #[cfg(test)]\n'
        '    pub(crate) fn drain_steering_queue_batched(&mut self) -> Option<String> {'
    )
    text, count = pattern.subn(replacement, text, count=1)
    if count != 1:
        raise SystemExit('expected one public batched drain declaration')
old_docs = (
    '    /// This method remains available for downstream callers that need the\n'
    '    /// original single-item behavior. Talos\'s interactive runtime uses\n'
    '    /// [`Self::drain_steering_queue_batched`] for TUI-041 / GitHub Issue #50.\n'
)
new_docs = (
    '    /// This compatibility helper preserves the original single-item behavior.\n'
    '    /// Talos interactive runtime uses structured prepare/commit/rollback.\n'
)
if old_docs in text:
    text = text.replace(old_docs, new_docs, 1)
path.write_text(text)

# 6. Align the Chinese README with the structured transactional semantics.
path = Path('README.zh-CN.md')
text = path.read_text()
old_readme = '如果你在模型仍在处理时连续输入多条消息，它们会自动排队。当前 turn 完成后，Talos 会把当时队列中的所有输入按 FIFO 顺序合并为一个后续用户 turn，每条输入保持原文并以空行分隔；在该后续 turn 期间新到达的输入会进入下一批。TUI 在输入区上方显示排队消息的紧凑预览（最多 6 行；较长队列显示 `+N more` 摘要），批次取出后预览自动清空。'
new_readme = '如果你在模型仍在处理时连续输入多条消息，它们会作为带独立 identity、kind、source 与附件的结构化条目进入有界 FIFO 队列。当前 turn 成功完成后，Talos 会准备一个有界前缀并通过 Session Actor 的关联确认事务式提交；Actor 接受前不会删除原队列，发送失败、上下文拒绝、取消或 session sender 切换都会按原顺序保留或回滚。Provider 请求仍将每条输入保留为独立用户消息，不使用空行拼接作为权威模型。在后续 turn 期间新到达的输入进入下一批。TUI 在输入区上方显示排队消息的紧凑预览（最多 6 行；较长队列显示 `+N more` 摘要），只有权威提交确认后对应预览才会移除。'
if old_readme in text:
    text = text.replace(old_readme, new_readme, 1)
elif new_readme not in text:
    raise SystemExit('expected Chinese queued-input paragraph')
path.write_text(text)
