// Bridge between the conversation engine and the TUI.
//
// Contains the conversation loop that mediates between agent events,
// user input, and UI output channels.

mod legacy_projection;

use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;
use tokio::time::MissedTickBehavior;

use crate::background_projection::format_background_terminal;
use crate::mode_runtime::request_preview_payload;
use crate::session_transition::authoritative_generation_for_sender;
use crate::skill_runtime::RuntimeSkills;
use talos_conversation::MessageSource;
use talos_conversation::{
    ContentOutput, ConversationEngine, CredentialResponseData, ModelInfo, ModelSwitchRequest,
    SessionDeleteRequest, SessionForkRequest, SessionNewRequest, SessionResumeRequest,
    SkillCommandRequest, TipKind, TodoCommandRequest, UiOutput, UserInput,
};
use talos_core::message::AgentEvent;
use talos_core::session::{
    PendingSubmissionState, SessionEvent, SessionOp, StructuredSubmission, SubmissionKind,
    SubmissionReceiptDisposition, SubmissionSource, TurnCompletionStatus, TurnEventPayload,
};

const RECEIPT_RECONCILE_AFTER: Duration = Duration::from_secs(1);
const RECEIPT_RECONCILE_TICK: Duration = Duration::from_millis(250);
const RECEIPT_WARNING_ATTEMPT: u32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ProgressMode {
    Unknown,
    Legacy,
    Structured,
}

#[derive(Debug)]
struct DeferredAcceptedSubmission {
    session_id: String,
    session_generation: u64,
    submission_id: String,
    receipt_id: String,
    submission: StructuredSubmission,
    projected: bool,
    cancel_requested: bool,
}

impl DeferredAcceptedSubmission {
    fn into_turn_state(self) -> BridgeTurnState {
        BridgeTurnState::AcceptedByActor {
            session_id: self.session_id,
            session_generation: self.session_generation,
            submission_id: self.submission_id,
            receipt_id: self.receipt_id,
            submission: self.submission,
            projected: self.projected,
            cancel_requested: self.cancel_requested,
        }
    }
}

#[derive(Debug)]
struct BoundarySubmission {
    submission: StructuredSubmission,
    sender_generation: u64,
    command_sender: tokio::sync::mpsc::Sender<SessionOp>,
    cancel_requested: bool,
    last_reconcile: Instant,
    reconcile_attempts: u32,
}

#[derive(Debug)]
pub(super) enum BridgeTurnState {
    Idle,
    Submitting {
        submission: StructuredSubmission,
        session_id: Option<String>,
        sender_generation: u64,
        command_sender: tokio::sync::mpsc::Sender<SessionOp>,
        cancel_requested: bool,
        last_reconcile: Instant,
        reconcile_attempts: u32,
    },
    AcceptedByActor {
        session_id: String,
        session_generation: u64,
        submission_id: String,
        receipt_id: String,
        submission: StructuredSubmission,
        projected: bool,
        cancel_requested: bool,
    },
    StructuredRunning {
        session_id: String,
        session_generation: u64,
        submission_id: String,
        receipt_id: String,
        turn_id: String,
        next_structured_sequence: u64,
        next_legacy_sequence: u64,
        progress_mode: ProgressMode,
    },
    StructuredCancelling {
        session_id: String,
        session_generation: u64,
        submission_id: String,
        receipt_id: String,
        turn_id: String,
        next_structured_sequence: u64,
        next_legacy_sequence: u64,
        progress_mode: ProgressMode,
    },
    LegacyRunning {
        session_id: String,
        turn_id: String,
        next_sequence: u64,
        sender_generation: u64,
    },
    LegacyCancelling {
        session_id: String,
        turn_id: String,
        next_sequence: u64,
        sender_generation: u64,
    },
    /// Exact durable submission that failed before Provider start. New user
    /// input remains Engine-owned until this identity is explicitly resolved.
    PreStartPaused {
        session_id: String,
        session_generation: u64,
        submission_id: String,
        receipt_id: String,
    },
    PreStartCancelling {
        session_id: String,
        session_generation: u64,
        submission_id: String,
        receipt_id: String,
    },
    PausedAfterFailure,
}

impl BridgeTurnState {
    fn accepts_queued_input(&self) -> bool {
        !matches!(self, Self::Idle | Self::PausedAfterFailure)
    }

    fn blocks_session_mutation(&self) -> bool {
        !matches!(self, Self::Idle)
    }
}

fn session_mutation_blocked(state: &BridgeTurnState, engine: &ConversationEngine) -> bool {
    state.blocks_session_mutation()
        || engine.has_steering()
        || !engine.pending_image_attachments.is_empty()
}

