//! Bridge between the conversation engine and the TUI.
//!
//! Contains the conversation loop that mediates between agent events,
//! user input, and UI output channels.

use talos_conversation::{ConversationEngine, SessionNewRequest, SessionResumeRequest, UiOutput, UserInput};
use talos_core::message::AgentEvent;

pub(crate) async fn run_conversation_loop(
    mut engine: ConversationEngine,
    mut agent_rx: tokio::sync::mpsc::UnboundedReceiver<AgentEvent>,
    mut user_rx: tokio::sync::mpsc::UnboundedReceiver<UserInput>,
    ui_tx: tokio::sync::mpsc::UnboundedSender<UiOutput>,
    submit_tx: tokio::sync::mpsc::UnboundedSender<String>,
    interrupt_tx: tokio::sync::mpsc::Sender<talos_core::session::SessionOp>,
    session_tx: tokio::sync::mpsc::UnboundedSender<SessionLifecycleRequest>,
) {
    loop {
        tokio::select! {
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
                    UserInput::Cancel => {
                        let _ = interrupt_tx.send(talos_core::session::SessionOp::Interrupt).await;
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

/// Session lifecycle request forwarded from the conversation loop to the mode runner.
pub(crate) enum SessionLifecycleRequest {
    New(SessionNewRequest),
    Resume(SessionResumeRequest),
}
