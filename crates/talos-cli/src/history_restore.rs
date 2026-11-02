//! Cancellable page-by-page startup history, without a whole-file fallback.

use anyhow::{Context, Result, bail};
use talos_core::message::Message;
use talos_session::Session;
use tokio_util::sync::CancellationToken;

pub(crate) async fn read_session_history(session: &Session) -> Result<Vec<Message>> {
    let cancellation = CancellationToken::new();
    tokio::select! {
        biased;
        signal = tokio::signal::ctrl_c() => {
            signal.context("failed to observe startup interruption")?;
            cancellation.cancel();
            bail!("session history startup cancelled");
        }
        result = read_session_history_cancellable(session.clone(), cancellation.clone()) => result,
    }
}

async fn read_session_history_cancellable(
    session: Session,
    cancellation: CancellationToken,
) -> Result<Vec<Message>> {
    read_session_snapshot_cancellable(session, cancellation)
        .await
        .map(|(messages, _)| messages)
}

// Runtime transitions own their existing fence/rollback lifecycle. Dropping this
// future cancels its worker, without installing a second process signal handler.
pub(crate) async fn read_session_snapshot(
    session: &Session,
) -> Result<(Vec<Message>, Option<talos_session::SessionEntry>)> {
    #[cfg(test)]
    if let Ok(Some(reached)) = SNAPSHOT_CUT.try_with(|cut| {
        let remaining = cut.0.get();
        cut.0.set(remaining.saturating_sub(1));
        (remaining == 1).then(|| cut.1.clone())
    }) {
        reached.notify_one();
        std::future::pending::<()>().await;
    }
    read_session_snapshot_cancellable(session.clone(), CancellationToken::new()).await
}

#[cfg(test)]
tokio::task_local! {
    pub(crate) static SNAPSHOT_CUT: (std::cell::Cell<usize>, std::sync::Arc<tokio::sync::Notify>);
}