fn completion_allows_queued_continuation(
    status: &TurnCompletionStatus,
    cancellation_requested: bool,
) -> bool {
    matches!(status, TurnCompletionStatus::Success { .. })
        || cancellation_requested && matches!(status, TurnCompletionStatus::Cancelled)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum QueuedDispatch {
    None,
    CurrentRoute,
    Generation(u64),
}

pub(crate) struct ConversationLoopIo {
    pub agent_rx: tokio::sync::mpsc::UnboundedReceiver<SessionEvent>,
    pub user_rx: tokio::sync::mpsc::UnboundedReceiver<UserInput>,
    pub ui_tx: tokio::sync::mpsc::UnboundedSender<UiOutput>,
    pub sq_tx_watch:
        tokio::sync::watch::Receiver<tokio::sync::mpsc::Sender<talos_core::session::SessionOp>>,
    pub model_info_watch: tokio::sync::watch::Receiver<ModelInfo>,
    pub session_tx: tokio::sync::mpsc::UnboundedSender<SessionLifecycleRequest>,
    pub runtime_skills: Arc<Mutex<RuntimeSkills>>,
    /// Optional SEC-001/ADR-047 permission engine for image attachment
    /// authorization (P1-A). When `Some`, the bridge evaluates every
    /// `/attach` path against this engine and prompts the user for
    /// external paths. When `None`, the bridge skips authorization
    /// (test fixtures only).
    pub permission_engine: Option<Arc<talos_permission::PermissionSessionState>>,
}

pub(crate) async fn run_conversation_loop(mut engine: ConversationEngine, io: ConversationLoopIo) {
    let ConversationLoopIo {
        mut agent_rx,
        mut user_rx,
        ui_tx,
        mut sq_tx_watch,
        mut model_info_watch,
        session_tx,
        runtime_skills,
        permission_engine,
    } = io;

    engine.set_model_info(&model_info_watch.borrow().clone());
    let _ = ui_tx.send(UiOutput::Status(engine.status_snapshot()));
    let mut turn_state = BridgeTurnState::Idle;
    let mut deferred_accepted = None;
    let mut boundary_submission: Option<BoundarySubmission> = None;
    let mut known_sender = sq_tx_watch.borrow().clone();
    let mut sender_generation = authoritative_generation_for_sender(&known_sender).unwrap_or(0);
    let mut pending_attachment_generation =
        (!engine.pending_image_attachments.is_empty()).then_some(sender_generation);
    let mut receipt_tick = tokio::time::interval(RECEIPT_RECONCILE_TICK);
    receipt_tick.set_missed_tick_behavior(MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            changed = sq_tx_watch.changed() => {
                if changed.is_ok() {
                    let current_sender = sq_tx_watch.borrow().clone();
                    if !known_sender.same_channel(&current_sender) {
                        if let Some(current_generation) =
                            authoritative_generation_for_sender(&current_sender)
                        {
                            sender_generation = current_generation;
                            known_sender = current_sender;
                            // A new/resumed/forked session publishes a fresh command sender.
                            // Clear the prior session's non-persistent auto override only after
                            // the replacement runtime is authoritative.
                            engine.reset_auto_override();
                            if !engine.pending_image_attachments.is_empty() {
                                pending_attachment_generation = Some(sender_generation);
                                send_bridge_stream(
                                    &ui_tx,
                                    MessageSource::System,
                                    "[System] Pending attachments were retained across the session runtime replacement.\n".into(),
                                );
                            }
                        } else {
                            emit_bridge_error(
                                &ui_tx,
                                "session lifecycle published a command Sender without an authoritative generation",
                            );
                        }
                    }
                }
            }
            changed = model_info_watch.changed() => {
                if changed.is_ok() {
                    let info = model_info_watch.borrow().clone();
                    engine.set_model_info(&info);
                    let _ = ui_tx.send(UiOutput::Status(engine.status_snapshot()));
                }
            }
            _ = receipt_tick.tick() => {
                retry_submission_receipt(&mut turn_state, &mut boundary_submission, &ui_tx);
            }
            event = agent_rx.recv() => {
                match event {
                    Some(event) => {
                        handle_session_event(
                            event,
                            &mut engine,
                            &mut turn_state,
                            &mut deferred_accepted,
                            &mut boundary_submission,
                            &sq_tx_watch,
                            &mut known_sender,
                            &mut sender_generation,
                            &ui_tx,
                        ).await;
                    }
                    None => break,
                }
            }
            Some(input) = user_rx.recv() => {
                match input {
                    UserInput::Message(msg) => {
                        if msg.starts_with('/')
                            && !ConversationEngine::is_model_passthrough_slash_command(&msg)
                        {
                            let outputs = engine.handle_slash_command(&msg);
                            for output in outputs {
                                match output {
                                    UiOutput::Exit => {
                                        let _ = ui_tx.send(UiOutput::Exit);
                                        return;
                                    }
                                    UiOutput::SessionNew(req) => {
                                        if session_mutation_blocked(&turn_state, &engine) {
                                            emit_bridge_error(&ui_tx, "finish or cancel queued work before changing sessions");
                                        } else {
                                            let _ = session_tx.send(SessionLifecycleRequest::New(req));
                                        }
                                    }
                                    UiOutput::SessionResume(req) => {
                                        if session_mutation_blocked(&turn_state, &engine) {
                                            emit_bridge_error(&ui_tx, "finish or cancel queued work before changing sessions");
                                        } else {
                                            let _ = session_tx.send(SessionLifecycleRequest::Resume(req));
                                        }
                                    }
                                    UiOutput::SessionFork(req) => {
                                        if session_mutation_blocked(&turn_state, &engine) {
                                            emit_bridge_error(&ui_tx, "finish or cancel queued work before changing sessions");
                                        } else {
                                            let _ = session_tx.send(SessionLifecycleRequest::Fork(req));
                                        }
                                    }
                                    UiOutput::SessionDelete(req) => {
                                        if session_mutation_blocked(&turn_state, &engine) {
                                            emit_bridge_error(&ui_tx, "finish or cancel queued work before changing sessions");
                                        } else {
                                            let _ = session_tx.send(SessionLifecycleRequest::Delete(req));
                                        }
                                    }
                                    UiOutput::TodoCommand(req) => {
                                        if session_mutation_blocked(&turn_state, &engine) {
                                            emit_bridge_error(&ui_tx, "finish or cancel queued work before changing session state");
                                        } else {
                                            let _ = session_tx.send(SessionLifecycleRequest::Todo(req));
                                        }
                                    }
                                    UiOutput::ModelSwitchRequest(req) => {
                                        if session_mutation_blocked(&turn_state, &engine) {
                                            emit_bridge_error(&ui_tx, "finish or cancel queued work before changing models");
                                            continue;
                                        }
                                        if req.model_id.trim().is_empty() {
                                            let _ = session_tx.send(SessionLifecycleRequest::ModelSwitch(req));
                                        } else {
                                            let _ = ui_tx.send(UiOutput::Content(ContentOutput::Block {
                                                source: MessageSource::System,
                                                text: format!(
                                                    "[System] /model no longer accepts arguments. Opening the model picker — use the panel search to find '{}'.\n",
                                                    req.model_id.trim()
                                                ),
                                            }));
                                            let _ = session_tx.send(SessionLifecycleRequest::ModelSwitch(
                                                ModelSwitchRequest {
                                                    model_id: String::new(),
                                                    provider_needs_credential: false,
                                                    provider_hint: None,
                                                },
                                            ));
                                        }
                                    }
                                    UiOutput::ConnectProviderRequest { provider } => {
                                        if session_mutation_blocked(&turn_state, &engine) {
                                            emit_bridge_error(&ui_tx, "finish or cancel queued work before changing providers");
                                            continue;
                                        }
                                        if provider.trim().is_empty() {
                                            let _ = session_tx.send(SessionLifecycleRequest::ConnectRequest { provider });
                                        } else {
                                            let _ = ui_tx.send(UiOutput::Content(ContentOutput::Block {
                                                source: MessageSource::System,
                                                text: format!(
                                                    "[System] /connect no longer accepts arguments. Opening the provider picker — use the panel search to find '{}'.\n",
                                                    provider.trim()
                                                ),
                                            }));
                                            let _ = session_tx.send(SessionLifecycleRequest::ConnectRequest {
                                                provider: String::new(),
                                            });
                                        }
                                    }
                                    UiOutput::SkillCommand(req) => {
                                        if session_mutation_blocked(&turn_state, &engine) {
                                            emit_bridge_error(&ui_tx, "finish or cancel queued work before changing skills");
                                        } else {
                                            handle_skill_command(
                                                req,
                                                &mut engine,
                                                &ui_tx,
                                                &sq_tx_watch,
                                                runtime_skills.clone(),
                                            ).await;
                                        }
                                    }
                                    UiOutput::AttachImageRequest { path } => {
                                        if turn_state.blocks_session_mutation() || engine.has_steering() {
                                            emit_bridge_error(&ui_tx, "finish or cancel queued work before changing attachments");
                                            continue;
                                        }
                                        if !engine.image_input_capability.allows_attachment() {
                                            let _ = ui_tx.send(UiOutput::Content(ContentOutput::Block {
                                                source: MessageSource::Error,
                                                text: format!(
                                                    "[Error] Active model does not support image input (capability: {:?}). /attach rejected before any file read. Use /model to switch to a vision-capable model.\n",
                                                    engine.image_input_capability
                                                ),
                                            }));
                                            continue;
                                        }
                                        let Some(authorized_canonical) = authorize_attach_image(&ui_tx, &permission_engine, &path).await else {
                                            continue;
                                        };
                                        match crate::image_validation::create_image_content_part(
                                            &authorized_canonical,
                                            engine.pending_image_attachments.len(),
                                            engine.pending_image_attachments.iter().map(|p| {
                                                match p {
                                                    talos_core::message::ContentPart::Image { byte_count, .. } => *byte_count,
                                                    _ => 0,
                                                }
                                            }).sum::<u64>(),
                                        ) {
                                            Ok(content_part) => {
                                                let summary = match &content_part {
                                                    talos_core::message::ContentPart::Image { path, mime, byte_count, .. } =>
                                                        attachment_summary(path, mime, *byte_count),
                                                    _ => String::new(),
                                                };
                                                engine.pending_image_attachments.push(content_part);
                                                pending_attachment_generation = Some(sender_generation);
                                                let _ = ui_tx.send(UiOutput::Content(ContentOutput::Block {
                                                    source: MessageSource::System,
                                                    text: format!("[System] Attached image: {summary}\n"),
                                                }));
                                                let _ = ui_tx.send(UiOutput::Status(engine.status_snapshot()));
                                            }
                                            Err(e) => {
                                                let _ = ui_tx.send(UiOutput::Content(ContentOutput::Block {
                                                    source: MessageSource::Error,
                                                    text: format!("[Error] Image attachment failed: {e}\n"),
                                                }));
                                            }
                                        }
                                    }
                                    other => {
                                        let _ = ui_tx.send(other);
                                    }
                                }
                            }
                        } else {
                            let (kind, text) = match request_preview_payload(&msg) {
                                Some(payload) => (SubmissionKind::PreviewRequest, payload),
                                None => (SubmissionKind::UserTurn, msg),
                            };
                            if !engine.pending_image_attachments.is_empty() {
                                pending_attachment_generation = Some(sender_generation);
                            }
                            let attachments = std::mem::take(&mut engine.pending_image_attachments);
                            let attachment_generation = pending_attachment_generation.take();
                            let queued_behind_active_turn = turn_state.accepts_queued_input();
                            let (accepted, outputs) = engine.enqueue_structured_steering(
                                text,
                                kind,
                                attachments.clone(),
                            );
                            if !accepted {
                                engine.pending_image_attachments = attachments;
                                pending_attachment_generation = attachment_generation;
                            }
                            for output in outputs {
                                if should_forward_enqueue_output(
                                    &output,
                                    queued_behind_active_turn,
                                ) {
                                    let _ = ui_tx.send(output);
                                }
                            }
                            if !queued_behind_active_turn {
                                dispatch_prepared_submission(
                                    &mut engine,
                                    &mut turn_state,
                                    &sq_tx_watch,
                                    &mut known_sender,
                                    &mut sender_generation,
                                    &ui_tx,
                                    QueuedDispatch::CurrentRoute,
                                )
                                .await;
                            }
                        }
                    }
                    UserInput::Credential(resp) => {
                        if session_mutation_blocked(&turn_state, &engine) {
                            emit_bridge_error(&ui_tx, "finish or cancel queued work before applying credentials");
                        } else if resp.connect_mode {
                            let _ = session_tx.send(SessionLifecycleRequest::ConnectWithCredential(resp));
                        } else {
                            let _ = session_tx.send(SessionLifecycleRequest::ModelSwitchWithCredential(resp));
                        }
                    }
                    UserInput::ProviderSetup(provider) => {
                        if session_mutation_blocked(&turn_state, &engine) {
                            emit_bridge_error(&ui_tx, "finish or cancel queued work before changing providers");
                        } else {
                            let _ = session_tx.send(SessionLifecycleRequest::ProviderSetup(provider));
                        }
                    }
                    UserInput::SwitchModel { provider, model_id, variant } => {
                        if session_mutation_blocked(&turn_state, &engine) {
                            emit_bridge_error(&ui_tx, "finish or cancel queued work before changing models");
                            continue;
                        }
                        let value = match variant {
                            Some(v) if !v.is_empty() => format!("{model_id}@{v}"),
                            _ => model_id,
                        };
                        let _ = session_tx.send(SessionLifecycleRequest::ModelSwitch(
                            ModelSwitchRequest {
                                model_id: value,
                                provider_needs_credential: false,
                                provider_hint: if provider.is_empty() { None } else { Some(provider) },
                            },
                        ));
                    }
                    UserInput::ConnectSelect { provider } => {
                        if session_mutation_blocked(&turn_state, &engine) {
                            emit_bridge_error(&ui_tx, "finish or cancel queued work before changing providers");
                        } else {
                            let _ = session_tx.send(SessionLifecycleRequest::ConnectRequest { provider });
                        }
                    }
                    UserInput::RegisterCustomProvider { name, protocol, base_url, api_key } => {
                        if session_mutation_blocked(&turn_state, &engine) {
                            emit_bridge_error(&ui_tx, "finish or cancel queued work before changing providers");
                        } else {
                            let _ = session_tx.send(SessionLifecycleRequest::RegisterCustomProvider {
                                name,
                                protocol,
                                base_url,
                                api_key,
                            });
                        }
                    }
                    UserInput::Cancel => {
                        request_cancel(&mut turn_state, &sq_tx_watch, &ui_tx);
                    }
                    UserInput::Exit => {
                        let _ = ui_tx.send(UiOutput::Exit);
                        break;
                    }
                }
            }
        }
    }
}

fn should_forward_enqueue_output(output: &UiOutput, queued_behind_active_turn: bool) -> bool {
    queued_behind_active_turn
        || !matches!(
            output,
            UiOutput::Tip {
                kind: TipKind::QueueHint,
                ..
            }
        )
}

#[cfg(test)]
mod enqueue_output_tests {
    use super::*;

    #[test]
    fn idle_first_submit_suppresses_only_the_queue_hint() {
        let hint = UiOutput::Tip {
            text: "Message queued and will send after current turn.".into(),
            kind: TipKind::QueueHint,
        };
        let status = UiOutput::Status(Default::default());

        assert!(!should_forward_enqueue_output(&hint, false));
        assert!(should_forward_enqueue_output(&status, false));
        assert!(should_forward_enqueue_output(&hint, true));
    }
}

