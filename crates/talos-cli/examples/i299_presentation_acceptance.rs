//! Offline I299 native-terminal acceptance; no configured provider or user session is loaded.
//! Todo uses the real native tool and permission/runtime path against a temporary database.
//! Other stages inject display fixtures only: they do not execute edit/process tools.

use anyhow::{Context, Result, bail};
use std::{path::Path, sync::Arc, time::Duration};
use talos_conversation::{ContentOutput, ConversationEngine, MessageSource, UiOutput, UserInput};
use talos_core::{
    message::{AgentEvent, MessageToolResult, ToolCall},
    session::{SessionEvent, TurnCompletionStatus, TurnEventPayload},
    tool::{ToolNature, ToolProvenance},
};
use talos_permission::{PermissionDecision, PermissionRule};
use talos_provider::mock::MockProvider;
use talos_runtime::RuntimeBuilder;
use talos_session::todo::{CreateTodo, TodoPriority, TodoQueryTool, TodoRepository, TodoStatus};
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
            text: format!("[I299 离线验收] {text}"),
        })],
    );
}

fn call(name: &str, input: serde_json::Value, fields: &[&str]) -> AgentEvent {
    AgentEvent::ToolCall {
        call: ToolCall {
            id: name.into(),
            name: name.into(),
            input,
        },
        provenance: ToolProvenance::Native,
        summary_fields: fields.iter().map(|field| (*field).into()).collect(),
    }
}

fn event(engine: &mut ConversationEngine, tx: &mpsc::UnboundedSender<UiOutput>, event: AgentEvent) {
    send(tx, engine.handle_agent_event(&event));
}

async fn todo_events(root: &Path, long: bool) -> Result<Vec<AgentEvent>> {
    let session_id = uuid::Uuid::new_v4();
    let db = root.join(format!("{session_id}.sqlite"));
    let repo = TodoRepository::new(&db)?;
    repo.init_schema()?;
    for (index, status) in [
        TodoStatus::Todo,
        TodoStatus::Completed,
        TodoStatus::Blocked,
        TodoStatus::InProgress,
    ]
    .into_iter()
    .enumerate()
    {
        let title = if long && status == TodoStatus::InProgress {
            format!(
                "进行中：多行标题\n{}尾部用户文字（状态行被省略时不应高亮）",
                "标题续行\n".repeat(35)
            )
        } else if index == 0 {
            "真实任务 0：中文与 ASCII\n  [x] 用户原文，不应转为图标".into()
        } else {
            format!("真实任务 {index}：中文与 ASCII wrapping 测试")
        };
        let item = repo.create(CreateTodo {
            session_id,
            title,
            priority: TodoPriority::Medium,
            description: None,
            assigned_to_turn: None,
            tags: vec![],
        })?;
        repo.update_status(session_id, item.id, status)?;
    }
    drop(repo);
    let provider = MockProvider::new()
        .with_tool_call("todo_query", serde_json::json!({}))
        .with_response("Todo 查询完成。");
    let mut runtime = RuntimeBuilder::new()
        .provider(Arc::new(provider))
        .workspace_root(root)
        .tool(Arc::new(TodoQueryTool::new(db, session_id)))
        .permission_rule(PermissionRule::new_nature(
            ToolNature::Internal,
            None,
            None,
            PermissionDecision::Allow,
        ))
        .build()?;
    runtime.submit("query fixture todos").await?;
    let collected = tokio::time::timeout(Duration::from_secs(15), async {
        let mut events = vec![];
        while let Some(output) = runtime.next_event().await {
            match output {
                SessionEvent::TurnEvent {
                    payload: TurnEventPayload::Progress { event },
                    ..
                } => events.push(event),
                SessionEvent::TurnEvent {
                    payload: TurnEventPayload::Completed { status },
                    ..
                } => {
                    if !matches!(status, TurnCompletionStatus::Success { .. }) {
                        bail!("fixture turn failed: {status:?}");
                    }
                    return Ok(events);
                }
                _ => {}
            }
        }
        bail!("fixture ended without completion")
    })
    .await
    .context("fixture runtime deadline")?;
    runtime.shutdown().await?;
    let events = collected?;
    for event in &events {
        if let AgentEvent::ToolResult {
            result,
            todo_presentation: Some(metadata),
        } = event
        {
            let spans = metadata
                .spans_for(&result.content)
                .context("fixture bound spans")?;
            anyhow::ensure!(spans.len() == 4, "fixture must cover all four real items");
            anyhow::ensure!(
                result.content.contains("[x] 用户原文"),
                "forged row must remain raw"
            );
            if long {
                let active = spans
                    .iter()
                    .find(|span| span.status == talos_core::tool::TodoDisplayStatus::InProgress)
                    .context("active item")?;
                let status_line = result.content[..active.status_range.start]
                    .bytes()
                    .filter(|byte| *byte == b'\n')
                    .count();
                anyhow::ensure!(
                    status_line >= 3
                        && status_line < result.content.lines().count().saturating_sub(3),
                    "active status must be in omitted middle"
                );
                anyhow::ensure!(
                    active.item_range.end == result.content.len(),
                    "active continuation must reach tail"
                );
            }
        }
    }
    if !events.iter().any(|event| matches!(event, AgentEvent::ToolResult { result, todo_presentation: Some(_) } if !result.is_error)) {
        bail!("native Todo metadata did not reach UI boundary");
    }
    Ok(events)
}

