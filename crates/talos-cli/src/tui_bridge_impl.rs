//! Bridge between the conversation engine and the TUI.
//!
//! Contains the conversation loop that mediates between agent events,
//! user input, and UI output channels.

use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;
use tokio::time::MissedTickBehavior;

use crate::mode_runtime::request_preview_payload;
use crate::skill_runtime::RuntimeSkills;
use talos_conversation::MessageSource;
use talos_conversation::{
    ContentOutput, ConversationEngine, CredentialResponseData, ModelInfo, ModelSwitchRequest,
    SessionDeleteRequest, SessionForkRequest, SessionNewRequest, SessionResumeRequest,
    SkillCommandRequest, TodoCommandRequest, UiOutput, UserInput,
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
enum ProgressMode {
    Unknown,
    Legacy,
    Structured,
}

#[derive(Debug)]
enum BridgeTurnState {
    Idle,
    Submitting {
        submission: StructuredSubmission,
        session_id: Option<String>,
        sender_generation: u64,
        cancel_requested: bool,
        last_reconcile: Instant,
        reconcile_attempts: u32,
    },
    AcceptedByActor {
        session_id: String,
        session_generation: u64,
        submission_id: String,
        receipt_id: String,
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
    },
    LegacyCancelling {
        session_id: String,
        turn_id: String,
        next_sequence: u64,
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
    pub permission_engine: Option<Arc<std::sync::Mutex<talos_permission::PermissionEngine>>>,
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
    let mut sender_generation = 0_u64;
    let mut known_sender = sq_tx_watch.borrow().clone();
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
                        sender_generation = sender_generation.saturating_add(1);
                        known_sender = current_sender;
                        if !engine.pending_image_attachments.is_empty() {
                            pending_attachment_generation = Some(sender_generation);
                            send_bridge_stream(
                                &ui_tx,
                                MessageSource::System,
                                "[System] Pending attachments were retained across the session runtime replacement.\n".into(),
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
                retry_submission_receipt(&mut turn_state, &sq_tx_watch, &ui_tx);
            }
            event = agent_rx.recv() => {
                match event {
                    Some(event) => {
                        handle_session_event(
                            event,
                            &mut engine,
                            &mut turn_state,
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
                                let _ = ui_tx.send(output);
                            }
                            if !turn_state.accepts_queued_input() {
                                dispatch_prepared_submission(
                                    &mut engine,
                                    &mut turn_state,
                                    &sq_tx_watch,
                                    &mut known_sender,
                                    &mut sender_generation,
                                    &ui_tx,
                                ).await;
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

async fn handle_session_event(
    event: SessionEvent,
    engine: &mut ConversationEngine,
    turn_state: &mut BridgeTurnState,
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
        SessionEvent::StructuredTurnEvent {
            session_id,
            session_generation,
            submission_id,
            receipt_id,
            turn_id,
            sequence,
            payload,
        } => {
            handle_structured_turn_event(
                engine,
                turn_state,
                sq_tx_watch,
                ui_tx,
                session_id,
                session_generation,
                submission_id,
                receipt_id,
                turn_id,
                sequence,
                payload,
            );
            if matches!(turn_state, BridgeTurnState::Idle) {
                dispatch_prepared_submission(
                    engine,
                    turn_state,
                    sq_tx_watch,
                    known_sender,
                    sender_generation,
                    ui_tx,
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
            let completed_success = handle_legacy_turn_event(
                engine,
                turn_state,
                ui_tx,
                session_id,
                turn_id,
                sequence,
                payload,
            );
            if completed_success {
                dispatch_prepared_submission(
                    engine,
                    turn_state,
                    sq_tx_watch,
                    known_sender,
                    sender_generation,
                    ui_tx,
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
                    cancel_requested,
                    last_reconcile,
                    reconcile_attempts,
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
        SessionEvent::SubmissionQueued { .. } | SessionEvent::SubmissionStarted { .. } => {
            // Compatibility projections only. Durable SubmissionReceipt and
            // StructuredTurnEvent own transfer and lifecycle authority.
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
    sq_tx_watch: &tokio::sync::watch::Receiver<tokio::sync::mpsc::Sender<SessionOp>>,
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
    let state = std::mem::replace(turn_state, BridgeTurnState::Idle);
    let BridgeTurnState::Submitting {
        submission,
        session_id: expected_session,
        sender_generation,
        cancel_requested,
        last_reconcile,
        reconcile_attempts,
    } = state
    else {
        *turn_state = state;
        return;
    };

    let metadata_matches = submission.id == submission_id
        && submission.id == reservation_id
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
            state: PendingSubmissionState::AcceptedPending
                | PendingSubmissionState::PausedPending,
            ..
        } => {
            if receipt_id.is_empty() {
                *turn_state = BridgeTurnState::Submitting {
                    submission,
                    session_id: Some(session_id),
                    sender_generation,
                    cancel_requested,
                    last_reconcile,
                    reconcile_attempts,
                };
                emit_bridge_error(ui_tx, "ignored an accepted submission receipt without identity");
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
                cancel_requested,
            };
            if cancel_requested {
                send_interrupt(sq_tx_watch, ui_tx);
            }
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
                &format!("the session rejected the queued submission ({reason:?}); input was retained"),
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
    for item in &submission.items {
        for output in engine.start_user_message(&item.text) {
            let _ = ui_tx.send(output);
        }
    }
    let _ = ui_tx.send(UiOutput::SteeringQueueSnapshot(
        engine.steering_queue_snapshot(),
    ));
    let _ = ui_tx.send(UiOutput::Status(engine.status_snapshot()));
    true
}

#[allow(clippy::too_many_arguments)]
fn handle_structured_turn_event(
    engine: &mut ConversationEngine,
    turn_state: &mut BridgeTurnState,
    sq_tx_watch: &tokio::sync::watch::Receiver<tokio::sync::mpsc::Sender<SessionOp>>,
    ui_tx: &tokio::sync::mpsc::UnboundedSender<UiOutput>,
    session_id: String,
    session_generation: u64,
    submission_id: String,
    receipt_id: String,
    turn_id: String,
    sequence: u64,
    payload: TurnEventPayload,
) {
    let state = std::mem::replace(turn_state, BridgeTurnState::Idle);
    match payload {
        TurnEventPayload::Started => match state {
            BridgeTurnState::AcceptedByActor {
                session_id: expected_session,
                session_generation: expected_generation,
                submission_id: expected_submission,
                receipt_id: expected_receipt,
                cancel_requested,
            } if expected_session == session_id
                && expected_generation == session_generation
                && expected_submission == submission_id
                && expected_receipt == receipt_id
                && sequence == 0 =>
            {
                for output in engine.handle_turn_started() {
                    let _ = ui_tx.send(output);
                }
                if cancel_requested {
                    send_interrupt(sq_tx_watch, ui_tx);
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
            }
        }
        TurnEventPayload::Completed { status } => {
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
            let success = matches!(status, TurnCompletionStatus::Success { .. });
            *turn_state = if success {
                BridgeTurnState::Idle
            } else {
                BridgeTurnState::PausedAfterFailure
            };
            let _ = ui_tx.send(UiOutput::SteeringQueueSnapshot(
                engine.steering_queue_snapshot(),
            ));
        }
        _ => {
            *turn_state = state;
        }
    }
}

fn handle_legacy_turn_event(
    engine: &mut ConversationEngine,
    turn_state: &mut BridgeTurnState,
    ui_tx: &tokio::sync::mpsc::UnboundedSender<UiOutput>,
    session_id: String,
    turn_id: String,
    sequence: u64,
    payload: TurnEventPayload,
) -> bool {
    if matches!(
        turn_state,
        BridgeTurnState::StructuredRunning { .. }
            | BridgeTurnState::StructuredCancelling { .. }
    ) {
        return handle_structured_legacy_projection(
            engine,
            turn_state,
            ui_tx,
            session_id,
            turn_id,
            sequence,
            payload,
        );
    }

    let state = std::mem::replace(turn_state, BridgeTurnState::Idle);
    match state {
        BridgeTurnState::Idle
            if sequence == 0 && matches!(payload, TurnEventPayload::Started) =>
        {
            for output in engine.handle_turn_started() {
                let _ = ui_tx.send(output);
            }
            *turn_state = BridgeTurnState::LegacyRunning {
                session_id,
                turn_id,
                next_sequence: 1,
            };
            false
        }
        BridgeTurnState::LegacyRunning {
            session_id: expected_session,
            turn_id: expected_turn,
            mut next_sequence,
        }
        | BridgeTurnState::LegacyCancelling {
            session_id: expected_session,
            turn_id: expected_turn,
            mut next_sequence,
        } => {
            let cancelling = matches!(state, BridgeTurnState::LegacyCancelling { .. });
            if expected_session != session_id
                || expected_turn != turn_id
                || next_sequence != sequence
            {
                *turn_state = if cancelling {
                    BridgeTurnState::LegacyCancelling {
                        session_id: expected_session,
                        turn_id: expected_turn,
                        next_sequence,
                    }
                } else {
                    BridgeTurnState::LegacyRunning {
                        session_id: expected_session,
                        turn_id: expected_turn,
                        next_sequence,
                    }
                };
                emit_bridge_error(ui_tx, "ignored stale or out-of-order legacy session event");
                return false;
            }
            next_sequence = next_sequence.saturating_add(1);
            match payload {
                TurnEventPayload::Started => {
                    for output in engine.handle_turn_started() {
                        let _ = ui_tx.send(output);
                    }
                    *turn_state = if cancelling {
                        BridgeTurnState::LegacyCancelling {
                            session_id,
                            turn_id,
                            next_sequence,
                        }
                    } else {
                        BridgeTurnState::LegacyRunning {
                            session_id,
                            turn_id,
                            next_sequence,
                        }
                    };
                    false
                }
                TurnEventPayload::Progress {
                    event: AgentEvent::Error { .. },
                } => {
                    *turn_state = if cancelling {
                        BridgeTurnState::LegacyCancelling {
                            session_id,
                            turn_id,
                            next_sequence,
                        }
                    } else {
                        BridgeTurnState::LegacyRunning {
                            session_id,
                            turn_id,
                            next_sequence,
                        }
                    };
                    false
                }
                TurnEventPayload::Progress { event } => {
                    for output in engine.handle_agent_event(&event) {
                        let _ = ui_tx.send(output);
                    }
                    *turn_state = if cancelling {
                        BridgeTurnState::LegacyCancelling {
                            session_id,
                            turn_id,
                            next_sequence,
                        }
                    } else {
                        BridgeTurnState::LegacyRunning {
                            session_id,
                            turn_id,
                            next_sequence,
                        }
                    };
                    false
                }
                TurnEventPayload::Completed { status } => {
                    for output in engine.handle_turn_completed(&status) {
                        let _ = ui_tx.send(output);
                    }
                    let success = matches!(status, TurnCompletionStatus::Success { .. });
                    *turn_state = if success {
                        BridgeTurnState::Idle
                    } else {
                        BridgeTurnState::PausedAfterFailure
                    };
                    let _ = ui_tx.send(UiOutput::SteeringQueueSnapshot(
                        engine.steering_queue_snapshot(),
                    ));
                    success
                }
                _ => {
                    *turn_state = if cancelling {
                        BridgeTurnState::LegacyCancelling {
                            session_id,
                            turn_id,
                            next_sequence,
                        }
                    } else {
                        BridgeTurnState::LegacyRunning {
                            session_id,
                            turn_id,
                            next_sequence,
                        }
                    };
                    false
                }
            }
        }
        other => {
            *turn_state = other;
            false
        }
    }
}

fn handle_structured_legacy_projection(
    engine: &mut ConversationEngine,
    turn_state: &mut BridgeTurnState,
    ui_tx: &tokio::sync::mpsc::UnboundedSender<UiOutput>,
    session_id: String,
    turn_id: String,
    sequence: u64,
    payload: TurnEventPayload,
) -> bool {
    let state = std::mem::replace(turn_state, BridgeTurnState::Idle);
    match state {
        BridgeTurnState::StructuredRunning {
            session_id: expected_session,
            session_generation,
            submission_id,
            receipt_id,
            turn_id: expected_turn,
            next_structured_sequence,
            mut next_legacy_sequence,
            mut progress_mode,
        } => {
            if matches!(payload, TurnEventPayload::Started | TurnEventPayload::Completed { .. }) {
                *turn_state = BridgeTurnState::StructuredRunning {
                    session_id: expected_session,
                    session_generation,
                    submission_id,
                    receipt_id,
                    turn_id: expected_turn,
                    next_structured_sequence,
                    next_legacy_sequence,
                    progress_mode,
                };
                return false;
            }
            if expected_session != session_id
                || expected_turn != turn_id
                || next_legacy_sequence != sequence
            {
                *turn_state = BridgeTurnState::StructuredRunning {
                    session_id: expected_session,
                    session_generation,
                    submission_id,
                    receipt_id,
                    turn_id: expected_turn,
                    next_structured_sequence,
                    next_legacy_sequence,
                    progress_mode,
                };
                emit_bridge_error(ui_tx, "ignored stale structured compatibility progress");
                return false;
            }
            next_legacy_sequence = next_legacy_sequence.saturating_add(1);
            let should_emit = progress_mode != ProgressMode::Structured;
            if progress_mode == ProgressMode::Unknown {
                progress_mode = ProgressMode::Legacy;
            }
            if should_emit
                && let TurnEventPayload::Progress { event } = payload
                && !matches!(event, AgentEvent::Error { .. })
            {
                for output in engine.handle_agent_event(&event) {
                    let _ = ui_tx.send(output);
                }
            }
            *turn_state = BridgeTurnState::StructuredRunning {
                session_id: expected_session,
                session_generation,
                submission_id,
                receipt_id,
                turn_id: expected_turn,
                next_structured_sequence,
                next_legacy_sequence,
                progress_mode,
            };
            false
        }
        BridgeTurnState::StructuredCancelling {
            session_id: expected_session,
            session_generation,
            submission_id,
            receipt_id,
            turn_id: expected_turn,
            next_structured_sequence,
            mut next_legacy_sequence,
            mut progress_mode,
        } => {
            if matches!(payload, TurnEventPayload::Started | TurnEventPayload::Completed { .. }) {
                *turn_state = BridgeTurnState::StructuredCancelling {
                    session_id: expected_session,
                    session_generation,
                    submission_id,
                    receipt_id,
                    turn_id: expected_turn,
                    next_structured_sequence,
                    next_legacy_sequence,
                    progress_mode,
                };
                return false;
            }
            if expected_session != session_id
                || expected_turn != turn_id
                || next_legacy_sequence != sequence
            {
                *turn_state = BridgeTurnState::StructuredCancelling {
                    session_id: expected_session,
                    session_generation,
                    submission_id,
                    receipt_id,
                    turn_id: expected_turn,
                    next_structured_sequence,
                    next_legacy_sequence,
                    progress_mode,
                };
                emit_bridge_error(ui_tx, "ignored stale structured compatibility progress");
                return false;
            }
            next_legacy_sequence = next_legacy_sequence.saturating_add(1);
            let should_emit = progress_mode != ProgressMode::Structured;
            if progress_mode == ProgressMode::Unknown {
                progress_mode = ProgressMode::Legacy;
            }
            if should_emit
                && let TurnEventPayload::Progress { event } = payload
                && !matches!(event, AgentEvent::Error { .. })
            {
                for output in engine.handle_agent_event(&event) {
                    let _ = ui_tx.send(output);
                }
            }
            *turn_state = BridgeTurnState::StructuredCancelling {
                session_id: expected_session,
                session_generation,
                submission_id,
                receipt_id,
                turn_id: expected_turn,
                next_structured_sequence,
                next_legacy_sequence,
                progress_mode,
            };
            false
        }
        other => {
            *turn_state = other;
            false
        }
    }
}

fn retry_submission_receipt(
    turn_state: &mut BridgeTurnState,
    sq_tx_watch: &tokio::sync::watch::Receiver<tokio::sync::mpsc::Sender<SessionOp>>,
    ui_tx: &tokio::sync::mpsc::UnboundedSender<UiOutput>,
) {
    let (submission, attempts) = match turn_state {
        BridgeTurnState::Submitting {
            submission,
            last_reconcile,
            reconcile_attempts,
            ..
        } if last_reconcile.elapsed() >= RECEIPT_RECONCILE_AFTER => {
            *last_reconcile = Instant::now();
            *reconcile_attempts = reconcile_attempts.saturating_add(1);
            (Some(submission.clone()), *reconcile_attempts)
        }
        _ => (None, 0),
    };
    let Some(submission) = submission else {
        return;
    };
    let sender = sq_tx_watch.borrow().clone();
    if sender
        .try_send(SessionOp::ReconcileStructured { submission })
        .is_err()
        && attempts == RECEIPT_WARNING_ATTEMPT
    {
        emit_bridge_error(
            ui_tx,
            "submission receipt reconciliation is waiting for command-channel capacity; Engine escrow remains frozen",
        );
    } else if attempts == RECEIPT_WARNING_ATTEMPT {
        emit_bridge_error(
            ui_tx,
            "submission receipt was delayed; reconciliation is continuing with the same immutable identity",
        );
    }
}

fn request_cancel(
    turn_state: &mut BridgeTurnState,
    sq_tx_watch: &tokio::sync::watch::Receiver<tokio::sync::mpsc::Sender<SessionOp>>,
    ui_tx: &tokio::sync::mpsc::UnboundedSender<UiOutput>,
) {
    if !send_interrupt(sq_tx_watch, ui_tx) {
        return;
    }
    match turn_state {
        BridgeTurnState::Submitting {
            cancel_requested, ..
        }
        | BridgeTurnState::AcceptedByActor {
            cancel_requested, ..
        } => {
            *cancel_requested = true;
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
        } => {
            *turn_state = BridgeTurnState::LegacyCancelling {
                session_id: session_id.clone(),
                turn_id: turn_id.clone(),
                next_sequence: *next_sequence,
            };
        }
        _ => {}
    }
}

fn send_interrupt(
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
) {
    let current_sender = sq_tx_watch.borrow().clone();
    if !known_sender.same_channel(&current_sender) {
        *sender_generation = (*sender_generation).saturating_add(1);
        *known_sender = current_sender;
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
        cancel_requested: false,
        last_reconcile: Instant::now(),
        reconcile_attempts: 0,
    };
    let sender = sq_tx_watch.borrow().clone();
    let reserve = tokio::time::timeout(Duration::from_secs(1), sender.clone().reserve_owned()).await;
    let sent = match reserve {
        Ok(Ok(permit)) if sender.same_channel(&sq_tx_watch.borrow()) => {
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

fn emit_bridge_error(ui_tx: &tokio::sync::mpsc::UnboundedSender<UiOutput>, message: &str) {
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

/// P1-A: evaluates an image attachment path against the SEC-001
/// permission pipeline before any filesystem probe. Returns the
/// authorized canonical PathBuf on success, or None when rejected
/// (Deny, no engine available, or denied approval). The caller MUST
/// pass the returned canonical path to create_image_content_part —
/// NOT the original user-supplied path — so that symlink drift
/// between authorization and ingestion is impossible.
async fn authorize_attach_image(
    ui_tx: &tokio::sync::mpsc::UnboundedSender<UiOutput>,
    permission_engine: &Option<Arc<std::sync::Mutex<talos_permission::PermissionEngine>>>,
    path: &str,
) -> Option<std::path::PathBuf> {
    use crate::image_authorization::{
        ATTACH_IMAGE_TOOL_NAME, ImageAuthorization, add_attach_image_allow_rule,
    };
    use talos_core::ApprovalChoice;

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
    let canonical_str = canonical.display().to_string();

    let decision = {
        let engine = engine_ref.lock().expect("permission engine lock poisoned");
        ImageAuthorization::evaluate(&canonical, &engine)
    };

    match decision {
        ImageAuthorization::Allow => Some(canonical),
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
            let (response_tx, response_rx) =
                tokio::sync::oneshot::channel::<talos_core::ApprovalChoice>();
            let summary_fields = vec!["path".to_string()];
            if ui_tx
                .send(UiOutput::ToolApprovalRequest {
                    tool_name: ATTACH_IMAGE_TOOL_NAME.to_string(),
                    arguments: serde_json::json!({ "path": canonical_str }),
                    summary_fields,
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
                Ok(ApprovalChoice::AlwaysApprove) => {
                    let mut engine = engine_ref.lock().expect("permission engine lock poisoned");
                    add_attach_image_allow_rule(&mut engine, canonical.clone());
                    Some(canonical)
                }
                Ok(_) => Some(canonical),
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

#[cfg(test)]
mod attachment_authorization_tests {
    use super::*;
    #[cfg(unix)]
    use talos_core::ApprovalChoice;

    #[cfg(unix)]
    fn write_png(path: &std::path::Path, width: u32) {
        image::RgbaImage::new(width, 1)
            .save_with_format(path, image::ImageFormat::Png)
            .unwrap();
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

    #[cfg(unix)]
    #[tokio::test]
    async fn approved_symlink_drift_cannot_redirect_attachment_ingestion() {
        let workspace = tempfile::tempdir().unwrap();
        let external = tempfile::tempdir().unwrap();
        let approved = external.path().join("approved.png");
        let replacement = external.path().join("replacement.png");
        write_png(&approved, 2);
        write_png(&replacement, 9);

        let link = workspace.path().join("selected.png");
        std::os::unix::fs::symlink(&approved, &link).unwrap();
        let approved_canonical = approved.canonicalize().unwrap();

        let permission_engine = Some(Arc::new(std::sync::Mutex::new(
            talos_permission::PermissionEngine::with_workspace_root(workspace.path().to_path_buf()),
        )));
        let (ui_tx, mut ui_rx) = tokio::sync::mpsc::unbounded_channel();
        let mut authorization = Box::pin(authorize_attach_image(
            &ui_tx,
            &permission_engine,
            link.to_str().unwrap(),
        ));

        let approval = tokio::select! {
            output = ui_rx.recv() => output.expect("authorization output"),
            _ = &mut authorization => panic!("external attachment must request approval"),
        };
        let UiOutput::ToolApprovalRequest { response, .. } = approval else {
            panic!("expected attachment approval request");
        };
        response.send(ApprovalChoice::ApproveOnce).unwrap();
        let canonical = authorization.await.expect("approved canonical path");
        assert_eq!(canonical, approved_canonical);

        std::fs::remove_file(&link).unwrap();
        std::os::unix::fs::symlink(&replacement, &link).unwrap();

        let part = crate::image_validation::create_image_content_part(&canonical, 0, 0)
            .expect("authorized canonical target remains readable");
        let talos_core::message::ContentPart::Image { path, .. } = part else {
            panic!("expected image content part");
        };
        assert_eq!(path, approved_canonical);
    }
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
