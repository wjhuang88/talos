//! Offline presentation fixture, not tool execution or permission acceptance.
//!
//! Run in a native terminal:
//! `cargo run --locked -p talos-tui --example i279_activity_acceptance`
//! Enter `next` for each stable inspection stage, `quit` to exit. Escape uses
//! the real TUI cancellation input route. Resize, scroll, click and drag use
//! the production TUI; no provider, filesystem tool, or network is invoked.

use std::io;
use talos_conversation::{ContentOutput, ConversationEngine, MessageSource, UiOutput, UserInput};
use talos_core::message::{
    AgentEvent, AssistantReasoning, Message, MessageToolResult, ReasoningBlock, StopReason,
    ToolCall, Usage,
};
use talos_core::session::TurnCompletionStatus;
use talos_core::tool::ToolProvenance;
use talos_tui::Tui;
use tokio::sync::mpsc;

fn send(tx: &mpsc::UnboundedSender<UiOutput>, outputs: Vec<UiOutput>) {
    for output in outputs {
        if tx.send(output).is_err() {
            break;
        }
    }
}

fn guide(tx: &mpsc::UnboundedSender<UiOutput>, text: &str) {
    send(
        tx,
        vec![UiOutput::Content(ContentOutput::Block {
            source: MessageSource::Assistant,
            text: format!("[OFFLINE DISPLAY FIXTURE] {text}"),
        })],
    );
}

fn event(engine: &mut ConversationEngine, tx: &mpsc::UnboundedSender<UiOutput>, event: AgentEvent) {
    send(tx, engine.handle_agent_event(&event));
}

fn call(id: &str, label: &str) -> AgentEvent {
    AgentEvent::ToolCall {
        call: ToolCall {
            id: id.into(),
            name: "fixture_tool".into(),
            input: serde_json::json!({"display_only": label, "执行": "没有执行工具"}),
        },
        provenance: ToolProvenance::Native,
        summary_fields: vec!["display_only".into()],
    }
}

fn result(id: &str, is_error: bool) -> AgentEvent {
    AgentEvent::ToolResult {
        result: MessageToolResult {
            tool_use_id: id.into(),
            content: (1..=18)
                .map(|n| format!("fixture result {n:02} 中文结果"))
                .collect::<Vec<_>>()
                .join("\n"),
            is_error,
        },
    }
}

fn end_response(engine: &mut ConversationEngine, tx: &mpsc::UnboundedSender<UiOutput>) {
    event(
        engine,
        tx,
        AgentEvent::TurnEnd {
            stop_reason: StopReason::ToolUse,
            usage: Usage::default(),
        },
    );
}

fn history() -> Vec<Message> {
    (1..=2)
        .map(|n| Message::Assistant {
            content: format!(
                "History {n}: {}",
                "ASCII continuation padding / 中文续行留白验证。".repeat(12)
            ),
            tool_calls: vec![],
            reasoning: Some(AssistantReasoning {
                provider: "offline-fixture".into(),
                model: "display-only".into(),
                blocks: vec![
                    ReasoningBlock::Plain {
                        text: format!(
                            "Thinking entry {n}: {}",
                            "独立展开折叠 ASCII 中文。".repeat(12)
                        ),
                    },
                    ReasoningBlock::Redacted {
                        data: "REDACTED_FIXTURE_MUST_NOT_RENDER".into(),
                    },
                ],
            }),
        })
        .collect()
}

