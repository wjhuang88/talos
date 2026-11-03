//! Cancellable page-by-page startup history, without a whole-file fallback.

use anyhow::{Context, Result, bail};
use talos_core::message::Message;
use talos_session::Session;
use tokio_util::sync::CancellationToken;

/// Unpublished fork file. The worker owns cleanup while I/O is in flight, so
/// dropping the caller cannot race cleanup against an unfinished write.
pub(crate) struct StagedForkHistory {
    path: std::path::PathBuf,
    file: Option<std::fs::File>,
    retained: bool,
    child_cleanup: Option<(talos_session::SessionManager, Session)>,
    extra_path: Option<std::path::PathBuf>,
}

impl StagedForkHistory {
    pub(crate) fn install(&mut self, destination: &std::path::Path) -> Result<()> {
        drop(self.file.take());
        // Unlike rename, hard_link cannot overwrite an existing destination.
        std::fs::hard_link(&self.path, destination)?;
        let original = std::mem::replace(&mut self.path, destination.to_path_buf());
        if let Err(error) = std::fs::remove_file(&original) {
            self.extra_path = Some(original);
            return Err(error.into());
        }
        Ok(())
    }

    pub(crate) fn retain(&mut self) {
        self.retained = true;
    }

    pub(crate) fn own_child(&mut self, manager: talos_session::SessionManager, child: Session) {
        self.child_cleanup = Some((manager, child));
    }
}

impl Drop for StagedForkHistory {
    fn drop(&mut self) {
        drop(self.file.take());
        if self.retained {
            return;
        }
        if let Some(path) = &self.extra_path
            && let Err(error) = std::fs::remove_file(path)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            tracing::warn!(%error, "failed to remove fork staging alias");
        }
        if let Some((manager, child)) = &self.child_cleanup {
            if let Err(error) = manager.rollback_session_artifacts(child) {
                tracing::warn!(%error, session_id = %child.id, "failed to roll back unpublished fork artifacts; storage reconciliation required");
            }
            return;
        }
        if let Err(error) = std::fs::remove_file(&self.path)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            tracing::warn!(%error, "failed to remove unpublished fork history");
        }
    }
}

