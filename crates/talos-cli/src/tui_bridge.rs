//! Bridge between the conversation engine and the TUI.
//!
//! Contains the conversation loop that mediates between agent events,
//! user input, and UI output channels.

use std::sync::Arc;

use tokio::sync::Mutex;

use crate::skill_runtime::RuntimeSkills;
use talos_conversation::MessageSource;
use talos_conversation::{
    ConversationEngine, CredentialResponseData, ModelSwitchRequest, SessionDeleteRequest,
    SessionForkRequest, SessionNewRequest, SessionResumeRequest, SkillCommandRequest,
    StreamMessage, UiOutput, UserInput,
};
use talos_core::message::AgentEvent;

pub(crate) struct ConversationLoopIo {
    pub agent_rx: tokio::sync::mpsc::UnboundedReceiver<AgentEvent>,
    pub user_rx: tokio::sync::mpsc::UnboundedReceiver<UserInput>,
    pub ui_tx: tokio::sync::mpsc::UnboundedSender<UiOutput>,
    pub submit_tx: tokio::sync::mpsc::UnboundedSender<String>,
    pub sq_tx_watch:
        tokio::sync::watch::Receiver<tokio::sync::mpsc::Sender<talos_core::session::SessionOp>>,
    pub model_info_watch: tokio::sync::watch::Receiver<(String, String)>,
    pub session_tx: tokio::sync::mpsc::UnboundedSender<SessionLifecycleRequest>,
    pub runtime_skills: Arc<Mutex<RuntimeSkills>>,
}

pub(crate) async fn run_conversation_loop(mut engine: ConversationEngine, io: ConversationLoopIo) {
    let ConversationLoopIo {
        mut agent_rx,
        mut user_rx,
        ui_tx,
        submit_tx,
        sq_tx_watch,
        mut model_info_watch,
        session_tx,
        runtime_skills,
    } = io;

    loop {
        tokio::select! {
            changed = model_info_watch.changed() => {
                if changed.is_ok() {
                    let (model, provider) = model_info_watch.borrow().clone();
                    engine.set_model_info(&model, &provider);
                    let _ = ui_tx.send(UiOutput::Status(engine.status_snapshot()));
                }
            }
            event = agent_rx.recv() => {
                match event {
                    Some(agent_event) => {
                        let is_turn_end = matches!(agent_event, AgentEvent::TurnEnd { .. });
                        let outputs = engine.handle_agent_event(&agent_event);
                        for output in outputs {
                            let _ = ui_tx.send(output);
                        }
                        if is_turn_end
                            && let Some(msg) = engine.drain_steering_queue()
                        {
                            let outputs = engine.start_user_message(&msg);
                            for output in outputs {
                                let _ = ui_tx.send(output);
                            }
                            let _ = ui_tx.send(UiOutput::Status(engine.status_snapshot()));
                            let _ = submit_tx.send(msg);
                        }
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
                                    UiOutput::ModelSwitchRequest(req) => {
                                        let _ = session_tx.send(SessionLifecycleRequest::ModelSwitch(req));
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
                            let _ = submit_tx.send(msg);
                        }
                    }
                    UserInput::Credential(resp) => {
                        let _ = session_tx.send(SessionLifecycleRequest::ModelSwitchWithCredential(resp));
                    }
                    UserInput::ProviderSetup(provider) => {
                        let _ = session_tx.send(SessionLifecycleRequest::ProviderSetup(provider));
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
    let _ = ui_tx.send(UiOutput::Stream(StreamMessage {
        source,
        stream: Box::pin(futures::stream::once(async move { text })),
    }));
}

/// Session lifecycle request forwarded from the conversation loop to the mode runner.
pub(crate) enum SessionLifecycleRequest {
    New(SessionNewRequest),
    Resume(SessionResumeRequest),
    Fork(SessionForkRequest),
    Delete(SessionDeleteRequest),
    ModelSwitch(ModelSwitchRequest),
    ModelSwitchWithCredential(CredentialResponseData),
    ProviderSetup(String),
}