async fn drive(tx: mpsc::UnboundedSender<UiOutput>, mut rx: mpsc::UnboundedReceiver<UserInput>) {
    let mut engine = ConversationEngine::new("display-only".into(), "offline-fixture".into());
    let mut stage = 0;
    guide(
        &tx,
        "0/5: Scroll up to two collapsed thinking entries. Click each title independently; drag text without toggling. Resize wide/narrow and inspect three-column continuation padding. Scroll away from the tail, then expand/collapse to check anchoring. Enter next for live thinking; quit exits. No tool is executed.",
    );
    while let Some(input) = rx.recv().await {
        match input {
            UserInput::Exit => {
                send(&tx, vec![UiOutput::Exit]);
                break;
            }
            UserInput::Cancel => {
                send(&tx, engine.cancel_turn());
                send(
                    &tx,
                    engine.handle_turn_completed(&TurnCompletionStatus::Cancelled),
                );
                stage = 0;
                guide(
                    &tx,
                    "Cancelled through real UserInput::Cancel. Live activity is cleared. Enter next to restart at live thinking; quit exits.",
                );
            }
            UserInput::Message(text) if matches!(text.trim(), "quit" | "/quit" | "/exit") => {
                send(&tx, vec![UiOutput::Exit]);
                break;
            }
            UserInput::Message(text) if text.trim() == "next" => {
                stage += 1;
                match stage {
                    1 => {
                        guide(
                            &tx,
                            "1/5: Live thinking has 24 logical lines. Inspect title/count above newest ten body rows; resize to change display-row count. Enter next for two same-name tool requests; Escape cancels.",
                        );
                        event(&mut engine, &tx, AgentEvent::TurnStart);
                        for n in 1..=24 {
                            event(
                                &mut engine,
                                &tx,
                                AgentEvent::ThinkingDelta {
                                    delta: format!(
                                        "{n:02}: ASCII and 中文 live thinking resize verification.{}",
                                        if n == 24 { "" } else { "\n" }
                                    ),
                                },
                            );
                        }
                    }
                    2 => {
                        guide(
                            &tx,
                            "2/5: Two independent fixture_tool requested titles. Neither means running. Thinking has become collapsed history. Enter next for reverse-order failed/succeeded results.",
                        );
                        event(&mut engine, &tx, call("call_0", "first request"));
                        event(&mut engine, &tx, call("call_1", "second request"));
                        end_response(&mut engine, &tx);
                    }
                    3 => {
                        guide(
                            &tx,
                            "3/5: Result call_1 failed; call_0 succeeded. Inspect identity and 18-line counts with bounded rolling bodies. Enter next to reuse call_0 in a new response.",
                        );
                        event(&mut engine, &tx, result("call_1", true));
                        event(&mut engine, &tx, result("call_0", false));
                    }
                    4 => {
                        guide(
                            &tx,
                            "4/5: New response reuses call_0. Exactly one fresh requested activity should appear, while prior history remains. Enter next to finalize.",
                        );
                        event(&mut engine, &tx, AgentEvent::TurnStart);
                        event(
                            &mut engine,
                            &tx,
                            call("call_0", "reused ID in new response"),
                        );
                        end_response(&mut engine, &tx);
                    }
                    5 => {
                        event(&mut engine, &tx, result("call_0", false));
                        send(
                            &tx,
                            engine.handle_turn_completed(&TurnCompletionStatus::Success {
                                final_text: String::new(),
                                new_messages: vec![],
                            }),
                        );
                        guide(
                            &tx,
                            "5/5: Fixture stopped; transient titles cleared. Inspect finalized history for one call/result per event, collapsed thinking, and intact scrolling/selection. This is mock presentation evidence only, not execution/permission acceptance. Enter quit to restore the terminal.",
                        );
                        stage = 5;
                    }
                    _ => {
                        stage = 5;
                        guide(
                            &tx,
                            "All fixture stages are complete. Enter quit to exit; history remains available for inspection.",
                        );
                    }
                }
            }
            _ => guide(
                &tx,
                "Enter next to advance or quit to exit. Only these fixture commands are handled; no runtime command is executed.",
            ),
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> io::Result<()> {
    let mut tui = Tui::new()?;
    tui.set_model_name("I279 offline display fixture".into());
    tui.set_provider("no provider / no tools".into());
    tui.hydrate_history(&history());
    let (output_tx, output_rx) = mpsc::unbounded_channel();
    let (input_tx, input_rx) = mpsc::unbounded_channel();
    tui.set_ui_output_rx(output_rx);
    tui.set_user_input_tx(input_tx);
    tokio::select! {
        result = tui.run() => result,
        () = drive(output_tx, input_rx) => Ok(()),
    }
    // Tui's Drop restores the terminal on either return path, including errors.
}