pub(crate) async fn stage_fork_history(
    session: &Session,
    staged_path: std::path::PathBuf,
) -> Result<(StagedForkHistory, Vec<Message>, talos_session::SessionEntry)> {
    let stop = CancellationToken::new();
    let _cancel_on_drop = stop.clone().drop_guard();
    let session = session.clone();
    let (mut scan, mut staged) = tokio::task::spawn_blocking(move || {
        let scan = session.begin_message_scan()?;
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let file = options.open(&staged_path)?;
        Ok::<_, anyhow::Error>((
            scan,
            StagedForkHistory {
                path: staged_path,
                file: Some(file),
                retained: false,
                child_cleanup: None,
                extra_path: None,
            },
        ))
    })
    .await
    .context("fork capture worker failed")??;
    loop {
        let worker_stop = stop.clone();
        let result = tokio::task::spawn_blocking(move || {
            use std::io::Write;
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
            let page = scan.copy_next_page(deadline, || worker_stop.is_cancelled())?;
            let copied = page.is_none();
            if let Some(page) = page {
                staged
                    .file
                    .as_mut()
                    .context("fork file closed")?
                    .write_all(&page)?;
            }
            let decoded = scan.step(deadline, || worker_stop.is_cancelled())?;
            Ok::<_, anyhow::Error>((scan, staged, copied && decoded))
        })
        .await
        .context("fork page worker failed")??;
        (scan, staged) = (result.0, result.1);
        if result.2 {
            break;
        }
    }
    tokio::task::spawn_blocking(move || {
        if stop.is_cancelled() {
            bail!("fork cancelled");
        }
        staged
            .file
            .as_ref()
            .context("fork file closed")?
            .sync_all()?;
        let (messages, tail) = scan.finish_with_tail()?;
        let tail = tail.context("cannot fork an empty Session")?;
        Ok((staged, messages, tail))
    })
    .await
    .context("fork finish worker failed")?
}

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

    #[tokio::test]
    async fn staged_fork_preserves_raw_bytes_and_cleans_up_without_overwriting() {
        let root = tempfile::tempdir().expect("root");
        let source = Session::new(
            uuid::Uuid::new_v4(),
            "test".into(),
            "test".into(),
            root.path().join("source.jsonl"),
        );
        let mut bytes = String::new();
        for i in 0..8300 {
            bytes.push_str(&serde_json::json!({"type":"message", "data":Message::User { content: format!("{i}:{}", "x".repeat(260)) }}).to_string());
            bytes.push('\n');
        }
        bytes.push_str("invalid trailing record\n");
        std::fs::write(&source.file_path, &bytes).expect("source");
        let stage = root.path().join("staged");
        let child = root.path().join("child.jsonl");
        let (mut guard, messages, _) = stage_fork_history(&source, stage.clone())
            .await
            .expect("stage");
        assert_eq!(messages.len(), 8300);
        assert_eq!(
            std::fs::read(&stage).expect("staged bytes"),
            bytes.as_bytes()
        );
        std::fs::write(&child, "existing child").expect("collision");
        assert!(guard.install(&child).is_err());
        assert_eq!(
            std::fs::read_to_string(&child).expect("preserved"),
            "existing child"
        );
        let installed = root.path().join("new-child.jsonl");
        guard.install(&installed).expect("install");
        assert!(!stage.exists());
        assert_eq!(std::fs::read(&installed).expect("child"), bytes.as_bytes());
        drop(guard);
        assert!(!installed.exists(), "unpublished child must be removed");
        assert_eq!(
            std::fs::read(&source.file_path).expect("source retained"),
            bytes.as_bytes()
        );
    }

    #[tokio::test]
    async fn cancelled_fork_owner_removes_runtime_index_and_lineage() {
        let root = tempfile::tempdir().expect("root");
        let manager = talos_session::SessionManager::with_dir(root.path().join("sessions"));
        let source = manager
            .create_session("source", "workspace")
            .expect("source");
        source
            .append(&Message::User {
                content: "original".into(),
            })
            .expect("message");
        let child = manager
            .defer_create_session("child", "workspace")
            .expect("child");
        let (mut staged, _, tail) =
            stage_fork_history(&source, child.file_path.with_extension("staging"))
                .await
                .expect("stage");
        staged.install(&child.file_path).expect("install");
        staged.own_child(manager.clone(), child.clone());
        talos_session::PendingSubmissionStore::for_session(&child)
            .initialize_runtime_identity(talos_session::SessionRuntimeIdentity::new(
                "test", "test", None,
            ))
            .expect("sidecar");
        manager.update_index(&source).expect("source index");
        manager.update_index(&child).expect("child index");
        manager
            .record_fork(&source.id, &child.id, &tail.id)
            .expect("lineage");
        assert_eq!(
            manager
                .get_forks(&source.id.to_string())
                .expect("forks")
                .len(),
            1
        );
        let reached = Arc::new(tokio::sync::Notify::new());
        let signal = reached.clone();
        let task = tokio::spawn(async move {
            let _owned = staged;
            signal.notify_one();
            std::future::pending::<()>().await;
        });
        reached.notified().await;
        task.abort();
        assert!(task.await.expect_err("cancelled").is_cancelled());
        assert!(!child.file_path.exists());
        assert!(
            manager
                .get_forks(&source.id.to_string())
                .expect("lineage removed")
                .is_empty()
        );
        assert!(
            manager
                .list_sessions()
                .expect("sessions")
                .iter()
                .all(|entry| entry.id != child.id)
        );
        assert!(
            talos_session::PendingSubmissionStore::for_session(&child)
                .runtime_state()
                .expect("sidecar removed")
                .is_none()
        );
        assert_eq!(
            source.read_messages().expect("source"),
            vec![Message::User {
                content: "original".into()
            }]
        );
    }

    #[tokio::test]
    async fn failed_fork_decode_removes_partial_staging_file() {
        let root = tempfile::tempdir().expect("root");
        let source = Session::new(
            uuid::Uuid::new_v4(),
            "test".into(),
            "test".into(),
            root.path().join("source.tlog"),
        );
        std::fs::write(&source.file_path, b"invalid header\n").expect("source");
        let stage = root.path().join("staged");
        assert!(stage_fork_history(&source, stage.clone()).await.is_err());
        assert!(!stage.exists());
    }

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