async fn read_session_snapshot_cancellable(
    session: Session,
    cancellation: CancellationToken,
) -> Result<(Vec<Message>, Option<talos_session::SessionEntry>)> {
    if cancellation.is_cancelled() {
        bail!("session history startup cancelled");
    }
    let worker_cancellation = cancellation.child_token();
    let _cancel_on_drop = worker_cancellation.clone().drop_guard();
    let capture = tokio::task::spawn_blocking(move || session.begin_message_scan());
    let mut scan = tokio::select! {
        biased;
        _ = cancellation.cancelled() => bail!("session history startup cancelled"),
        result = capture => result.context("history capture worker failed")?
            .context("session history source unavailable")?,
    };
    loop {
        let stop = worker_cancellation.clone();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        let page = tokio::task::spawn_blocking(move || {
            let complete = scan.step(deadline, || stop.is_cancelled())?;
            Ok::<_, talos_session::history_page::HistoryPageError>((scan, complete))
        });
        let (next, complete) = tokio::select! {
            biased;
            _ = cancellation.cancelled() => bail!("session history startup cancelled"),
            result = page => result.context("history page worker failed")?
                .context("session history restore failed")?,
        };
        scan = next;
        if complete {
            break;
        }
    }
    let finish = tokio::task::spawn_blocking(move || scan.finish_with_tail());
    tokio::select! {
        biased;
        _ = cancellation.cancelled() => bail!("session history startup cancelled"),
        result = finish => result.context("history finish worker failed")?
            .context("session history changed before startup"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::time::Duration;
    use talos_core::message::{AgentEvent, StopReason, Usage};
    use talos_core::provider::{LanguageModel, ProviderResult};
    use talos_core::session::{RuntimePolicy, SessionConfig, SessionEvent, SessionOp};
    use tokio::sync::mpsc::{self, Receiver};

    struct ObserveHistory(mpsc::UnboundedSender<Vec<Message>>);
    #[async_trait::async_trait]
    impl LanguageModel for ObserveHistory {
        async fn stream(&self, messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
            self.0.send(messages.to_vec()).expect("observer");
            let (tx, rx) = mpsc::channel(1);
            tx.send(AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                usage: Usage::default(),
            })
            .await
            .expect("terminal");
            Ok(rx)
        }
    }

    #[tokio::test]
    async fn paged_startup_restores_original_history_into_real_actor_provider_request() {
        let root = tempfile::tempdir().expect("root");
        let session = Session::new(
            uuid::Uuid::new_v4(),
            "test".into(),
            "test".into(),
            root.path().join("history.jsonl"),
        );
        let mut lines = String::new();
        for i in 0..8300 {
            let message = Message::User {
                content: format!("original-{i}: {}", "x".repeat(260)),
            };
            lines.push_str(&serde_json::json!({"type":"message", "data":message}).to_string());
            lines.push('\n');
        }
        assert!(lines.len() > 2 * 1024 * 1024);
        std::fs::write(&session.file_path, &lines).expect("fixture");
        let history = read_session_history_cancellable(session.clone(), CancellationToken::new())
            .await
            .expect("paged startup");
        assert_eq!(history.len(), 8300);
        let (observed, mut requests) = mpsc::unbounded_channel();
        #[allow(deprecated)]
        let agent = talos_agent::Agent::new(
            Arc::new(ObserveHistory(observed)),
            talos_core::tool::ToolRegistry::new(),
        );
        let (mut handle, mut actor) = talos_agent::session::AppServerSession::new(
            agent,
            SessionConfig {
                runtime_policy: RuntimePolicy::interactive(),
                workspace_root: root.path().into(),
                initial_history: history,
                model_context_limit: 2_000_000,
            },
        );
        let task = tokio::spawn(async move { actor.run().await });
        handle
            .sq_tx
            .send(SessionOp::Submit {
                message: "continue restored work".into(),
            })
            .await
            .expect("submit");
        let request = tokio::time::timeout(Duration::from_secs(10), requests.recv())
            .await
            .expect("request deadline")
            .expect("request");
        let users: Vec<_> = request
            .iter()
            .filter_map(|m| match m {
                Message::User { content } => Some(content.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(users.len(), 8301);
        for (i, content) in users[..8300].iter().enumerate() {
            assert_eq!(*content, format!("original-{i}: {}", "x".repeat(260)));
        }
        assert_eq!(users[8300], "continue restored work");
        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                if let SessionEvent::TurnEvent {
                    payload: talos_core::session::TurnEventPayload::Completed { status },
                    ..
                } = handle.eq_rx.recv().await.expect("Actor event stream")
                {
                    assert!(matches!(
                        status,
                        talos_core::session::TurnCompletionStatus::Success { .. }
                    ));
                    break;
                }
            }
        })
        .await
        .expect("turn completes");
        handle
            .sq_tx
            .send(SessionOp::Shutdown)
            .await
            .expect("shutdown");
        tokio::time::timeout(Duration::from_secs(5), task)
            .await
            .expect("shutdown deadline")
            .expect("actor");
        assert_eq!(
            std::fs::read_to_string(&session.file_path).expect("raw history"),
            lines
        );
    }

    #[tokio::test]
    async fn cancellation_and_bad_source_are_errors_never_empty_history() {
        let root = tempfile::tempdir().expect("root");
        let session = Session::new(
            uuid::Uuid::new_v4(),
            "test".into(),
            "test".into(),
            root.path().join("history.tlog"),
        );
        let token = CancellationToken::new();
        token.cancel();
        assert!(
            read_session_history_cancellable(session.clone(), token)
                .await
                .expect_err("cancel")
                .to_string()
                .contains("cancelled")
        );
        let raw = b"private-malformed-source\n";
        std::fs::write(&session.file_path, raw).expect("fixture");
        let error = read_session_history_cancellable(session.clone(), CancellationToken::new())
            .await
            .expect_err("bad source");
        assert!(!format!("{error:#}").contains("private-malformed-source"));
        assert_eq!(std::fs::read(&session.file_path).expect("unchanged"), raw);
    }
}
