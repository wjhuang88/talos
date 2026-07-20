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
    } = io;

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
                            if submit_session_message(&sq_tx_watch, msg).await.is_err() {
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
                    UserInput::SwitchModel { provider: _, model_id, variant } => {
                        let value = match variant {
                            Some(v) if !v.is_empty() => format!("{model_id}@{v}"),
                            _ => model_id,
                        };
                        let _ = session_tx.send(SessionLifecycleRequest::ModelSwitch(
                            ModelSwitchRequest {
                                model_id: value,
                                provider_needs_credential: false,
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