#[allow(clippy::too_many_arguments)]
async fn handle_session_event(
    event: SessionEvent,
    engine: &mut ConversationEngine,
    turn_state: &mut BridgeTurnState,
    deferred_accepted: &mut Option<DeferredAcceptedSubmission>,
    boundary_submission: &mut Option<BoundarySubmission>,
    sq_tx_watch: &tokio::sync::watch::Receiver<tokio::sync::mpsc::Sender<SessionOp>>,
    known_sender: &mut tokio::sync::mpsc::Sender<SessionOp>,
    sender_generation: &mut u64,
    ui_tx: &tokio::sync::mpsc::UnboundedSender<UiOutput>,
) {
    match event {
        SessionEvent::SubmissionReceipt {
            session_id,
            session_generation,
            submission_id,
            reservation_id,
            receipt_id,
            source,
            item_count,
            total_text_bytes,
            disposition,
        } => {
            handle_submission_receipt(
                engine,
                turn_state,
                boundary_submission,
                deferred_accepted,
                sq_tx_watch,
                ui_tx,
                session_id,
                session_generation,
                submission_id,
                reservation_id,
                receipt_id,
                source,
                item_count,
                total_text_bytes,
                disposition,
            );
        }
        SessionEvent::BackgroundJobTerminal { summary, .. } => {
            let _ = ui_tx.send(UiOutput::Content(ContentOutput::Block {
                source: MessageSource::System,
                text: format_background_terminal(&summary),
            }));
        }
        SessionEvent::StructuredSubmissionStarted {
            session_id,
            session_generation,
            submission,
            receipt_id,
            turn_id,
        } => {
            handle_structured_submission_started(
                engine,
                turn_state,
                ui_tx,
                session_id,
                session_generation,
                submission,
                receipt_id,
                turn_id,
            );
        }
        SessionEvent::StructuredTurnEvent {
            session_id,
            session_generation,
            source,
            submission_id,
            receipt_id,
            turn_id,
            sequence,
            payload,
        } => {
            let mut dispatch = QueuedDispatch::None;
            handle_structured_turn_event(
                engine,
                turn_state,
                deferred_accepted,
                sq_tx_watch,
                ui_tx,
                session_id,
                session_generation,
                source,
                submission_id,
                receipt_id,
                turn_id,
                sequence,
                payload,
                &mut dispatch,
                boundary_submission,
                *sender_generation,
            );
            if !matches!(dispatch, QueuedDispatch::None)
                && matches!(turn_state, BridgeTurnState::Idle)
            {
                dispatch_prepared_submission(
                    engine,
                    turn_state,
                    sq_tx_watch,
                    known_sender,
                    sender_generation,
                    ui_tx,
                    dispatch,
                )
                .await;
            }
        }
        SessionEvent::TurnEvent {
            session_id,
            turn_id,
            sequence,
            payload,
        } => {
            let dispatch = legacy_projection::handle_legacy_turn_event(
                engine,
                turn_state,
                ui_tx,
                session_id,
                turn_id,
                sequence,
                payload,
                *sender_generation,
            );
            if !matches!(dispatch, QueuedDispatch::None) {
                dispatch_prepared_submission(
                    engine,
                    turn_state,
                    sq_tx_watch,
                    known_sender,
                    sender_generation,
                    ui_tx,
                    dispatch,
                )
                .await;
            }
        }
        SessionEvent::SubmissionRejected {
            session_id,
            submission_id,
            sender_generation: rejected_generation,
            ..
        } => {
            let state = std::mem::replace(turn_state, BridgeTurnState::Idle);
            match state {
                BridgeTurnState::Submitting {
                    submission,
                    session_id: expected_session,
                    sender_generation: expected_generation,
                    command_sender: _,
                    cancel_requested: _,
                    last_reconcile: _,
                    reconcile_attempts: _,
                } if submission.id == submission_id
                    && expected_generation == rejected_generation
                    && expected_session
                        .as_ref()
                        .is_none_or(|expected| expected == &session_id) =>
                {
                    engine.rollback_prepared_steering(&submission_id);
                    *turn_state = BridgeTurnState::PausedAfterFailure;
                    emit_bridge_error(
                        ui_tx,
                        "session rejected the queued submission; input was retained",
                    );
                }
                other => {
                    *turn_state = other;
                }
            }
        }
        SessionEvent::SubmissionPaused {
            session_id,
            session_generation,
            submission_id,
            receipt_id,
            reason,
        } => {
            let state = std::mem::replace(turn_state, BridgeTurnState::Idle);
            match state {
                BridgeTurnState::AcceptedByActor {
                    session_id: expected_session,
                    session_generation: expected_generation,
                    submission_id: expected_submission,
                    receipt_id: expected_receipt,
                    ..
                } if expected_session == session_id
                    && expected_generation == session_generation
                    && expected_submission == submission_id
                    && expected_receipt == receipt_id =>
                {
                    *turn_state = BridgeTurnState::PreStartPaused {
                        session_id,
                        session_generation,
                        submission_id,
                        receipt_id,
                    };
                    emit_bridge_error(
                        ui_tx,
                        &format!(
                            "accepted submission paused before Provider start ({reason:?}); press cancel to terminalize it without execution"
                        ),
                    );
                }
                BridgeTurnState::AcceptedByActor {
                    session_id: expected_session,
                    session_generation: expected_generation,
                    submission_id: expected_submission,
                    receipt_id: expected_receipt,
                    submission,
                    projected,
                    cancel_requested,
                } if expected_session == session_id
                    && expected_generation == session_generation
                    && (expected_submission != submission_id || expected_receipt != receipt_id) =>
                {
                    if deferred_accepted.is_none() {
                        *deferred_accepted = Some(DeferredAcceptedSubmission {
                            session_id: expected_session,
                            session_generation: expected_generation,
                            submission_id: expected_submission.clone(),
                            receipt_id: expected_receipt,
                            submission,
                            projected,
                            cancel_requested,
                        });
                    }
                    *turn_state = BridgeTurnState::PreStartPaused {
                        session_id,
                        session_generation,
                        submission_id: submission_id.clone(),
                        receipt_id,
                    };
                    emit_bridge_error(
                        ui_tx,
                        &format!(
                            "older retained submission {submission_id} paused before {expected_submission}; press cancel to terminalize the paused identity and continue"
                        ),
                    );
                }
                other => {
                    *turn_state = other;
                }
            }
        }
        SessionEvent::SubmissionResolved {
            session_id,
            session_generation,
            submission_id,
            receipt_id,
            state: PendingSubmissionState::TerminalCancelled,
        } => {
            let state = std::mem::replace(turn_state, BridgeTurnState::Idle);
            let matches = matches!(
                &state,
                BridgeTurnState::PreStartPaused {
                    session_id: expected_session,
                    session_generation: expected_generation,
                    submission_id: expected_submission,
                    receipt_id: expected_receipt,
                } | BridgeTurnState::PreStartCancelling {
                    session_id: expected_session,
                    session_generation: expected_generation,
                    submission_id: expected_submission,
                    receipt_id: expected_receipt,
                } if expected_session == &session_id
                    && *expected_generation == session_generation
                    && expected_submission == &submission_id
                    && expected_receipt == &receipt_id
            );
            if !matches {
                *turn_state = state;
                emit_bridge_error(ui_tx, "ignored stale paused-submission resolution");
                return;
            }
            *turn_state = deferred_accepted.take().map_or(
                BridgeTurnState::Idle,
                DeferredAcceptedSubmission::into_turn_state,
            );
            send_bridge_stream(
                ui_tx,
                MessageSource::System,
                format!(
                    "[System] Cancelled paused submission {submission_id} before Provider execution.\n"
                ),
            );
            if matches!(turn_state, BridgeTurnState::Idle) {
                dispatch_prepared_submission(
                    engine,
                    turn_state,
                    sq_tx_watch,
                    known_sender,
                    sender_generation,
                    ui_tx,
                    QueuedDispatch::CurrentRoute,
                )
                .await;
            }
        }
        SessionEvent::SubmissionResolved { .. } => {
            emit_bridge_error(
                ui_tx,
                "ignored unsupported paused-submission terminal state",
            );
        }
        SessionEvent::SubmissionQueued { .. } | SessionEvent::SubmissionStarted { .. } => {
            // Compatibility projections only. Durable SubmissionReceipt,
            // StructuredSubmissionStarted and StructuredTurnEvent own transfer,
            // visible ordering and lifecycle authority.
        }
        SessionEvent::Error { message } => {
            emit_bridge_error(ui_tx, &message);
        }
        _ => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_submission_receipt(
    engine: &mut ConversationEngine,
    turn_state: &mut BridgeTurnState,
    boundary_submission: &mut Option<BoundarySubmission>,
    deferred_accepted: &mut Option<DeferredAcceptedSubmission>,
    _sq_tx_watch: &tokio::sync::watch::Receiver<tokio::sync::mpsc::Sender<SessionOp>>,
    ui_tx: &tokio::sync::mpsc::UnboundedSender<UiOutput>,
    session_id: String,
    session_generation: u64,
    submission_id: String,
    reservation_id: String,
    receipt_id: String,
    source: SubmissionSource,
    item_count: usize,
    total_text_bytes: usize,
    disposition: SubmissionReceiptDisposition,
) {
    if let Some(boundary) = boundary_submission.take() {
        let metadata_matches = boundary.submission.id == submission_id
            && reservation_id == format!("reservation:{}", boundary.submission.id)
            && boundary.submission.sender_generation == session_generation
            && boundary.sender_generation == session_generation
            && source == SubmissionSource::User
            && item_count == boundary.submission.items.len()
            && total_text_bytes == boundary.submission.total_text_bytes();
        if !metadata_matches {
            *boundary_submission = Some(boundary);
            emit_bridge_error(ui_tx, "ignored mismatched boundary submission receipt");
            return;
        }
        match disposition {
            SubmissionReceiptDisposition::AcceptedPending
            | SubmissionReceiptDisposition::AlreadyAccepted {
                state:
                    PendingSubmissionState::AcceptedPending | PendingSubmissionState::PausedPending,
                ..
            } => {
                if receipt_id.is_empty() || !engine.commit_prepared_steering(&submission_id) {
                    *boundary_submission = Some(boundary);
                    emit_bridge_error(
                        ui_tx,
                        "boundary receipt did not match the frozen Engine reservation",
                    );
                    return;
                }
                let _ = ui_tx.send(UiOutput::SteeringQueueSnapshot(
                    engine.steering_queue_snapshot(),
                ));
                let _ = ui_tx.send(UiOutput::Status(engine.status_snapshot()));
                *deferred_accepted = Some(DeferredAcceptedSubmission {
                    session_id,
                    session_generation,
                    submission_id,
                    receipt_id,
                    submission: boundary.submission,
                    projected: false,
                    cancel_requested: boundary.cancel_requested,
                });
            }
            SubmissionReceiptDisposition::NotAccepted
            | SubmissionReceiptDisposition::Rejected { .. } => {
                engine.rollback_prepared_steering(&submission_id);
                *boundary_submission = Some(boundary);
                emit_bridge_error(
                    ui_tx,
                    "boundary submission was not accepted; input was retained",
                );
            }
            SubmissionReceiptDisposition::AlreadyAccepted { state, .. } => {
                *boundary_submission = Some(boundary);
                emit_bridge_error(
                    ui_tx,
                    &format!("boundary submission custody recovered in terminal state {state:?}"),
                );
            }
        }
        return;
    }
    let state = std::mem::replace(turn_state, BridgeTurnState::Idle);
    let BridgeTurnState::Submitting {
        submission,
        session_id: expected_session,
        sender_generation,
        command_sender,
        cancel_requested,
        last_reconcile,
        reconcile_attempts,
    } = state
    else {
        *turn_state = state;
        return;
    };

    let metadata_matches = submission.id == submission_id
        && reservation_id == format!("reservation:{}", submission.id)
        && submission.sender_generation == session_generation
        && sender_generation == session_generation
        && source == submission.source
        && source == SubmissionSource::User
        && item_count == submission.items.len()
        && total_text_bytes == submission.total_text_bytes()
        && expected_session
            .as_ref()
            .is_none_or(|expected| expected == &session_id);
    if !metadata_matches {
        *turn_state = BridgeTurnState::Submitting {
            submission,
            session_id: expected_session,
            sender_generation,
            command_sender,
            cancel_requested,
            last_reconcile,
            reconcile_attempts,
        };
        emit_bridge_error(ui_tx, "ignored mismatched durable submission receipt");
        return;
    }

    match disposition {
        SubmissionReceiptDisposition::AcceptedPending
        | SubmissionReceiptDisposition::AlreadyAccepted {
            state: PendingSubmissionState::AcceptedPending | PendingSubmissionState::PausedPending,
            ..
        } => {
            if receipt_id.is_empty() {
                *turn_state = BridgeTurnState::Submitting {
                    submission,
                    session_id: Some(session_id),
                    sender_generation,
                    command_sender,
                    cancel_requested,
                    last_reconcile,
                    reconcile_attempts,
                };
                emit_bridge_error(
                    ui_tx,
                    "ignored an accepted submission receipt without identity",
                );
                return;
            }
            if !commit_actor_custody(engine, ui_tx, &submission) {
                *turn_state = BridgeTurnState::PausedAfterFailure;
                return;
            }
            *turn_state = BridgeTurnState::AcceptedByActor {
                session_id,
                session_generation,
                submission_id,
                receipt_id,
                submission,
                projected: false,
                cancel_requested,
            };
        }
        SubmissionReceiptDisposition::AlreadyAccepted { state, .. } => {
            if !commit_actor_custody(engine, ui_tx, &submission) {
                *turn_state = BridgeTurnState::PausedAfterFailure;
                return;
            }
            *turn_state = BridgeTurnState::PausedAfterFailure;
            emit_bridge_error(
                ui_tx,
                &format!(
                    "submission custody was recovered in terminal or running state {state:?}; reload the session before continuing"
                ),
            );
        }
        SubmissionReceiptDisposition::NotAccepted => {
            engine.rollback_prepared_steering(&submission_id);
            *turn_state = BridgeTurnState::PausedAfterFailure;
            emit_bridge_error(
                ui_tx,
                "the session authoritatively reported that the queued submission was not accepted; input was retained",
            );
        }
        SubmissionReceiptDisposition::Rejected { reason } => {
            engine.rollback_prepared_steering(&submission_id);
            *turn_state = BridgeTurnState::PausedAfterFailure;
            emit_bridge_error(
                ui_tx,
                &format!(
                    "the session rejected the queued submission ({reason:?}); input was retained"
                ),
            );
        }
    }
}

fn commit_actor_custody(
    engine: &mut ConversationEngine,
    ui_tx: &tokio::sync::mpsc::UnboundedSender<UiOutput>,
    submission: &StructuredSubmission,
) -> bool {
    if !engine.commit_prepared_steering(&submission.id) {
        emit_bridge_error(
            ui_tx,
            "durable receipt did not match the frozen Engine reservation",
        );
        return false;
    }
    let _ = ui_tx.send(UiOutput::SteeringQueueSnapshot(
        engine.steering_queue_snapshot(),
    ));
    let _ = ui_tx.send(UiOutput::Status(engine.status_snapshot()));
    true
}

fn emit_submission_projection(
    engine: &mut ConversationEngine,
    ui_tx: &tokio::sync::mpsc::UnboundedSender<UiOutput>,
    submission: &StructuredSubmission,
) {
    for item in &submission.items {
        for output in engine.start_user_message(&item.text) {
            let _ = ui_tx.send(output);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_structured_submission_started(
    engine: &mut ConversationEngine,
    turn_state: &mut BridgeTurnState,
    ui_tx: &tokio::sync::mpsc::UnboundedSender<UiOutput>,
    session_id: String,
    session_generation: u64,
    submission: StructuredSubmission,
    receipt_id: String,
    turn_id: String,
) {
    if submission.sender_generation != session_generation
        || submission.source == SubmissionSource::Compatibility
        || submission.validate().is_err()
        || receipt_id.is_empty()
        || turn_id.is_empty()
    {
        emit_bridge_error(ui_tx, "ignored invalid structured submission projection");
        return;
    }

    if submission.source == SubmissionSource::Scheduler
        && matches!(turn_state, BridgeTurnState::Idle)
    {
        emit_submission_projection(engine, ui_tx, &submission);
        return;
    }

    let BridgeTurnState::AcceptedByActor {
        session_id: expected_session,
        session_generation: expected_generation,
        submission_id: expected_submission,
        receipt_id: expected_receipt,
        submission: expected_payload,
        projected,
        ..
    } = turn_state
    else {
        emit_bridge_error(
            ui_tx,
            "ignored uncorrelated structured submission projection",
        );
        return;
    };

    if expected_session != &session_id || *expected_generation != session_generation {
        emit_bridge_error(ui_tx, "ignored stale structured submission projection");
        return;
    }

    let matches_expected = expected_submission == &submission.id && expected_receipt == &receipt_id;
    if matches_expected {
        if expected_payload != &submission {
            emit_bridge_error(
                ui_tx,
                "ignored conflicting structured submission projection",
            );
            return;
        }
        if !*projected {
            emit_submission_projection(engine, ui_tx, &submission);
            *projected = true;
        }
        return;
    }

    let retained_predecessor =
        expected_submission != &submission.id && expected_receipt != &receipt_id;
    if !retained_predecessor {
        emit_bridge_error(ui_tx, "ignored mismatched structured submission projection");
        return;
    }
    emit_submission_projection(engine, ui_tx, &submission);
}

#[allow(clippy::too_many_arguments)]
fn handle_structured_turn_event(
    engine: &mut ConversationEngine,
    turn_state: &mut BridgeTurnState,
    deferred_accepted: &mut Option<DeferredAcceptedSubmission>,
    sq_tx_watch: &tokio::sync::watch::Receiver<tokio::sync::mpsc::Sender<SessionOp>>,
    ui_tx: &tokio::sync::mpsc::UnboundedSender<UiOutput>,
    session_id: String,
    session_generation: u64,
    source: SubmissionSource,
    submission_id: String,
    receipt_id: String,
    turn_id: String,
    sequence: u64,
    payload: TurnEventPayload,
    queued_dispatch: &mut QueuedDispatch,
    boundary_submission: &mut Option<BoundarySubmission>,
    sender_generation: u64,
) {
    let state = std::mem::replace(turn_state, BridgeTurnState::Idle);
    match payload {
        TurnEventPayload::Started => match state {
            BridgeTurnState::Idle
                if source == SubmissionSource::Scheduler
                    && sequence == 0
                    && !submission_id.is_empty()
                    && !receipt_id.is_empty()
                    && !turn_id.is_empty() =>
            {
                for output in engine.handle_turn_started() {
                    let _ = ui_tx.send(output);
                }
                *turn_state = BridgeTurnState::StructuredRunning {
                    session_id,
                    session_generation,
                    submission_id,
                    receipt_id,
                    turn_id,
                    next_structured_sequence: 1,
                    next_legacy_sequence: 1,
                    progress_mode: ProgressMode::Unknown,
                };
            }
            BridgeTurnState::AcceptedByActor {
                session_id: expected_session,
                session_generation: expected_generation,
                submission_id: expected_submission,
                receipt_id: expected_receipt,
                submission,
                projected,
                cancel_requested,
            } if source == SubmissionSource::User
                && expected_session == session_id
                && expected_generation == session_generation
                && expected_submission == submission_id
                && expected_receipt == receipt_id
                && sequence == 0 =>
            {
                if !projected {
                    emit_submission_projection(engine, ui_tx, &submission);
                }
                for output in engine.handle_turn_started() {
                    let _ = ui_tx.send(output);
                }
                if cancel_requested {
                    if send_targeted_interrupt(sq_tx_watch, session_generation, &turn_id, ui_tx) {
                        *turn_state = BridgeTurnState::StructuredCancelling {
                            session_id,
                            session_generation,
                            submission_id,
                            receipt_id,
                            turn_id,
                            next_structured_sequence: 1,
                            next_legacy_sequence: 1,
                            progress_mode: ProgressMode::Unknown,
                        };
                    } else {
                        *turn_state = BridgeTurnState::StructuredRunning {
                            session_id,
                            session_generation,
                            submission_id,
                            receipt_id,
                            turn_id,
                            next_structured_sequence: 1,
                            next_legacy_sequence: 1,
                            progress_mode: ProgressMode::Unknown,
                        };
                    }
                } else {
                    *turn_state = BridgeTurnState::StructuredRunning {
                        session_id,
                        session_generation,
                        submission_id,
                        receipt_id,
                        turn_id,
                        next_structured_sequence: 1,
                        next_legacy_sequence: 1,
                        progress_mode: ProgressMode::Unknown,
                    };
                }
            }
            BridgeTurnState::AcceptedByActor {
                session_id: expected_session,
                session_generation: expected_generation,
                submission_id: expected_submission,
                receipt_id: expected_receipt,
                submission: expected_payload,
                projected,
                cancel_requested,
            } if source != SubmissionSource::Compatibility
                && expected_session == session_id
                && expected_generation == session_generation
                && sequence == 0
                && !receipt_id.is_empty()
                && (expected_submission != submission_id || expected_receipt != receipt_id) =>
            {
                if deferred_accepted.is_some() {
                    *turn_state = BridgeTurnState::AcceptedByActor {
                        session_id: expected_session,
                        session_generation: expected_generation,
                        submission_id: expected_submission,
                        receipt_id: expected_receipt,
                        submission: expected_payload,
                        projected,
                        cancel_requested,
                    };
                    emit_bridge_error(
                        ui_tx,
                        "cannot adopt more than one retained submission lifecycle at a time",
                    );
                    return;
                }
                *deferred_accepted = Some(DeferredAcceptedSubmission {
                    session_id: expected_session,
                    session_generation: expected_generation,
                    submission_id: expected_submission,
                    receipt_id: expected_receipt,
                    submission: expected_payload,
                    projected,
                    cancel_requested,
                });
                for output in engine.handle_turn_started() {
                    let _ = ui_tx.send(output);
                }
                *turn_state = BridgeTurnState::StructuredRunning {
                    session_id,
                    session_generation,
                    submission_id,
                    receipt_id,
                    turn_id,
                    next_structured_sequence: 1,
                    next_legacy_sequence: 1,
                    progress_mode: ProgressMode::Unknown,
                };
            }
            other => {
                *turn_state = other;
                emit_bridge_error(ui_tx, "ignored stale structured turn start");
            }
        },
        TurnEventPayload::Progress { event } => {
            let (matches, should_emit, next_state) = match state {
                BridgeTurnState::StructuredRunning {
                    session_id: expected_session,
                    session_generation: expected_generation,
                    submission_id: expected_submission,
                    receipt_id: expected_receipt,
                    turn_id: expected_turn,
                    mut next_structured_sequence,
                    next_legacy_sequence,
                    mut progress_mode,
                } => {
                    let matches = expected_session == session_id
                        && expected_generation == session_generation
                        && expected_submission == submission_id
                        && expected_receipt == receipt_id
                        && expected_turn == turn_id
                        && next_structured_sequence == sequence;
                    let should_emit = matches && progress_mode != ProgressMode::Legacy;
                    if matches {
                        next_structured_sequence = next_structured_sequence.saturating_add(1);
                        if progress_mode == ProgressMode::Unknown {
                            progress_mode = ProgressMode::Structured;
                        }
                    }
                    (
                        matches,
                        should_emit,
                        BridgeTurnState::StructuredRunning {
                            session_id: expected_session,
                            session_generation: expected_generation,
                            submission_id: expected_submission,
                            receipt_id: expected_receipt,
                            turn_id: expected_turn,
                            next_structured_sequence,
                            next_legacy_sequence,
                            progress_mode,
                        },
                    )
                }
                BridgeTurnState::StructuredCancelling {
                    session_id: expected_session,
                    session_generation: expected_generation,
                    submission_id: expected_submission,
                    receipt_id: expected_receipt,
                    turn_id: expected_turn,
                    mut next_structured_sequence,
                    next_legacy_sequence,
                    mut progress_mode,
                } => {
                    let matches = expected_session == session_id
                        && expected_generation == session_generation
                        && expected_submission == submission_id
                        && expected_receipt == receipt_id
                        && expected_turn == turn_id
                        && next_structured_sequence == sequence;
                    let should_emit = matches && progress_mode != ProgressMode::Legacy;
                    if matches {
                        next_structured_sequence = next_structured_sequence.saturating_add(1);
                        if progress_mode == ProgressMode::Unknown {
                            progress_mode = ProgressMode::Structured;
                        }
                    }
                    (
                        matches,
                        should_emit,
                        BridgeTurnState::StructuredCancelling {
                            session_id: expected_session,
                            session_generation: expected_generation,
                            submission_id: expected_submission,
                            receipt_id: expected_receipt,
                            turn_id: expected_turn,
                            next_structured_sequence,
                            next_legacy_sequence,
                            progress_mode,
                        },
                    )
                }
                other => (false, false, other),
            };
            *turn_state = next_state;
            if !matches {
                emit_bridge_error(ui_tx, "ignored stale or out-of-order structured progress");
            } else if should_emit && !matches!(event, AgentEvent::Error { .. }) {
                for output in engine.handle_agent_event(&event) {
                    let _ = ui_tx.send(output);
                }
                if matches!(
                    event,
                    AgentEvent::TurnEnd {
                        stop_reason: talos_core::message::StopReason::ToolUse,
                        ..
                    }
                ) {
                    prepare_boundary_submission(
                        engine,
                        boundary_submission,
                        deferred_accepted,
                        sq_tx_watch,
                        sender_generation,
                        ui_tx,
                    );
                }
            }
        }
        TurnEventPayload::Completed { status } => {
            let cancellation_requested =
                matches!(&state, BridgeTurnState::StructuredCancelling { .. });
            let matching = match &state {
                BridgeTurnState::StructuredRunning {
                    session_id: expected_session,
                    session_generation: expected_generation,
                    submission_id: expected_submission,
                    receipt_id: expected_receipt,
                    turn_id: expected_turn,
                    next_structured_sequence,
                    ..
                }
                | BridgeTurnState::StructuredCancelling {
                    session_id: expected_session,
                    session_generation: expected_generation,
                    submission_id: expected_submission,
                    receipt_id: expected_receipt,
                    turn_id: expected_turn,
                    next_structured_sequence,
                    ..
                } => {
                    expected_session == &session_id
                        && *expected_generation == session_generation
                        && expected_submission == &submission_id
                        && expected_receipt == &receipt_id
                        && expected_turn == &turn_id
                        && *next_structured_sequence == sequence
                }
                _ => false,
            };
            if !matching {
                *turn_state = state;
                emit_bridge_error(ui_tx, "ignored stale or out-of-order structured completion");
                return;
            }
            for output in engine.handle_turn_completed(&status) {
                let _ = ui_tx.send(output);
            }
            let continuation_allowed =
                completion_allows_queued_continuation(&status, cancellation_requested);
            *turn_state = if continuation_allowed {
                deferred_accepted.take().map_or(
                    BridgeTurnState::Idle,
                    DeferredAcceptedSubmission::into_turn_state,
                )
            } else {
                deferred_accepted.take();
                BridgeTurnState::PausedAfterFailure
            };
            let _ = ui_tx.send(UiOutput::SteeringQueueSnapshot(
                engine.steering_queue_snapshot(),
            ));
            if continuation_allowed {
                *queued_dispatch = if cancellation_requested {
                    QueuedDispatch::Generation(session_generation)
                } else {
                    QueuedDispatch::CurrentRoute
                };
            }
        }
        _ => {
            *turn_state = state;
        }
    }
}

#[cfg(test)]
mod completion_continuation_tests {
    use super::*;

    #[test]
    fn only_success_or_requested_cancellation_allows_queued_continuation() {
        let success = TurnCompletionStatus::Success {
            final_text: String::new(),
            new_messages: Vec::new(),
        };
        assert!(completion_allows_queued_continuation(&success, false));
        assert!(completion_allows_queued_continuation(
            &TurnCompletionStatus::Cancelled,
            true
        ));
        assert!(!completion_allows_queued_continuation(
            &TurnCompletionStatus::Cancelled,
            false
        ));
        assert!(!completion_allows_queued_continuation(
            &TurnCompletionStatus::Error {
                message: "provider failed".into(),
            },
            true
        ));
    }

    #[tokio::test]
    async fn generation_change_after_esc_retains_queue_without_cross_generation_submit() {
        let (generation_eight, mut generation_eight_rx) = tokio::sync::mpsc::channel(4);
        crate::session_transition::register_generation_bound_sender(&generation_eight, 8);
        let (_watch_tx, watch_rx) = tokio::sync::watch::channel(generation_eight.clone());
        let (ui_tx, mut ui_rx) = tokio::sync::mpsc::unbounded_channel();
        let mut engine = ConversationEngine::new("model".into(), "provider".into());
        engine.enqueue_steering("generation-seven-queued".into());
        let mut turn_state = BridgeTurnState::Idle;
        let mut known_sender = generation_eight;
        let mut sender_generation = 8;

        dispatch_prepared_submission(
            &mut engine,
            &mut turn_state,
            &watch_rx,
            &mut known_sender,
            &mut sender_generation,
            &ui_tx,
            QueuedDispatch::Generation(7),
        )
        .await;

        assert!(matches!(turn_state, BridgeTurnState::PausedAfterFailure));
        assert_eq!(engine.steering_queue_snapshot().total_count, 1);
        assert!(generation_eight_rx.try_recv().is_err());
        assert!(matches!(
            ui_rx.try_recv(),
            Ok(UiOutput::Content(ContentOutput::Block {
                source: MessageSource::Error,
                text,
            })) if text.contains("session generation changed after cancellation")
        ));
    }
}

#[cfg(test)]
mod boundary_submission_tests {
    use super::*;

    #[test]
    fn model_tool_boundary_transfers_one_fifo_batch_without_replacing_active_turn() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(2);
        crate::session_transition::register_generation_bound_sender(&tx, 7);
        let (_watch_tx, watch_rx) = tokio::sync::watch::channel(tx);
        let (ui_tx, _ui_rx) = tokio::sync::mpsc::unbounded_channel();
        let mut engine = ConversationEngine::new("model".into(), "provider".into());
        engine.enqueue_steering("first".into());
        let mut boundary = None;
        let mut deferred = None;
        prepare_boundary_submission(
            &mut engine,
            &mut boundary,
            &mut deferred,
            &watch_rx,
            7,
            &ui_tx,
        );
        assert!(boundary.is_some());
        assert!(deferred.is_none());
        assert_eq!(engine.steering_queue_snapshot().total_count, 1);
        assert!(matches!(
            rx.try_recv(),
            Ok(SessionOp::SubmitStructured { .. })
        ));
    }
}

fn retry_submission_receipt(
    turn_state: &mut BridgeTurnState,
    boundary_submission: &mut Option<BoundarySubmission>,
    ui_tx: &tokio::sync::mpsc::UnboundedSender<UiOutput>,
) {
    if let Some(boundary) = boundary_submission.as_mut()
        && boundary.last_reconcile.elapsed() >= RECEIPT_RECONCILE_AFTER
    {
        boundary.last_reconcile = Instant::now();
        boundary.reconcile_attempts = boundary.reconcile_attempts.saturating_add(1);
        if boundary
            .command_sender
            .try_send(SessionOp::ReconcileStructured {
                submission: boundary.submission.clone(),
            })
            .is_err()
            && boundary.reconcile_attempts == RECEIPT_WARNING_ATTEMPT
        {
            emit_bridge_error(
                ui_tx,
                "boundary submission receipt reconciliation is waiting for its original session route",
            );
        }
        return;
    }
    let (submission, command_sender, attempts) = match turn_state {
        BridgeTurnState::Submitting {
            submission,
            command_sender,
            last_reconcile,
            reconcile_attempts,
            ..
        } if last_reconcile.elapsed() >= RECEIPT_RECONCILE_AFTER => {
            *last_reconcile = Instant::now();
            *reconcile_attempts = reconcile_attempts.saturating_add(1);
            (
                Some(submission.clone()),
                Some(command_sender.clone()),
                *reconcile_attempts,
            )
        }
        _ => (None, None, 0),
    };
    let (Some(submission), Some(command_sender)) = (submission, command_sender) else {
        return;
    };
    if command_sender
        .try_send(SessionOp::ReconcileStructured { submission })
        .is_err()
        && attempts == RECEIPT_WARNING_ATTEMPT
    {
        emit_bridge_error(
            ui_tx,
            "submission receipt reconciliation is waiting for its original generation-bound command route; Engine escrow remains frozen",
        );
    } else if attempts == RECEIPT_WARNING_ATTEMPT {
        emit_bridge_error(
            ui_tx,
            "submission receipt was delayed; reconciliation is continuing on the original command route with the same immutable identity",
        );
    }
}

fn request_cancel(
    turn_state: &mut BridgeTurnState,
    sq_tx_watch: &tokio::sync::watch::Receiver<tokio::sync::mpsc::Sender<SessionOp>>,
    ui_tx: &tokio::sync::mpsc::UnboundedSender<UiOutput>,
) {
    match turn_state {
        BridgeTurnState::Submitting {
            cancel_requested, ..
        }
        | BridgeTurnState::AcceptedByActor {
            cancel_requested, ..
        } => {
            // No Turn identity exists yet. Remember intent and send the exact
            // targeted interrupt only after a matching Structured Started event.
            *cancel_requested = true;
        }
        BridgeTurnState::PreStartPaused {
            session_id,
            session_generation,
            submission_id,
            receipt_id,
        } => {
            if !send_paused_submission_cancel(
                sq_tx_watch,
                *session_generation,
                submission_id,
                ui_tx,
            ) {
                return;
            }
            *turn_state = BridgeTurnState::PreStartCancelling {
                session_id: session_id.clone(),
                session_generation: *session_generation,
                submission_id: submission_id.clone(),
                receipt_id: receipt_id.clone(),
            };
        }
        BridgeTurnState::StructuredRunning {
            session_id,
            session_generation,
            submission_id,
            receipt_id,
            turn_id,
            next_structured_sequence,
            next_legacy_sequence,
            progress_mode,
        } => {
            if !send_targeted_interrupt(sq_tx_watch, *session_generation, turn_id, ui_tx) {
                return;
            }
            *turn_state = BridgeTurnState::StructuredCancelling {
                session_id: session_id.clone(),
                session_generation: *session_generation,
                submission_id: submission_id.clone(),
                receipt_id: receipt_id.clone(),
                turn_id: turn_id.clone(),
                next_structured_sequence: *next_structured_sequence,
                next_legacy_sequence: *next_legacy_sequence,
                progress_mode: *progress_mode,
            };
        }
        BridgeTurnState::LegacyRunning {
            session_id,
            turn_id,
            next_sequence,
            sender_generation,
        } => {
            if !send_legacy_interrupt(sq_tx_watch, ui_tx) {
                return;
            }
            *turn_state = BridgeTurnState::LegacyCancelling {
                session_id: session_id.clone(),
                turn_id: turn_id.clone(),
                next_sequence: *next_sequence,
                sender_generation: *sender_generation,
            };
        }
        _ => {}
    }
}

fn send_paused_submission_cancel(
    sq_tx_watch: &tokio::sync::watch::Receiver<tokio::sync::mpsc::Sender<SessionOp>>,
    session_generation: u64,
    submission_id: &str,
    ui_tx: &tokio::sync::mpsc::UnboundedSender<UiOutput>,
) -> bool {
    match sq_tx_watch
        .borrow()
        .clone()
        .try_send(SessionOp::CancelPausedSubmission {
            session_generation,
            submission_id: submission_id.to_owned(),
        }) {
        Ok(()) => true,
        Err(_) => {
            emit_bridge_error(
                ui_tx,
                "session command channel is busy; paused submission was not resolved",
            );
            false
        }
    }
}

fn send_targeted_interrupt(
    sq_tx_watch: &tokio::sync::watch::Receiver<tokio::sync::mpsc::Sender<SessionOp>>,
    session_generation: u64,
    turn_id: &str,
    ui_tx: &tokio::sync::mpsc::UnboundedSender<UiOutput>,
) -> bool {
    match sq_tx_watch
        .borrow()
        .clone()
        .try_send(SessionOp::InterruptTurn {
            session_generation,
            turn_id: turn_id.to_owned(),
        }) {
        Ok(()) => true,
        Err(_) => {
            emit_bridge_error(
                ui_tx,
                "session command channel is busy; targeted cancel was not accepted",
            );
            false
        }
    }
}

fn send_legacy_interrupt(
    sq_tx_watch: &tokio::sync::watch::Receiver<tokio::sync::mpsc::Sender<SessionOp>>,
    ui_tx: &tokio::sync::mpsc::UnboundedSender<UiOutput>,
) -> bool {
    match sq_tx_watch.borrow().clone().try_send(SessionOp::Interrupt) {
        Ok(()) => true,
        Err(_) => {
            emit_bridge_error(
                ui_tx,
                "session command channel is busy; cancel was not accepted",
            );
            false
        }
    }
}

async fn dispatch_prepared_submission(
    engine: &mut ConversationEngine,
    turn_state: &mut BridgeTurnState,
    sq_tx_watch: &tokio::sync::watch::Receiver<tokio::sync::mpsc::Sender<SessionOp>>,
    known_sender: &mut tokio::sync::mpsc::Sender<SessionOp>,
    sender_generation: &mut u64,
    ui_tx: &tokio::sync::mpsc::UnboundedSender<UiOutput>,
    dispatch: QueuedDispatch,
) {
    let current_sender = sq_tx_watch.borrow().clone();
    if !known_sender.same_channel(&current_sender) {
        let Some(current_generation) = authoritative_generation_for_sender(&current_sender) else {
            emit_bridge_error(
                ui_tx,
                "cannot submit through a command Sender without an authoritative generation",
            );
            return;
        };
        *sender_generation = current_generation;
        *known_sender = current_sender.clone();
    }
    if let QueuedDispatch::Generation(expected_generation) = dispatch
        && *sender_generation != expected_generation
    {
        *turn_state = BridgeTurnState::PausedAfterFailure;
        emit_bridge_error(
            ui_tx,
            "session generation changed after cancellation; queued input was retained",
        );
        return;
    }
    let Some(mut submission) = engine.prepare_steering_submission() else {
        return;
    };
    submission.sender_generation = *sender_generation;
    let submission_id = submission.id.clone();
    *turn_state = BridgeTurnState::Submitting {
        submission: submission.clone(),
        session_id: None,
        sender_generation: *sender_generation,
        command_sender: current_sender.clone(),
        cancel_requested: false,
        last_reconcile: Instant::now(),
        reconcile_attempts: 0,
    };
    let reserve = tokio::time::timeout(
        Duration::from_secs(1),
        current_sender.clone().reserve_owned(),
    )
    .await;
    let sent = match reserve {
        Ok(Ok(permit))
            if current_sender.same_channel(&sq_tx_watch.borrow())
                && authoritative_generation_for_sender(&current_sender)
                    .is_none_or(|generation| generation == *sender_generation) =>
        {
            permit.send(SessionOp::SubmitStructured { submission });
            true
        }
        _ => false,
    };
    if !sent {
        engine.rollback_prepared_steering(&submission_id);
        *turn_state = BridgeTurnState::PausedAfterFailure;
        emit_bridge_error(
            ui_tx,
            "session command channel unavailable; queued input was retained",
        );
    }
}

/// Transfers one queued steering batch at the provider's model-response
/// boundary. The actor keeps the current turn running and queues the accepted
/// batch durably; execution still begins only at the actor's next turn.
fn prepare_boundary_submission(
    engine: &mut ConversationEngine,
    boundary_submission: &mut Option<BoundarySubmission>,
    deferred_accepted: &mut Option<DeferredAcceptedSubmission>,
    sq_tx_watch: &tokio::sync::watch::Receiver<tokio::sync::mpsc::Sender<SessionOp>>,
    sender_generation: u64,
    ui_tx: &tokio::sync::mpsc::UnboundedSender<UiOutput>,
) {
    if boundary_submission.is_some() || deferred_accepted.is_some() {
        return;
    }
    let Some(mut submission) = engine.prepare_steering_submission() else {
        return;
    };
    submission.sender_generation = sender_generation;
    let submission_id = submission.id.clone();
    let command_sender = sq_tx_watch.borrow().clone();
    if authoritative_generation_for_sender(&command_sender) != Some(sender_generation) {
        engine.rollback_prepared_steering(&submission_id);
        emit_bridge_error(
            ui_tx,
            "session generation changed at model boundary; queued input was retained",
        );
        return;
    }
    match command_sender.try_send(SessionOp::SubmitStructured {
        submission: submission.clone(),
    }) {
        Ok(()) => {
            *boundary_submission = Some(BoundarySubmission {
                submission,
                sender_generation,
                command_sender,
                cancel_requested: false,
                last_reconcile: Instant::now(),
                reconcile_attempts: 0,
            });
        }
        Err(_) => {
            engine.rollback_prepared_steering(&submission_id);
            emit_bridge_error(
                ui_tx,
                "session command channel unavailable at model boundary; queued input was retained",
            );
        }
    }
}

pub(super) fn emit_bridge_error(
    ui_tx: &tokio::sync::mpsc::UnboundedSender<UiOutput>,
    message: &str,
) {
    let _ = ui_tx.send(UiOutput::Content(ContentOutput::Block {
        source: MessageSource::Error,
        text: format!("[Error] {message}.\n"),
    }));
}

/// Produces the user-visible, path-safe summary for a pending image attachment.
///
/// The basename is deliberately fenced so the scrollback Markdown renderer does
/// not interpret underscores or other filename characters as formatting.
fn attachment_summary(path: &std::path::Path, mime: &str, byte_count: u64) -> String {
    format!(
        "```{}``` ({} bytes, {})",
        path.file_name().unwrap_or_default().to_string_lossy(),
        byte_count,
        mime
    )
}

fn propose_attach_grants(
    ui_tx: &tokio::sync::mpsc::UnboundedSender<UiOutput>,
    state: &talos_permission::PermissionSessionState,
    request: &talos_permission::PermissionRequest<'_>,
    context: &talos_permission::PermissionContext,
) -> Option<(
    talos_permission::ProposedGrant,
    talos_permission::ProposedGrant,
)> {
    let once = match state.propose(request, context, talos_permission::GrantScope::Once) {
        Ok(proposal) => proposal,
        Err(error) => {
            let _ = ui_tx.send(UiOutput::Content(ContentOutput::Block {
                source: MessageSource::Error,
                text: format!(
                    "[Error] /attach permission proposal failed: {error}. No file was read.\n"
                ),
            }));
            return None;
        }
    };
    let session = match state.propose(request, context, talos_permission::GrantScope::Session) {
        Ok(proposal) => proposal,
        Err(error) => {
            let _ = ui_tx.send(UiOutput::Content(ContentOutput::Block {
                source: MessageSource::Error,
                text: format!(
                    "[Error] /attach permission proposal failed: {error}. No file was read.\n"
                ),
            }));
            return None;
        }
    };
    Some((once, session))
}

/// P1-A: evaluates an image attachment path against the SEC-001
/// permission pipeline before any filesystem probe. Returns the
/// authorized canonical PathBuf on success, or None when rejected
/// (Deny, no engine available, or denied approval). The caller MUST
/// pass the returned canonical path to create_image_content_part —
/// NOT the original user-supplied path — so that symlink drift
/// between authorization and ingestion is impossible.
async fn authorize_attach_image(
    ui_tx: &tokio::sync::mpsc::UnboundedSender<UiOutput>,
    permission_engine: &Option<Arc<talos_permission::PermissionSessionState>>,
    path: &str,
) -> Option<std::path::PathBuf> {
    use crate::image_authorization::{ATTACH_IMAGE_TOOL_NAME, ImageAuthorization};
    use talos_core::ApprovalChoice;
    use talos_core::tool::{ToolNature, ToolPermissionFacet, ToolProvenance, ToolResourceKind};
    use talos_permission::{
        GrantSource, InteractionCapability, PermissionContext, PermissionMode, PermissionRequest,
    };

    let Some(engine_ref) = permission_engine else {
        let _ = ui_tx.send(UiOutput::Content(ContentOutput::Block {
            source: MessageSource::Error,
            text: format!(
                "[Error] /attach {path} refused: no permission engine available (fail-closed). No file was read.\n"
            ),
        }));
        return None;
    };

    let canonical = match std::path::Path::new(path).canonicalize() {
        Ok(c) => c,
        Err(e) => {
            let _ = ui_tx.send(UiOutput::Content(ContentOutput::Block {
                source: MessageSource::Error,
                text: format!(
                    "[Error] /attach {path} failed to canonicalize: {e}. No file was read.\n"
                ),
            }));
            return None;
        }
    };
    let Some(canonical_str) = canonical.to_str() else {
        let _ = ui_tx.send(UiOutput::Content(ContentOutput::Block {
            source: MessageSource::Error,
            text: format!(
                "[Error] /attach {path} has a non-UTF-8 canonical path that cannot be represented safely. No file was read.\n"
            ),
        }));
        return None;
    };

    let context = PermissionContext::new(
        PermissionMode::Interactive,
        InteractionCapability::Available,
    );
    let input = serde_json::json!({ "path": canonical_str });
    let facets = [ToolPermissionFacet::with_resource(
        ToolNature::Read,
        canonical_str,
        ToolResourceKind::Path,
    )];
    let request = PermissionRequest::new(
        ATTACH_IMAGE_TOOL_NAME,
        ToolProvenance::Native,
        &facets,
        &input,
    );
    let decision = match ImageAuthorization::evaluate(&canonical, engine_ref, &context) {
        Ok(decision) => decision,
        Err(error) => {
            let _ = ui_tx.send(UiOutput::Content(ContentOutput::Block {
                source: MessageSource::Error,
                text: format!(
                    "[Error] /attach permission evaluation failed: {error}. No file was read.\n"
                ),
            }));
            return None;
        }
    };

    match decision {
        ImageAuthorization::Allow => {
            let pending = match engine_ref.prepare_authorized(&request, &context) {
                Ok(Some(pending)) => pending,
                Ok(None) => {
                    let _ = ui_tx.send(UiOutput::Content(ContentOutput::Block {
                        source: MessageSource::Error,
                        text:
                            "[Error] /attach authorization is no longer valid. No file was read.\n"
                                .to_string(),
                    }));
                    return None;
                }
                Err(error) => {
                    let _ = ui_tx.send(UiOutput::Content(ContentOutput::Block {
                        source: MessageSource::Error,
                        text: format!(
                            "[Error] /attach authorization failed: {error}. No file was read.\n"
                        ),
                    }));
                    return None;
                }
            };
            match engine_ref.admit(pending, &request, &context) {
                Ok(_) => Some(canonical),
                Err(error) => {
                    let _ = ui_tx.send(UiOutput::Content(ContentOutput::Block {
                        source: MessageSource::Error,
                        text: format!("[Error] /attach authorization became stale: {error}. No file was read.\n"),
                    }));
                    None
                }
            }
        }
        ImageAuthorization::Deny(reason) => {
            let _ = ui_tx.send(UiOutput::Content(ContentOutput::Block {
                source: MessageSource::Error,
                text: format!(
                    "[Error] /attach {path} (canonical: {canonical_str}) denied by permission rule: {reason}. No file was read.\n"
                ),
            }));
            None
        }
        ImageAuthorization::Ask => {
            let (once, session) = propose_attach_grants(ui_tx, engine_ref, &request, &context)?;
            let (response_tx, response_rx) =
                tokio::sync::oneshot::channel::<talos_core::ApprovalChoice>();
            let summary_fields = vec!["path".to_string()];
            if ui_tx
                .send(UiOutput::ToolApprovalRequest {
                    tool_name: ATTACH_IMAGE_TOOL_NAME.to_string(),
                    arguments: serde_json::json!({ "path": canonical_str }),
                    summary_fields,
                    preview: Some(crate::approval::format_grant_preview(session.preview())),
                    response: response_tx,
                })
                .is_err()
            {
                return None;
            }
            match response_rx.await {
                Ok(ApprovalChoice::Deny) => {
                    let _ = ui_tx.send(UiOutput::Content(ContentOutput::Block {
                        source: MessageSource::Error,
                        text: format!(
                            "[Error] /attach {path} (canonical: {canonical_str}) denied by user. No file was read.\n"
                        ),
                    }));
                    None
                }
                Ok(choice @ (ApprovalChoice::AlwaysApprove | ApprovalChoice::ApproveOnce)) => {
                    let result = match choice {
                        ApprovalChoice::AlwaysApprove => engine_ref.approve_session(
                            session,
                            &request,
                            &context,
                            GrantSource::InteractiveHuman,
                        ),
                        ApprovalChoice::ApproveOnce => {
                            engine_ref.approve_once(once, &request, &context)
                        }
                        ApprovalChoice::Deny => unreachable!(),
                    }
                    .and_then(|pending| engine_ref.admit(pending, &request, &context));
                    match result {
                        Ok(_) => Some(canonical),
                        Err(error) => {
                            let _ = ui_tx.send(UiOutput::Content(ContentOutput::Block {
                                source: MessageSource::Error,
                                text: format!("[Error] /attach authorization became stale: {error}. No file was read.\n"),
                            }));
                            None
                        }
                    }
                }
                Err(_) => {
                    let _ = ui_tx.send(UiOutput::Content(ContentOutput::Block {
                        source: MessageSource::Error,
                        text: format!(
                            "[Error] /attach {path} approval channel closed. No file was read.\n"
                        ),
                    }));
                    None
                }
            }
        }
    }
}

async fn handle_skill_command(
    req: SkillCommandRequest,
    engine: &mut ConversationEngine,
    ui_tx: &tokio::sync::mpsc::UnboundedSender<UiOutput>,
    sq_tx_watch: &tokio::sync::watch::Receiver<
        tokio::sync::mpsc::Sender<talos_core::session::SessionOp>,
    >,
    runtime_skills: Arc<Mutex<RuntimeSkills>>,
) {
    let mut skills = runtime_skills.lock().await;
    let result = match req {
        SkillCommandRequest::Activate { name } => {
            let trimmed = name.trim().to_string();
            skills
                .activate(&trimmed)
                .map(|content| (Some(trimmed), Some(content), "activated"))
        }
        SkillCommandRequest::Reference { path } => {
            let active = skills.active_name().map(str::to_string);
            skills
                .load_reference(path.trim())
                .map(|content| (active, Some(content), "loaded reference"))
        }
    };

    match result {
        Ok((name, content, action)) => {
            let sq_tx = sq_tx_watch.borrow().clone();
            let _ = sq_tx
                .send(talos_core::session::SessionOp::SetSkillContext {
                    name: name.clone(),
                    content,
                })
                .await;
            engine.set_skills(skills.diagnostics());
            let label = name.unwrap_or_else(|| "active skill".to_string());
            send_bridge_stream(
                ui_tx,
                MessageSource::System,
                format!(
                    "[System] Skill {action}: {label}. Content added to provider context only.\n"
                ),
            );
        }
        Err(error) => {
            send_bridge_stream(ui_tx, MessageSource::Error, format!("[Error] {error}\n"));
        }
    }
}

fn send_bridge_stream(
    ui_tx: &tokio::sync::mpsc::UnboundedSender<UiOutput>,
    source: MessageSource,
    text: String,
) {
    let _ = ui_tx.send(UiOutput::Content(ContentOutput::Block { source, text }));
}

/// Session lifecycle request forwarded from the conversation loop to the mode runner.
pub(crate) enum SessionLifecycleRequest {
    New(SessionNewRequest),
    Resume(SessionResumeRequest),
    Fork(SessionForkRequest),
    Delete(SessionDeleteRequest),
    Todo(TodoCommandRequest),
    ModelSwitch(ModelSwitchRequest),
    ModelSwitchWithCredential(CredentialResponseData),
    ProviderSetup(String),
    ConnectRequest {
        provider: String,
    },
    ConnectWithCredential(CredentialResponseData),
    RegisterCustomProvider {
        name: String,
        protocol: String,
        base_url: String,
        api_key: String,
    },
}

#[cfg(test)]
mod background_terminal_projection_tests {
    use super::*;
    use talos_core::background_job::{
        BackgroundCleanupOutcome, BackgroundJobId, BackgroundJobState, BackgroundJobTerminalSummary,
    };

    fn terminal_summary(job_id: &str, state: BackgroundJobState) -> BackgroundJobTerminalSummary {
        BackgroundJobTerminalSummary {
            job_id: BackgroundJobId::new(job_id),
            tool_name: "bash".to_string(),
            state,
            exit_code: matches!(state, BackgroundJobState::Completed).then_some(0),
            stdout_bytes: 12,
            stderr_bytes: 3,
            earliest_cursor: 1,
            next_cursor: 4,
            truncated: false,
            started_at_unix_ms: 1,
            finished_at_unix_ms: 2,
            cleanup_outcome: BackgroundCleanupOutcome::Natural,
            cleanup_error: None,
        }
    }

    #[tokio::test]
    async fn real_session_terminal_events_project_exactly_once_for_each_lifecycle_path() {
        let cases = [
            ("natural-exit", BackgroundJobState::Completed),
            ("timeout", BackgroundJobState::TimedOut),
            ("cancel", BackgroundJobState::Cancelled),
            ("shutdown", BackgroundJobState::Cancelled),
            ("failure", BackgroundJobState::Failed),
        ];

        for (job_id, state) in cases {
            let mut engine = ConversationEngine::new("test-model".into(), "test-provider".into());
            let mut turn_state = BridgeTurnState::Idle;
            let mut deferred_accepted = None;
            let mut boundary_submission = None;
            let (session_tx, _session_rx) = tokio::sync::mpsc::channel(1);
            let (_watch_tx, watch_rx) = tokio::sync::watch::channel(session_tx.clone());
            let mut known_sender = session_tx;
            let mut sender_generation = 0;
            let (ui_tx, mut ui_rx) = tokio::sync::mpsc::unbounded_channel();

            handle_session_event(
                SessionEvent::BackgroundJobTerminal {
                    session_id: "session-1".into(),
                    session_generation: 1,
                    summary: terminal_summary(job_id, state),
                },
                &mut engine,
                &mut turn_state,
                &mut deferred_accepted,
                &mut boundary_submission,
                &watch_rx,
                &mut known_sender,
                &mut sender_generation,
                &ui_tx,
            )
            .await;

            let output = ui_rx.try_recv().expect("one terminal projection");
            let UiOutput::Content(ContentOutput::Block {
                source: MessageSource::System,
                text,
            }) = output
            else {
                panic!("expected a system terminal block");
            };
            assert!(text.contains(&format!("[background {job_id}] terminal:")));
            assert!(text.contains(&format!("{state:?}")));
            assert!(
                ui_rx.try_recv().is_err(),
                "terminal event projected more than once"
            );
        }
    }
}

#[cfg(test)]
mod attachment_authorization_tests {
    use super::*;
    #[cfg(unix)]
    use talos_core::ApprovalChoice;

    #[cfg(unix)]
    fn write_png(path: &std::path::Path, width: u32) {
        image::RgbaImage::new(width, 1)
            .save_with_format(path, image::ImageFormat::Png)
            .expect("operation should succeed");
    }

    #[test]
    fn attachment_summary_fences_filename_without_mutating_it() {
        assert_eq!(
            attachment_summary(
                std::path::Path::new("/tmp/ScreenShot_2026-07-22_151225_812.png"),
                "image/png",
                246_943,
            ),
            "```ScreenShot_2026-07-22_151225_812.png``` (246943 bytes, image/png)"
        );
    }

    #[test]
    fn attachment_proposal_failure_is_visible_and_fail_closed() {
        use talos_core::tool::{ToolNature, ToolPermissionFacet, ToolProvenance, ToolResourceKind};
        use talos_permission::{
            InteractionCapability, PermissionContext, PermissionMode, PermissionRequest,
        };

        let state = talos_permission::PermissionSessionState::new(
            talos_permission::PermissionEngine::new(),
        );
        let input = serde_json::json!({ "path": "/private/tmp/not-read.png" });
        let facets = [ToolPermissionFacet::with_resource(
            ToolNature::Read,
            "/private/tmp/not-read.png",
            ToolResourceKind::Path,
        )];
        let request = PermissionRequest::new(
            crate::image_authorization::ATTACH_IMAGE_TOOL_NAME,
            ToolProvenance::Native,
            &facets,
            &input,
        );
        let context = PermissionContext::new(
            PermissionMode::Interactive,
            InteractionCapability::Available,
        );
        let (ui_tx, mut ui_rx) = tokio::sync::mpsc::unbounded_channel();

        assert!(propose_attach_grants(&ui_tx, &state, &request, &context).is_none());
        let output = ui_rx.try_recv().expect("visible proposal failure");
        let UiOutput::Content(ContentOutput::Block {
            source: MessageSource::Error,
            text,
        }) = output
        else {
            panic!("expected visible attachment error");
        };
        assert!(text.contains("permission proposal failed"));
        assert!(text.contains("No file was read"));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn approved_symlink_drift_cannot_redirect_attachment_ingestion() {
        let workspace = tempfile::tempdir().expect("operation should succeed");
        let external = tempfile::tempdir().expect("operation should succeed");
        let approved = external.path().join("approved.png");
        let replacement = external.path().join("replacement.png");
        write_png(&approved, 2);
        write_png(&replacement, 9);

        let link = workspace.path().join("selected.png");
        std::os::unix::fs::symlink(&approved, &link).expect("operation should succeed");
        let approved_canonical = approved.canonicalize().expect("operation should succeed");

        let permission_engine = Some(Arc::new(talos_permission::PermissionSessionState::new(
            talos_permission::PermissionEngine::with_workspace_root(workspace.path().to_path_buf()),
        )));
        let (ui_tx, mut ui_rx) = tokio::sync::mpsc::unbounded_channel();
        let mut authorization = Box::pin(authorize_attach_image(
            &ui_tx,
            &permission_engine,
            link.to_str().expect("operation should succeed"),
        ));

        let approval = tokio::select! {
            output = ui_rx.recv() => output.expect("authorization output"),
            _ = &mut authorization => panic!("external attachment must request approval"),
        };
        let UiOutput::ToolApprovalRequest { response, .. } = approval else {
            panic!("expected attachment approval request");
        };
        response
            .send(ApprovalChoice::ApproveOnce)
            .expect("operation should succeed");
        let canonical = authorization.await.expect("approved canonical path");
        assert_eq!(canonical, approved_canonical);

        std::fs::remove_file(&link).expect("operation should succeed");
        std::os::unix::fs::symlink(&replacement, &link).expect("operation should succeed");

        let part = crate::image_validation::create_image_content_part(&canonical, 0, 0)
            .expect("authorized canonical target remains readable");
        let talos_core::message::ContentPart::Image { path, .. } = part else {
            panic!("expected image content part");
        };
        assert_eq!(path, approved_canonical);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn stale_attachment_approval_reports_error_without_reading() {
        let workspace = tempfile::tempdir().expect("operation should succeed");
        let external = tempfile::tempdir().expect("operation should succeed");
        let target = external.path().join("stale.png");
        write_png(&target, 2);
        let permission_state = Arc::new(talos_permission::PermissionSessionState::new(
            talos_permission::PermissionEngine::with_workspace_root(workspace.path().to_path_buf()),
        ));
        let permission_engine = Some(permission_state.clone());
        let (ui_tx, mut ui_rx) = tokio::sync::mpsc::unbounded_channel();
        let mut authorization = Box::pin(authorize_attach_image(
            &ui_tx,
            &permission_engine,
            target.to_str().expect("operation should succeed"),
        ));

        let approval = tokio::select! {
            output = ui_rx.recv() => output.expect("authorization output"),
            _ = &mut authorization => panic!("external attachment must request approval"),
        };
        let UiOutput::ToolApprovalRequest { response, .. } = approval else {
            panic!("expected attachment approval request");
        };
        permission_state
            .rebind_session()
            .expect("invalidate approval");
        response
            .send(ApprovalChoice::ApproveOnce)
            .expect("operation should succeed");
        assert!(authorization.await.is_none());

        let error = ui_rx.recv().await.expect("visible stale error");
        let UiOutput::Content(ContentOutput::Block {
            source: MessageSource::Error,
            text,
        }) = error
        else {
            panic!("expected visible attachment error");
        };
        assert!(text.contains("authorization became stale"));
        assert!(text.contains("No file was read"));
    }
}