async fn drive(
    tx: mpsc::UnboundedSender<UiOutput>,
    mut rx: mpsc::UnboundedReceiver<UserInput>,
    normal: Vec<AgentEvent>,
    long: Vec<AgentEvent>,
) {
    let mut engine = ConversationEngine::new("I299".into(), "offline".into());
    let mut stage = 0;
    guide(
        &tx,
        "输入 next 开始。共四项：活动与队列、编辑背景、真实 Todo、长结果截断。quit 退出。仅临时 Todo 数据库有写入。",
    );
    while let Some(input) = rx.recv().await {
        match input {
            UserInput::Exit => break,
            UserInput::Cancel => {
                send(&tx, engine.cancel_turn());
                send(
                    &tx,
                    engine.handle_turn_completed(&TurnCompletionStatus::Cancelled),
                );
                guide(&tx, "已取消展示活动；next 继续，quit 退出。");
            }
            UserInput::Message(text) if matches!(text.trim(), "quit" | "/quit" | "/exit") => break,
            UserInput::Message(text) if text.trim() == "next" => {
                stage += 1;
                send(
                    &tx,
                    engine.handle_turn_completed(&TurnCompletionStatus::Success {
                        final_text: String::new(),
                        new_messages: vec![],
                    }),
                );
                while engine.drain_steering_queue().is_some() {}
                send(
                    &tx,
                    vec![UiOutput::SteeringQueueSnapshot(
                        engine.steering_queue_snapshot(),
                    )],
                );
                match stage {
                    1 => {
                        guide(
                            &tx,
                            "1/4：观察活动动画；缩窄、缩短窗口，确认下方队列正文仍可读。process 参数只是展示，不会执行。next 继续。",
                        );
                        event(&mut engine, &tx, AgentEvent::TurnStart);
                        event(
                            &mut engine,
                            &tx,
                            call(
                                "process",
                                serde_json::json!({"action":"read", "job_id":"fixture-job", "cursor":7, "max_bytes":2048, "wait_ms":100}),
                                &["action", "job_id", "cursor", "max_bytes", "wait_ms"],
                            ),
                        );
                        send(
                            &tx,
                            engine.enqueue_steering("队列第一条：请保留中文与 ASCII 正文。".into()),
                        );
                        send(
                            &tx,
                            engine.enqueue_steering("队列第二条：缩短窗口后仍能读到内容。".into()),
                        );
                    }
                    2 => {
                        guide(
                            &tx,
                            "2/4：编辑结果是固定展示数据，不修改文件。红绿背景应覆盖中文换行；拖选后文字正确，正文使用小圆点。next 继续。",
                        );
                        event(&mut engine, &tx, AgentEvent::TurnStart);
                        event(
                            &mut engine,
                            &tx,
                            call("edit", serde_json::json!({"path":"fixture.txt"}), &["path"]),
                        );
                        event(
                            &mut engine,
                            &tx,
                            AgentEvent::ToolResult {
                                todo_presentation: None,
                                result: MessageToolResult {
                                    tool_use_id: "edit".into(),
                                    is_error: false,
                                    content: format!(
                                        "--- a/fixture.txt\n+++ b/fixture.txt\n@@ -1 +1 @@\n-{}\n+{}",
                                        "旧内容 中文 ASCII ".repeat(12),
                                        "新内容 中文 ASCII ".repeat(12)
                                    ),
                                },
                            },
                        );
                        send(
                            &tx,
                            engine.handle_turn_completed(&TurnCompletionStatus::Success {
                                final_text: String::new(),
                                new_messages: vec![],
                            }),
                        );
                    }
                    3 | 4 => {
                        guide(
                            &tx,
                            if stage == 3 {
                                "3/4：真实 Todo 查询。四种状态应显示对应图形，进行中项有高亮；标题续行中的伪状态保留原文。缩窄窗口观察换行。next 查看长结果。"
                            } else {
                                "4/4：长 Todo 结果应仅显示首尾与省略提示；省略状态行不能留下孤立高亮。滚动/拖选检查后 quit 退出。"
                            },
                        );
                        for output in if stage == 3 { &normal } else { &long } {
                            event(&mut engine, &tx, output.clone());
                        }
                        send(
                            &tx,
                            engine.handle_turn_completed(&TurnCompletionStatus::Success {
                                final_text: String::new(),
                                new_messages: vec![],
                            }),
                        );
                    }
                    _ => guide(
                        &tx,
                        "四项已展示；quit 退出。请把实际观察结果反馈给维护会话，本程序不会自动认定验收通过。",
                    ),
                }
            }
            _ => guide(&tx, "输入 next 继续，quit 退出。"),
        }
    }
    send(&tx, vec![UiOutput::Exit]);
}

#[tokio::main]
async fn main() -> Result<()> {
    let root = tempfile::tempdir()?;
    let normal = todo_events(root.path(), false).await?;
    let long = todo_events(root.path(), true).await?;
    if std::env::args().any(|arg| arg == "--check") {
        println!(
            "I299 native Todo runtime fixtures passed (normal and truncated); terminal acceptance not performed."
        );
        return Ok(());
    }
    let mut tui = Tui::new()?;
    tui.set_model_name("I299 presentation acceptance".into());
    tui.set_provider("offline / isolated Todo".into());
    let (tx, rx) = mpsc::unbounded_channel();
    let (input_tx, input_rx) = mpsc::unbounded_channel();
    tui.set_ui_output_rx(rx);
    tui.set_user_input_tx(input_tx);
    tokio::select! {
        result = tui.run() => result?,
        () = drive(tx, input_rx, normal, long) => {},
    }
    Ok(())
}
