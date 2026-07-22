//! Bridge between the conversation engine and the TUI.
//!
//! Contains the conversation loop that mediates between agent events,
//! user input, and UI output channels.

use std::sync::Arc;

use tokio::sync::Mutex;

use crate::mode_runtime::request_preview_payload;
use crate::skill_runtime::RuntimeSkills;
use talos_conversation::MessageSource;
use talos_conversation::{
    ContentOutput, ConversationEngine, CredentialResponseData, ModelInfo, ModelSwitchRequest,
    SessionDeleteRequest, SessionForkRequest, SessionNewRequest, SessionResumeRequest,
    SkillCommandRequest, TodoCommandRequest, UiOutput, UserInput,
};
use talos_core::message::AgentEvent;
use talos_core::session::{SessionEvent, TurnEventPayload};

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
        sq_tx_watch,
        mut model_info_watch,
        session_tx,
        runtime_skills,
        permission_engine,
    } = io;

    // A watch receiver exposes its initial value without emitting `changed()`.
    // Apply it before entering the loop so a freshly started TUI has the same
    // model capabilities as a model selected later during the session.
    engine.set_model_info(&model_info_watch.borrow().clone());
    let _ = ui_tx.send(UiOutput::Status(engine.status_snapshot()));

    loop {
        tokio::select! {
            changed = model_info_watch.changed() => {
                if changed.is_ok() {
                    let info = model_info_watch.borrow().clone();
                    engine.set_model_info(&info);
                    let _ = ui_tx.send(UiOutput::Status(engine.status_snapshot()));
                }
            }
            event = agent_rx.recv() => {
                match event {
                    Some(SessionEvent::TurnEvent { payload, .. }) => {
                        let turn_completed = matches!(payload, TurnEventPayload::Completed { .. });
                        let outputs = match payload {
                            TurnEventPayload::Started => engine.handle_turn_started(),
                            TurnEventPayload::Progress { event: AgentEvent::Error { .. } } => {
                                // The authoritative terminal error follows as Completed.
                                Vec::new()
                            }
                            TurnEventPayload::Progress { event } => engine.handle_agent_event(&event),
                            TurnEventPayload::Completed { status } => {
                                engine.handle_turn_completed(&status)
                            }
                            _ => Vec::new(),
                        };
                        for output in outputs {
                            let _ = ui_tx.send(output);
                        }
                        if turn_completed
                            && let Some(msg) = engine.drain_steering_queue()
                        {
                            let outputs = engine.start_user_message(&msg);
                            for output in outputs {
                                let _ = ui_tx.send(output);
                            }
                            let _ = ui_tx.send(UiOutput::SteeringQueueSnapshot(
                                engine.steering_queue_snapshot(),
                            ));
                            let _ = ui_tx.send(UiOutput::Status(engine.status_snapshot()));
                            if submit_session_message(&sq_tx_watch, msg).await.is_err() {
                                for output in engine.handle_turn_completed(
                                    &talos_core::session::TurnCompletionStatus::Error {
                                        message: "session command channel closed".into(),
                                    },
                                ) {
                                    let _ = ui_tx.send(output);
                                }
                            }
                        } else if turn_completed {
                            let _ = ui_tx.send(UiOutput::SteeringQueueSnapshot(
                                engine.steering_queue_snapshot(),
                            ));
                        }
                    }
                    Some(SessionEvent::Error { message }) => {
                        for output in engine.handle_turn_completed(
                            &talos_core::session::TurnCompletionStatus::Error { message },
                        ) {
                            let _ = ui_tx.send(output);
                        }
                    }
                    Some(_) => {}
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
                                        let _ = session_tx.send(SessionLifecycleRequest::New(req));
                                    }
                                    UiOutput::SessionResume(req) => {
                                        let _ = session_tx.send(SessionLifecycleRequest::Resume(req));
                                    }
                                    UiOutput::SessionFork(req) => {
                                        let _ = session_tx.send(SessionLifecycleRequest::Fork(req));
                                    }
                                    UiOutput::SessionDelete(req) => {
                                        let _ = session_tx.send(SessionLifecycleRequest::Delete(req));
                                    }
                                    UiOutput::TodoCommand(req) => {
                                        let _ = session_tx.send(SessionLifecycleRequest::Todo(req));
                                    }
                                    UiOutput::ModelSwitchRequest(req) => {
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
                                        handle_skill_command(
                                            req,
                                            &mut engine,
                                            &ui_tx,
                                            &sq_tx_watch,
                                            runtime_skills.clone(),
                                        ).await;
                                    }
                                    UiOutput::AttachImageRequest { path } => {
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
                                                        format!("{} ({} bytes, {})",
                                                            escape_markdown_filename(&path.file_name().unwrap_or_default().to_string_lossy()),
                                                            byte_count, mime),
                                                    _ => String::new(),
                                                };
                                                engine.pending_image_attachments.push(content_part);
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
                                    other => { let _ = ui_tx.send(other); }
                                }
                            }
                        } else if engine.is_processing() {
                            for output in engine.enqueue_steering(msg) {
                                let _ = ui_tx.send(output);
                            }
                        } else {
                            let outputs = engine.start_user_message(&msg);
                            for output in outputs {
                                let _ = ui_tx.send(output);
                            }
                            let _ = ui_tx.send(UiOutput::Status(engine.status_snapshot()));
                            let attachments = std::mem::take(&mut engine.pending_image_attachments);
                            let submit_result = if attachments.is_empty() {
                                submit_session_message(&sq_tx_watch, msg).await
                            } else {
                                submit_session_message_multimodal(&sq_tx_watch, msg, attachments).await
                            };
                            if submit_result.is_err() {
                                for output in engine.handle_turn_completed(
                                    &talos_core::session::TurnCompletionStatus::Error {
                                        message: "session command channel closed".into(),
                                    },
                                ) {
                                    let _ = ui_tx.send(output);
                                }
                            }
                        }
                    }
                    UserInput::Credential(resp) => {
                        if resp.connect_mode {
                            let _ = session_tx.send(SessionLifecycleRequest::ConnectWithCredential(resp));
                        } else {
                            let _ = session_tx.send(SessionLifecycleRequest::ModelSwitchWithCredential(resp));
                        }
                    }
                    UserInput::ProviderSetup(provider) => {
                        let _ = session_tx.send(SessionLifecycleRequest::ProviderSetup(provider));
                    }
                    UserInput::SwitchModel { provider, model_id, variant } => {
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
                        let _ = session_tx.send(SessionLifecycleRequest::ConnectRequest { provider });
                    }
                    UserInput::RegisterCustomProvider { name, protocol, base_url, api_key } => {
                        let _ = session_tx.send(SessionLifecycleRequest::RegisterCustomProvider {
                            name,
                            protocol,
                            base_url,
                            api_key,
                        });
                    }
                    UserInput::Cancel => {
                        let sq_tx = sq_tx_watch.borrow().clone();
                        let _ = sq_tx.send(talos_core::session::SessionOp::Interrupt).await;
                        for output in engine.cancel_turn() {
                            let _ = ui_tx.send(output);
                        }
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

async fn submit_session_message(
    sq_tx_watch: &tokio::sync::watch::Receiver<
        tokio::sync::mpsc::Sender<talos_core::session::SessionOp>,
    >,
    message: String,
) -> Result<(), ()> {
    let sq_tx = sq_tx_watch.borrow().clone();
    let op = match request_preview_payload(&message) {
        Some(message) => talos_core::session::SessionOp::PreviewRequest { message },
        None => talos_core::session::SessionOp::Submit { message },
    };
    sq_tx.send(op).await.map_err(|_| ())
}

/// Escapes a basename for display in a Markdown-rendered system message.
///
/// The filename remains only a UI summary, but it must not be interpreted as
/// formatting (for example, underscores must remain visible).
fn escape_markdown_filename(filename: &str) -> String {
    filename
        .replace('\\', "\\\\")
        .replace('_', "\\_")
        .replace('*', "\\*")
        .replace('`', "\\`")
        .replace('[', "\\[")
        .replace(']', "\\]")
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

async fn submit_session_message_multimodal(
    sq_tx_watch: &tokio::sync::watch::Receiver<
        tokio::sync::mpsc::Sender<talos_core::session::SessionOp>,
    >,
    message: String,
    attachments: Vec<talos_core::message::ContentPart>,
) -> Result<(), ()> {
    let sq_tx = sq_tx_watch.borrow().clone();
    sq_tx
        .send(talos_core::session::SessionOp::SubmitMultimodal {
            text: message,
            attachments,
        })
        .await
        .map_err(|_| ())
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
    use talos_core::ApprovalChoice;

    fn write_png(path: &std::path::Path, width: u32) {
        image::RgbaImage::new(width, 1)
            .save_with_format(path, image::ImageFormat::Png)
            .unwrap();
    }

    #[test]
    fn attachment_filename_escapes_markdown_without_changing_visible_text() {
        assert_eq!(
            escape_markdown_filename("ScreenShot_2026-07-22_140448_800.png"),
            "ScreenShot\\_2026-07-22\\_140448\\_800.png"
        );
    }

    /// P1-A regression: after approval resolves a symlink to its canonical
    /// target, later changing the user-supplied symlink cannot redirect the
    /// image ingestion path. The bridge must use the returned canonical path.
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
