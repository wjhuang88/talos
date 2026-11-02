//! Host-prepared durable history queries, scoped to one configured execution.
use serde_json::{Value, json};
use std::{
    sync::Mutex,
    time::{Duration, Instant},
};
use talos_session::sqlite::{RecoveryBuildPage, RecoveryIndexSource, RecoveryWorkBudget};
use talos_session::{DurableSession, RecoveryPrefixSeal, SessionIndex};
use tokio_util::sync::CancellationToken;

pub(crate) struct History {
    path: std::path::PathBuf,
    scope: String,
    owner: String,
    cancel: CancellationToken,
    seal: RecoveryPrefixSeal,
    state: Mutex<QueryState>,
}
struct QueryState {
    secret: [u8; 32],
    cursor: Option<(String, String, Option<u64>)>,
}
impl Default for QueryState {
    fn default() -> Self {
        let mut secret = [0; 32];
        secret[..16].copy_from_slice(uuid::Uuid::new_v4().as_bytes());
        secret[16..].copy_from_slice(uuid::Uuid::new_v4().as_bytes());
        Self {
            secret,
            cursor: None,
        }
    }
}
impl QueryState {
    // Fixed-size domain/ordinal/secret input; not a general variable-length MAC.
    // The per-discovery secret is never persisted or exposed to the model.
    fn reference(&self, ordinal: u64) -> String {
        use sha2::{Digest, Sha256};
        let mut digest = Sha256::new();
        digest.update(b"talos-history-reference-v1");
        digest.update(ordinal.to_le_bytes());
        digest.update(self.secret);
        let tag = digest
            .finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        format!("{ordinal:016x}.{tag}")
    }
    fn ordinal(&self, reference: &str) -> Option<u64> {
        if reference.len() != 81 || !reference.is_ascii() {
            return None;
        }
        let ordinal = u64::from_str_radix(reference.get(..16)?, 16).ok()?;
        (self.reference(ordinal) == reference).then_some(ordinal)
    }
}

/// Host-owned progress; no partial state authorizes query results.
pub(crate) struct Preparation {
    #[cfg(any(unix, windows))]
    scan_store: ScanStore,
    source: talos_session::DurableRecoverySource,
    scan: Option<talos_session::RecoveryPrefixScan>,
    seal: Option<RecoveryPrefixSeal>,
    path: std::path::PathBuf,
    scope: String,
    parent: Option<String>,
    resolved: bool,
    #[cfg(test)]
    after_page_read: Option<Box<dyn FnOnce() + Send>>,
}
#[cfg(any(unix, windows))]
struct ScanStore {
    owner: DurableSession,
    receipt: Option<talos_session::RecoveryScanReceipt>,
    saved_bytes: u64,
    cancel: CancellationToken,
}
#[cfg(any(unix, windows))]
impl ScanStore {
    fn clear(&self) {
        if let Some(receipt) = &self.receipt {
            for attempt in 0..3 {
                match self.owner.clear_recovery_scan(
                    receipt,
                    &RecoveryWorkBudget::new(Duration::from_millis(100)),
                ) {
                    Ok(true) | Ok(false) => return,
                    Err(_) if attempt < 2 => {
                        tracing::debug!(attempt, "checkpoint cleanup will retry");
                    }
                    Err(_) => {
                        tracing::warn!("checkpoint cleanup failed after retries");
                    }
                }
            }
        }
    }
}
#[cfg(any(unix, windows))]
impl Drop for ScanStore {
    fn drop(&mut self) {
        // Actor interruption aborts the turn task after cancelling its token.
        // No async finalizer can be awaited from that dropped future.
        if self.cancel.is_cancelled() {
            self.clear();
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum PreparationStep {
    Pending,
    Complete,
    Expired,
}

impl Preparation {
    fn begin(
        session: &DurableSession,
        cancel: &CancellationToken,
        deadline: Instant,
    ) -> Option<Self> {
        if cancel.is_cancelled() || Instant::now() >= deadline {
            return None;
        }
        let source = session.capture_recovery_source().ok()?;
        if !session.owns_recovery_source(&source) {
            return None;
        }
        let scan = source.begin_prefix_seal().ok()?;
        let path = session.recovery_index_path().ok()?;
        let index = SessionIndex::new(&path).ok()?;
        let stopped = cancel.clone();
        let budget = RecoveryWorkBudget::with_cancellation_check(
            deadline
                .saturating_duration_since(Instant::now())
                .min(Duration::from_secs(2)),
            move || stopped.is_cancelled(),
        );
        index.init_recovery_schema_with_budget(&budget).ok()?;
        if let Ok(Some(saved)) = index.recovery_continuation(&session.id().to_string(), &budget)
            && let Some((source, scan)) = source
                .recovery_prefix_at(saved.prefix.cutoff())
                .ok()
                .and_then(|source| {
                    let scan = source.verify_prefix_expectation(&saved.prefix).ok()?;
                    Some((source, scan))
                })
        {
            #[cfg(any(unix, windows))]
            let restored = session
                .restore_recovery_scan_for_source(&source, &budget)
                .ok()?;
            #[cfg(any(unix, windows))]
            let (scan, receipt, saved_bytes) = match restored {
                Some(restored) if restored.scan.verifies_expectation(&saved.prefix) => {
                    let offset = restored.scan.completed_bytes();
                    (restored.scan, Some(restored.receipt), offset)
                }
                Some(restored) => {
                    // Retire only the exact incompatible record we observed.
                    // Busy/error or a concurrent replacement must be retried,
                    // never treated as permission to overwrite another owner.
                    if !session
                        .clear_recovery_scan(&restored.receipt, &budget)
                        .ok()?
                    {
                        return None;
                    }
                    (scan, None, 0)
                }
                None => (scan, None, 0),
            };
            return Some(Self {
                #[cfg(any(unix, windows))]
                scan_store: ScanStore {
                    owner: session.clone(),
                    receipt,
                    saved_bytes,
                    cancel: cancel.clone(),
                },
                source,
                scan: Some(scan),
                seal: None,
                path,
                scope: saved.scope,
                parent: saved.parent,
                resolved: false,
                #[cfg(test)]
                after_page_read: None,
            });
        }
        #[cfg(any(unix, windows))]
        let restored = session
            .restore_recovery_scan_for_source(&source, &budget)
            .ok()
            .flatten();
        #[cfg(any(unix, windows))]
        let (scan, scan_receipt, saved_scan_bytes) = match restored {
            Some(restored) => {
                let offset = restored.scan.completed_bytes();
                (restored.scan, Some(restored.receipt), offset)
            }
            None => (scan, None, 0),
        };
        let scope = uuid::Uuid::new_v4().to_string();
        index
            .replace_recovery_owner_scope(&session.id().to_string(), &scope, &budget)
            .ok()?;
        index
            .begin_recovery_build(&scope, source.source_end(), &budget)
            .ok()?;
        Some(Self {
            #[cfg(any(unix, windows))]
            scan_store: ScanStore {
                owner: session.clone(),
                receipt: scan_receipt,
                saved_bytes: saved_scan_bytes,
                cancel: cancel.clone(),
            },
            source,
            scan: Some(scan),
            seal: None,
            path,
            scope,
            parent: None,
            resolved: false,
            #[cfg(test)]
            after_page_read: None,
        })
    }
    fn persist(&mut self, cancel: &CancellationToken, deadline: Instant) -> Option<()> {
        // During revalidation retain the previous checkpoint; no new index page
        // can commit until the scan finishes and installs the refreshed seal.
        if let Some(scan) = &self.scan {
            #[cfg(not(any(unix, windows)))]
            let _ = scan;
            #[cfg(any(unix, windows))]
            if scan.completed_bytes() > self.scan_store.saved_bytes {
                let stopped = cancel.clone();
                let budget = RecoveryWorkBudget::with_cancellation_check(
                    deadline.saturating_duration_since(Instant::now()),
                    move || stopped.is_cancelled(),
                );
                self.scan_store.receipt = Some(
                    self.scan_store
                        .owner
                        .save_recovery_scan(scan, &budget)
                        .ok()?,
                );
                self.scan_store.saved_bytes = scan.completed_bytes();
            }
            return Some(());
        }
        let Some(seal) = &self.seal else {
            return Some(());
        };
        seal.check_current().ok()?;
        let stopped = cancel.clone();
        let budget = RecoveryWorkBudget::with_cancellation_check(
            deadline.saturating_duration_since(Instant::now()),
            move || stopped.is_cancelled(),
        );
        let index = SessionIndex::new(&self.path).ok()?;
        let cursor = index.recovery_build_cursor(&self.scope, &budget).ok()??;
        index
            .save_recovery_continuation(
                &self.source.session_id().to_string(),
                &talos_session::sqlite::RecoveryContinuation {
                    scope: self.scope.clone(),
                    offset: cursor.next_offset(),
                    ordinal: cursor.next_ordinal(),
                    parent: self.parent.clone(),
                    prefix: seal.expectation(),
                },
                &budget,
            )
            .ok()?;
        seal.check_current().ok()?;
        Some(())
    }
    fn retire(&self) {
        #[cfg(any(unix, windows))]
        self.scan_store.clear();
        if let Ok(index) = SessionIndex::new(&self.path) {
            let _ = index.clear_recovery_continuation(
                &self.scope,
                &RecoveryWorkBudget::new(Duration::from_secs(1)),
            );
        }
    }

    fn step(
        &mut self,
        cancel: &CancellationToken,
        operation_deadline: Instant,
    ) -> Option<PreparationStep> {
        let deadline = operation_deadline.min(Instant::now() + Duration::from_secs(2));
        if cancel.is_cancelled() {
            return None;
        }
        if self.scan.is_none() && self.seal.as_ref()?.check_current().is_err() {
            self.scan = Some(self.seal.as_ref()?.begin_verification().ok()?);
        }
        if let Some(scan) = &mut self.scan {
            return match scan.step(deadline, || cancel.is_cancelled()) {
                Ok(true) => {
                    self.seal = Some(self.scan.take()?.finish().ok()?);
                    Some(PreparationStep::Pending)
                }
                Ok(false) => Some(PreparationStep::Pending),
                Err(talos_session::history_page::HistoryPageError::Stopped)
                    if !cancel.is_cancelled() =>
                {
                    Some(PreparationStep::Expired)
                }
                Err(_) => None,
            };
        }
        if self.resolved {
            return Some(PreparationStep::Complete);
        }
        let stopped = cancel.clone();
        let budget = RecoveryWorkBudget::with_cancellation_check(
            deadline.saturating_duration_since(Instant::now()),
            move || stopped.is_cancelled(),
        );
        let mut index = SessionIndex::new(&self.path).ok()?;
        let retry = || {
            (!cancel.is_cancelled()
                && budget.deadline_interrupted()
                && self
                    .seal
                    .as_ref()
                    .is_some_and(|seal| seal.check_current().is_ok()))
            .then_some(PreparationStep::Expired)
        };
        let cursor = match index.recovery_build_cursor(&self.scope, &budget) {
            Ok(Some(cursor)) => cursor,
            Ok(None) => return None,
            Err(_) => return retry(),
        };
        if !cursor.complete() {
            let page = match self
                .source
                .read_page(cursor.next_offset(), deadline, || cancel.is_cancelled())
            {
                Ok(page) => page,
                Err(talos_session::history_page::HistoryPageError::Stopped)
                    if !cancel.is_cancelled() =>
                {
                    return Some(PreparationStep::Expired);
                }
                Err(_) => return None,
            };
            #[cfg(test)]
            if let Some(hook) = self.after_page_read.take() {
                hook();
            }
            // Never cache a page read across file versions, even if the original
            // prefix is restored before the next continuation attempt.
            self.seal.as_ref()?.check_current().ok()?;
            let mut parent = self.parent.clone();
            for entry in &page.entries {
                if entry.parent_id != parent {
                    return None;
                }
                parent = Some(entry.id.clone());
            }
            let sources: Vec<_> = page
                .entries
                .iter()
                .enumerate()
                .map(|(i, entry)| RecoveryIndexSource {
                    entry,
                    terminal: None,
                    ordinal: cursor.next_ordinal() + i as u64,
                    record_range: Some(page.record_ranges[i].clone()),
                })
                .collect();
            if index
                .advance_recovery_build(
                    &self.scope,
                    &cursor,
                    RecoveryBuildPage {
                        next_offset: page.next_offset,
                        scanned_entries: page.entries.len() as u64,
                        sources: &sources,
                    },
                    &budget,
                )
                .is_err()
            {
                return retry();
            }
            self.seal.as_ref()?.check_current().ok()?;
            self.parent = parent;
            return Some(PreparationStep::Pending);
        }
        if !self.resolved {
            self.resolved = match index.resolve_recovery_pending(&self.scope, &self.source, &budget)
            {
                Ok(resolved) => resolved,
                Err(_) => return retry(),
            };
            // Resolution rereads source records internally. If its transaction
            // committed across a version change, discard this whole preparation.
            self.seal.as_ref()?.check_current().ok()?;
            return Some(PreparationStep::Pending);
        }
        Some(PreparationStep::Pending)
    }
}

tokio::task_local! {
    pub(crate) static HISTORY: Option<History>;
}
impl History {
    /// Prepare within one bounded host operation, retaining completed pages on timeout.
    pub(crate) async fn prepare_cached(
        session: &DurableSession,
        cancel: &CancellationToken,
        pending: &tokio::sync::Mutex<Option<Preparation>>,
    ) -> Option<Self> {
        Self::prepare_until(
            session,
            cancel,
            pending,
            Instant::now() + Duration::from_secs(10),
        )
        .await
    }

    async fn prepare_until(
        session: &DurableSession,
        cancel: &CancellationToken,
        pending: &tokio::sync::Mutex<Option<Preparation>>,
        deadline: Instant,
    ) -> Option<Self> {
        let mut pending = tokio::select! {
            _ = cancel.cancelled() => return None,
            result = tokio::time::timeout_at(tokio::time::Instant::from_std(deadline), pending.lock()) => result.ok()?,
        };
        if pending
            .as_ref()
            .is_some_and(|work| !session.owns_recovery_source(&work.source))
        {
            *pending = None;
        }
        let mut work = pending
            .take()
            .or_else(|| Preparation::begin(session, cancel, deadline))?;
        loop {
            if cancel.is_cancelled() {
                work.retire();
                return None;
            }
            if Instant::now() >= deadline {
                // Windows stamps own a no-write/delete lease. Never retain it
                // between turns: normal persistence must be able to append.
                // Persisted preparation metadata is revalidated on the next attempt.
                #[cfg(not(windows))]
                {
                    *pending = Some(work);
                }
                #[cfg(windows)]
                drop(work);
                return None;
            }
            match work.step(cancel, deadline) {
                Some(PreparationStep::Complete) => {
                    work.retire();
                    break;
                }
                Some(PreparationStep::Pending) => {
                    if work.persist(cancel, deadline).is_none() {
                        work.retire();
                        return None;
                    }
                    tokio::task::yield_now().await;
                }
                Some(PreparationStep::Expired) => {
                    // Windows stamps own a no-write/delete lease. Never retain it
                    // between turns: normal persistence must be able to append.
                    // Persisted preparation metadata is revalidated on the next attempt.
                    #[cfg(not(windows))]
                    {
                        *pending = Some(work);
                    }
                    #[cfg(windows)]
                    drop(work);
                    return None;
                }
                None => {
                    work.retire();
                    return None;
                }
            }
        }
        Some(Self {
            path: work.path,
            scope: work.scope,
            owner: session.id().to_string(),
            cancel: cancel.clone(),
            seal: work.seal?,
            state: Mutex::new(QueryState::default()),
        })
    }

    #[cfg(test)]
    pub(crate) async fn prepare(
        session: &DurableSession,
        cancel: &CancellationToken,
    ) -> Option<Self> {
        Self::prepare_cached(session, cancel, &tokio::sync::Mutex::new(None)).await
    }

    pub(crate) fn scope_identity(&self) -> &str {
        &self.scope
    }

    pub(crate) fn reset_queries(&self) -> Option<()> {
        *self.state.lock().ok()? = QueryState::default();
        Some(())
    }

    pub(crate) fn cancelled(&self) -> bool {
        self.cancel.is_cancelled()
    }

    pub(crate) fn current(&self) -> bool {
        !self.cancel.is_cancelled() && self.seal.check_current().is_ok()
    }

    pub(crate) fn available(&self) -> bool {
        self.available_until(Instant::now() + Duration::from_secs(2))
    }

    pub(crate) fn available_until(&self, deadline: Instant) -> bool {
        if Instant::now() >= deadline {
            return false;
        }
        let stopped = self.cancel.clone();
        let budget = RecoveryWorkBudget::with_cancellation_check(
            deadline.saturating_duration_since(Instant::now()),
            move || stopped.is_cancelled(),
        );
        self.current()
            && SessionIndex::open_read_only(&self.path)
                .ok()
                .is_some_and(|index| {
                    index
                        .recovery_owner_scope_ready(&self.owner, &self.scope, &budget)
                        .unwrap_or(false)
                })
            && self.current()
            && Instant::now() < deadline
    }

    pub(crate) fn query(
        &self,
        query: Option<&str>,
        reference: Option<&str>,
        cursor: Option<&str>,
    ) -> Option<Value> {
        let stopped = self.cancel.clone();
        let budget =
            RecoveryWorkBudget::with_cancellation_check(Duration::from_secs(2), move || {
                stopped.is_cancelled()
            });
        if !self.current() {
            return None;
        }
        let index = SessionIndex::open_read_only(&self.path).ok()?;
        if !index
            .recovery_owner_scope_ready(&self.owner, &self.scope, &budget)
            .ok()?
        {
            return None;
        }
        let mut state = self.state.lock().ok()?;
        let result = if let Some(reference) = reference {
            if cursor.is_some() {
                return None;
            }
            let ordinal = state.ordinal(reference)?;
            let candidates = index
                .recovery_candidates(&self.scope, "", ordinal.checked_sub(1), 1, &budget)
                .ok()?;
            let candidate = candidates
                .first()
                .filter(|candidate| candidate.ordinal == ordinal)?;
            let text = index
                .recovery_candidate_text(
                    &self.scope,
                    candidate,
                    &self.seal,
                    talos_session::history_projection::RecoveryTextKind::Detail,
                    &budget,
                )
                .ok()??;
            json!({"status":"Ready","reference":reference,"text":text.text,"truncated":text.truncated})
        } else {
            let query = query?;
            let after = if let Some(cursor) = cursor {
                let (expected, expected_query, after) = state.cursor.as_ref()?;
                if cursor != expected || query != expected_query {
                    return None;
                }
                *after
            } else {
                None
            };
            let candidates = index
                .recovery_candidates(&self.scope, query, after, 20, &budget)
                .ok()?;
            let mut incomplete = candidates.len() == 20;
            let mut results = Vec::new();
            let mut last = after;
            // Bound all source/terminal record reads for this query together.
            let mut remaining = talos_session::history_page::HISTORY_PAGE_BYTES as u64;
            for candidate in candidates {
                let text = match index.recovery_candidate_text_with_source_budget(
                    &self.scope,
                    &candidate,
                    &self.seal,
                    talos_session::history_projection::RecoveryTextKind::Snippet,
                    &budget,
                    &mut remaining,
                ) {
                    Ok(Some(text)) => text,
                    Ok(None) => {
                        last = Some(candidate.ordinal);
                        continue;
                    }
                    Err(talos_session::sqlite::RecoveryReadError::PageBudgetExhausted) => {
                        incomplete = true;
                        break;
                    }
                    Err(_) => return None,
                };
                let reference = state.reference(candidate.ordinal);
                results.push(
                    json!({"reference":reference,"text":text.text,"truncated":text.truncated}),
                );
                if serde_json::to_vec(&results).ok()?.len() > 15_000 {
                    results.pop();
                    incomplete = true;
                    break;
                }
                last = Some(candidate.ordinal);
            }
            if incomplete && last == after {
                return None;
            }
            let next = incomplete.then(|| uuid::Uuid::new_v4().to_string());
            state.cursor = next
                .as_ref()
                .map(|token| (token.clone(), query.to_owned(), last));
            json!({"status":"Ready","results":results,"complete":!incomplete,"next_cursor":next})
        };
        if !self.current()
            || !index
                .recovery_owner_scope_ready(&self.owner, &self.scope, &budget)
                .ok()?
        {
            return None;
        }
        (serde_json::to_vec(&result).ok()?.len() <= 16_384).then_some(result)
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    #[tokio::test]
    async fn cancelled_preparation_task_abort_retires_saved_record() {
        let root = tempfile::tempdir().expect("root");
        let session = DurableSession::open_or_create(root.path(), "abort-scan").expect("owner");
        session
            .commit_turn(
                "seed",
                &[talos_core::message::Message::User {
                    content: "fact".into(),
                }],
                &Default::default(),
            )
            .expect("commit");
        let cancel = CancellationToken::new();
        let mut work =
            Preparation::begin(&session, &cancel, Instant::now() + Duration::from_secs(2))
                .expect("begin");
        work.scan_store.receipt = Some(
            session
                .save_recovery_scan(
                    work.scan.as_ref().expect("scan"),
                    &RecoveryWorkBudget::new(Duration::from_secs(2)),
                )
                .expect("save"),
        );
        let (ready, reached) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(async move {
            let _owned = work;
            ready.send(()).expect("ready");
            std::future::pending::<()>().await;
        });
        reached.await.expect("task owns preparation");
        cancel.cancel();
        task.abort();
        assert!(task.await.expect_err("aborted").is_cancelled());
        assert!(
            session
                .restore_recovery_scan(&RecoveryWorkBudget::new(Duration::from_secs(2)))
                .expect("check cleanup")
                .is_none()
        );
    }

    #[tokio::test]
    async fn initial_scan_reopens_from_persisted_page_and_retires_record() {
        if let Some(root) = std::env::var_os("TALOS_TEST_INITIAL_SCAN_ROOT") {
            let root = std::path::PathBuf::from(root);
            let parent: u32 = std::env::var("TALOS_TEST_INITIAL_SCAN_PARENT")
                .expect("parent")
                .parse()
                .expect("pid");
            assert_ne!(parent, std::process::id());
            let cancel = CancellationToken::new();
            let deadline = Instant::now() + Duration::from_secs(10);
            let reopened = DurableSession::open_existing(&root, "initial-scan")
                .expect("open")
                .expect("owner");
            let restored = Preparation::begin(&reopened, &cancel, deadline).expect("restore");
            assert_eq!(
                restored.scan.as_ref().expect("scan").completed_bytes(),
                2 * 1024 * 1024
            );
            assert!(
                restored.scan_store.receipt.is_some(),
                "restore carries exact cleanup authority"
            );
            let pending = tokio::sync::Mutex::new(Some(restored));
            let history = History::prepare_cached(&reopened, &cancel, &pending)
                .await
                .expect("finish");
            assert!(
                !history.query(Some("29"), None, None).expect("query")["results"]
                    .as_array()
                    .expect("array")
                    .is_empty()
            );
            assert!(
                reopened
                    .restore_recovery_scan(&RecoveryWorkBudget::new(Duration::from_secs(2)))
                    .expect("retired")
                    .is_none()
            );
            println!("INITIAL_SCAN_RESUMED_NONZERO");
            return;
        }
        let root = tempfile::tempdir().expect("root");
        let session = DurableSession::open_or_create(root.path(), "initial-scan").expect("owner");
        let raw: Vec<_> = (0..30)
            .map(|i| talos_core::message::Message::User {
                content: format!("initialscan fact {i} {}", "x".repeat(100_000)),
            })
            .collect();
        session
            .commit_turn("seed", &raw, &Default::default())
            .expect("commit");
        let cancel = CancellationToken::new();
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut work = Preparation::begin(&session, &cancel, deadline).expect("begin");
        for _ in 0..32 {
            assert_eq!(work.step(&cancel, deadline), Some(PreparationStep::Pending));
            work.persist(&cancel, deadline).expect("bounded persist");
        }
        assert_eq!(work.scan_store.saved_bytes, 2 * 1024 * 1024);
        assert!(
            work.seal.is_none(),
            "partial scan does not authorize results"
        );
        drop(work);
        drop(session);
        let mut command =
            tokio::process::Command::new(std::env::current_exe().expect("test binary"));
        command.args(["--exact", "checkpoint_history::tests::initial_scan_reopens_from_persisted_page_and_retires_record", "--nocapture"])
            .env("TALOS_TEST_INITIAL_SCAN_ROOT", root.path())
            .env("TALOS_TEST_INITIAL_SCAN_PARENT", std::process::id().to_string())
            .kill_on_drop(true);
        let output = tokio::time::timeout(Duration::from_secs(30), command.output())
            .await
            .expect("child deadline")
            .expect("child launch");
        assert!(
            output.status.success(),
            "child failed: {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("INITIAL_SCAN_RESUMED_NONZERO"));
    }

    #[tokio::test]
    async fn invalid_persisted_version_rebuilds_and_cutoff_mismatch_is_not_saved() {
        let root = tempfile::tempdir().expect("root");
        let session = DurableSession::open_or_create(root.path(), "bad-resume").expect("session");
        session
            .commit_turn(
                "first",
                &[talos_core::message::Message::User {
                    content: "original fact".into(),
                }],
                &Default::default(),
            )
            .expect("commit");
        let cancel = CancellationToken::new();
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut work = Preparation::begin(&session, &cancel, deadline).expect("begin");
        while work.seal.is_none() {
            assert_eq!(work.step(&cancel, deadline), Some(PreparationStep::Pending));
        }
        work.persist(&cancel, deadline).expect("persist");
        let index = SessionIndex::new(&work.path).expect("index");
        let budget = RecoveryWorkBudget::new(Duration::from_secs(2));
        let owner = session.id().to_string();
        let saved = index
            .recovery_continuation(&owner, &budget)
            .expect("load")
            .expect("saved");
        let mut json = serde_json::to_value(&saved).expect("json");
        json["prefix"]["cutoff"] = json!(saved.prefix.cutoff() + 1);
        let bad = serde_json::from_value(json).expect("bad cutoff");
        assert!(
            index
                .save_recovery_continuation(&owner, &bad, &budget)
                .is_err()
        );
        let mut json = serde_json::to_value(&saved).expect("json");
        json["prefix"]["version"] = json!(255);
        let bad = serde_json::from_value(json).expect("future version");
        index
            .save_recovery_continuation(&owner, &bad, &budget)
            .expect("untrusted version stored");
        let old_scope = work.scope.clone();
        drop(work);
        let rebuilt = Preparation::begin(&session, &cancel, deadline)
            .expect("bad record must not block new build");
        assert_ne!(rebuilt.scope, old_scope);
        let pending = tokio::sync::Mutex::new(Some(rebuilt));
        assert!(
            History::prepare_cached(&session, &cancel, &pending)
                .await
                .is_some()
        );
    }

    #[tokio::test]
    async fn crash_between_page_commit_and_metadata_save_rebuilds_instead_of_reusing_parent() {
        let root = tempfile::tempdir().expect("root");
        let session = DurableSession::open_or_create(root.path(), "crash-gap").expect("session");
        session
            .commit_turn(
                "first",
                &[talos_core::message::Message::User {
                    content: "crashgap fact".into(),
                }],
                &Default::default(),
            )
            .expect("commit");
        let cancel = CancellationToken::new();
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut work = Preparation::begin(&session, &cancel, deadline).expect("begin");
        while work.seal.is_none() {
            assert_eq!(work.step(&cancel, deadline), Some(PreparationStep::Pending));
        }
        work.persist(&cancel, deadline).expect("save zero cursor");
        let old_scope = work.scope.clone();
        assert_eq!(work.step(&cancel, deadline), Some(PreparationStep::Pending));
        assert!(work.parent.is_some());
        // Simulate a process exit before saving metadata for the committed page.
        drop(work);
        let rebuilt = Preparation::begin(&session, &cancel, deadline).expect("rebuild mismatch");
        assert_ne!(rebuilt.scope, old_scope);
        assert!(rebuilt.parent.is_none());
        let pending = tokio::sync::Mutex::new(Some(rebuilt));
        let history = History::prepare_cached(&session, &cancel, &pending)
            .await
            .expect("rebuilt history");
        assert_eq!(
            history.query(Some("crashgap"), None, None).expect("query")["results"][0]["text"],
            "crashgap fact"
        );
    }

    #[test]
    fn preparation_restart_child() {
        let Ok(root) = std::env::var("TALOS_TEST_RECOVERY_RESTART_ROOT") else {
            return;
        };
        let expected = std::env::var("TALOS_TEST_RECOVERY_RESTART_SCOPE").expect("scope");
        tokio::runtime::Runtime::new()
            .expect("runtime")
            .block_on(async {
                let session = DurableSession::open_or_create(
                    std::path::Path::new(&root),
                    "restart-preparation",
                )
                .expect("child reopen");
                let cancel = CancellationToken::new();
                let work =
                    Preparation::begin(&session, &cancel, Instant::now() + Duration::from_secs(10))
                        .expect("child restore");
                assert_eq!(work.scope, expected);
                assert!(work.parent.is_some());
                assert!(work.seal.is_none());
                assert_eq!(
                    work.scan
                        .as_ref()
                        .expect("verification scan")
                        .completed_bytes(),
                    2 * 1024 * 1024,
                    "independent process resumes prefix verification"
                );
                let pending = tokio::sync::Mutex::new(Some(work));
                let history = History::prepare_cached(&session, &cancel, &pending)
                    .await
                    .expect("child resume");
                assert_eq!(history.scope_identity(), expected);
                assert!(
                    history.query(Some("29"), None, None).expect("last fact")["results"][0]["text"]
                        .as_str()
                        .expect("text")
                        .starts_with("restartfact 29")
                );
            });
    }

    #[tokio::test]
    async fn changed_index_expectation_retires_only_observed_scan() {
        let root = tempfile::tempdir().expect("root");
        let session =
            DurableSession::open_or_create(root.path(), "changed-expectation").expect("owner");
        session
            .commit_turn(
                "seed",
                &[talos_core::message::Message::User {
                    content: "fact ".repeat(600_000),
                }],
                &Default::default(),
            )
            .expect("seed");
        let cancel = CancellationToken::new();
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut work = Preparation::begin(&session, &cancel, deadline).expect("begin");
        while work.seal.is_none() {
            assert_eq!(work.step(&cancel, deadline), Some(PreparationStep::Pending));
        }
        work.persist(&cancel, deadline).expect("index continuation");
        let budget = RecoveryWorkBudget::new(Duration::from_secs(2));
        let index = SessionIndex::new(&work.path).expect("index");
        let mut saved = index
            .recovery_continuation(&session.id().to_string(), &budget)
            .expect("read")
            .expect("saved");
        let mut verifying = Preparation::begin(&session, &cancel, deadline).expect("verify");
        for _ in 0..32 {
            assert_eq!(
                verifying.step(&cancel, deadline),
                Some(PreparationStep::Pending)
            );
        }
        verifying.persist(&cancel, deadline).expect("save scan");
        let mut changed = serde_json::to_value(&saved.prefix).expect("encode");
        changed["digest"][0] = serde_json::json!(changed["digest"][0].as_u64().expect("byte") ^ 1);
        saved.prefix = serde_json::from_value(changed).expect("decode");
        index
            .save_recovery_continuation(&session.id().to_string(), &saved, &budget)
            .expect("changed cache expectation");
        let mut replacement = Preparation::begin(&session, &cancel, deadline).expect("restart");
        assert_eq!(
            replacement.scan.as_ref().expect("scan").completed_bytes(),
            0
        );
        for _ in 0..32 {
            assert_eq!(
                replacement.step(&cancel, deadline),
                Some(PreparationStep::Pending)
            );
        }
        replacement
            .persist(&cancel, deadline)
            .expect("new expectation can save");
        verifying.scan_store.clear();
        assert!(
            session
                .restore_recovery_scan(&budget)
                .expect("read replacement")
                .expect("old cleanup preserved new record")
                .scan
                .verifies_expectation(&saved.prefix)
        );
        // A forged cache digest still cannot finish verification or issue a seal.
        while replacement.step(&cancel, deadline) == Some(PreparationStep::Pending) {}
        assert!(replacement.seal.is_none());
    }

    #[tokio::test]
    async fn persisted_preparation_reopens_and_resumes_same_index_scope() {
        let root = tempfile::tempdir().expect("root");
        let session =
            DurableSession::open_or_create(root.path(), "restart-preparation").expect("session");
        let raw: Vec<_> = (0..30)
            .map(|i| talos_core::message::Message::User {
                content: format!("restartfact {i} {}", "x".repeat(100_000)),
            })
            .collect();
        session
            .commit_turn("first", &raw, &Default::default())
            .expect("commit");
        let cancel = CancellationToken::new();
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut work = Preparation::begin(&session, &cancel, deadline).expect("begin");
        while work.seal.is_none() {
            assert_eq!(work.step(&cancel, deadline), Some(PreparationStep::Pending));
            if work.scan.is_some() {
                work.persist(&cancel, deadline)
                    .expect("persist initial scan");
            }
        }
        assert_eq!(work.step(&cancel, deadline), Some(PreparationStep::Pending));
        work.persist(&cancel, deadline)
            .expect("persist continuation");
        let scope = work.scope.clone();
        let parent = work.parent.clone();
        assert!(parent.is_some());
        drop(work);
        // Interrupt index-prefix verification after one completed page. The
        // child must reuse this host-issued progress, not restart at zero.
        let mut verification = Preparation::begin(&session, &cancel, deadline).expect("verify");
        for _ in 0..32 {
            assert_eq!(
                verification.step(&cancel, deadline),
                Some(PreparationStep::Pending)
            );
        }
        verification
            .persist(&cancel, deadline)
            .expect("save verification");
        assert!(verification.seal.is_none());
        drop(verification);
        drop(session);
        let mut child = std::process::Command::new(std::env::current_exe().expect("test binary"))
            .args([
                "--exact",
                "checkpoint_history::tests::preparation_restart_child",
                "--nocapture",
            ])
            .env("TALOS_TEST_RECOVERY_RESTART_ROOT", root.path())
            .env("TALOS_TEST_RECOVERY_RESTART_SCOPE", &scope)
            .spawn()
            .expect("spawn independent recovery process");
        let until = Instant::now() + Duration::from_secs(30);
        loop {
            if let Some(status) = child.try_wait().expect("child status") {
                assert!(status.success());
                break;
            }
            if Instant::now() >= until {
                child.kill().expect("kill timed out child");
                let _ = child.wait();
                panic!("child recovery timeout");
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        // The successful child retires restart metadata. Re-create it for the
        // same-process verification below from a fresh bounded preparation.
        let session = DurableSession::open_or_create(root.path(), "restart-preparation")
            .expect("reopen fixture");
        let mut work = Preparation::begin(&session, &cancel, deadline).expect("fresh begin");
        while work.seal.is_none() {
            assert_eq!(work.step(&cancel, deadline), Some(PreparationStep::Pending));
        }
        assert_eq!(work.step(&cancel, deadline), Some(PreparationStep::Pending));
        work.persist(&cancel, deadline).expect("save again");
        let scope = work.scope.clone();
        let parent = work.parent.clone();
        drop(work);
        drop(session);
        let reopened =
            DurableSession::open_or_create(root.path(), "restart-preparation").expect("reopen");
        let restored = Preparation::begin(&reopened, &cancel, deadline).expect("restored");
        assert_eq!(restored.scope, scope);
        assert_eq!(restored.parent, parent);
        assert!(
            restored.seal.is_none(),
            "persisted digest is not a verified seal"
        );
        let pending = tokio::sync::Mutex::new(Some(restored));
        let history = History::prepare_cached(&reopened, &cancel, &pending)
            .await
            .expect("resumed history");
        assert_eq!(history.scope_identity(), scope);
        assert!(
            history.query(Some("29"), None, None).expect("last fact")["results"][0]["text"]
                .as_str()
                .expect("text")
                .starts_with("restartfact 29")
        );
    }

    #[tokio::test]
    async fn mutation_then_restore_during_page_read_discards_preparation() {
        let root = tempfile::tempdir().expect("root");
        let session = DurableSession::open_or_create(root.path(), "mutation").expect("session");
        session
            .commit_turn(
                "prior",
                &[talos_core::message::Message::User {
                    content: "originalfact".into(),
                }],
                &Default::default(),
            )
            .expect("commit");
        let cancel = CancellationToken::new();
        let mut work =
            Preparation::begin(&session, &cancel, Instant::now() + Duration::from_secs(2))
                .expect("begin");
        while work.seal.is_none() {
            assert_eq!(
                work.step(&cancel, Instant::now() + Duration::from_secs(2)),
                Some(PreparationStep::Pending)
            );
        }
        let path = session.file_path().to_path_buf();
        let original = std::fs::read(&path).expect("original");
        work.after_page_read = Some(Box::new(move || {
            std::fs::write(&path, b"temporary corrupt source").expect("mutate");
            std::fs::write(&path, original).expect("restore bytes");
        }));
        let scope = work.scope.clone();
        let index_path = work.path.clone();
        let pending = tokio::sync::Mutex::new(Some(work));
        assert!(
            History::prepare_cached(&session, &cancel, &pending)
                .await
                .is_none()
        );
        assert!(pending.lock().await.is_none());
        let index = SessionIndex::open_read_only(&index_path).expect("index");
        let budget = RecoveryWorkBudget::new(Duration::from_secs(2));
        assert_eq!(
            index
                .recovery_build_cursor(&scope, &budget)
                .expect("cursor")
                .expect("scope")
                .next_ordinal(),
            0
        );
        assert!(
            index
                .recovery_candidates(&scope, "originalfact", None, 20, &budget)
                .is_err()
        );
    }

    #[tokio::test]
    async fn sealed_preparation_survives_real_append_without_extending_captured_cutoff() {
        let root = tempfile::tempdir().expect("root");
        let session = DurableSession::open_or_create(root.path(), "append").expect("session");
        let raw: Vec<_> = (0..30)
            .map(|i| talos_core::message::Message::User {
                content: format!("oldfact {i} {}", "x".repeat(100_000)),
            })
            .collect();
        session
            .commit_turn("first", &raw, &Default::default())
            .expect("commit");
        let cancel = CancellationToken::new();
        let mut work =
            Preparation::begin(&session, &cancel, Instant::now() + Duration::from_secs(2))
                .expect("begin");
        while work.seal.is_none() {
            assert_eq!(
                work.step(&cancel, Instant::now() + Duration::from_secs(2)),
                Some(PreparationStep::Pending)
            );
        }
        assert_eq!(
            work.step(&cancel, Instant::now() + Duration::from_secs(2)),
            Some(PreparationStep::Pending)
        );
        assert!(work.parent.is_some());
        let scope = work.scope.clone();
        session
            .commit_turn(
                "append",
                &[talos_core::message::Message::User {
                    content: "newfact outside cutoff".into(),
                }],
                &Default::default(),
            )
            .expect("append");
        let pending = tokio::sync::Mutex::new(Some(work));
        let history = History::prepare_cached(&session, &cancel, &pending)
            .await
            .expect("revalidated continuation");
        assert_eq!(history.scope_identity(), scope);
        assert!(
            !history
                .query(Some("oldfact"), None, None)
                .expect("old facts")["results"]
                .as_array()
                .expect("array")
                .is_empty()
        );
        assert!(
            history
                .query(Some("newfact"), None, None)
                .expect("query cutoff")["results"]
                .as_array()
                .expect("array")
                .is_empty()
        );
    }

    #[tokio::test]
    async fn expired_seal_step_can_resume_but_cancelled_step_cannot() {
        let root = tempfile::tempdir().expect("root");
        let session = DurableSession::open_or_create(root.path(), "seal-timeout").expect("session");
        session
            .commit_turn(
                "prior",
                &[talos_core::message::Message::User {
                    content: "deadline fact".into(),
                }],
                &Default::default(),
            )
            .expect("commit");
        let cancel = CancellationToken::new();
        let mut work =
            Preparation::begin(&session, &cancel, Instant::now() + Duration::from_secs(2))
                .expect("begin");
        let original_scope = work.scope.clone();
        assert_eq!(
            work.step(&cancel, Instant::now()),
            Some(PreparationStep::Expired)
        );
        assert!(
            work.parent.is_none(),
            "expired SQLite cursor read must not advance parent"
        );
        assert_eq!(work.scope, original_scope);
        let cancelled = CancellationToken::new();
        cancelled.cancel();
        assert_eq!(
            work.step(&cancelled, Instant::now() + Duration::from_secs(2)),
            None
        );
        // Both failures leave the seal scan unchanged; only timeout is resumable by the host.
        loop {
            match work.step(&cancel, Instant::now() + Duration::from_secs(2)) {
                Some(PreparationStep::Complete) => break,
                Some(PreparationStep::Pending) => {}
                other => panic!("unexpected {other:?}"),
            }
        }
        assert!(work.seal.expect("seal").check_current().is_ok());
    }

    #[tokio::test]
    async fn preparation_waiters_cancel_or_expire_without_consuming_pending_work() {
        let root = tempfile::tempdir().expect("root");
        let session = DurableSession::open_or_create(root.path(), "waiters").expect("session");
        session
            .commit_turn(
                "prior",
                &[talos_core::message::Message::User {
                    content: "retained fact".into(),
                }],
                &Default::default(),
            )
            .expect("commit");
        let cancel = CancellationToken::new();
        let work = Preparation::begin(&session, &cancel, Instant::now() + Duration::from_secs(2))
            .expect("begin");
        let scope = work.scope.clone();
        let pending = tokio::sync::Mutex::new(Some(work));
        let guard = pending.lock().await;
        assert!(
            History::prepare_until(
                &session,
                &cancel,
                &pending,
                Instant::now() + Duration::from_millis(1)
            )
            .await
            .is_none()
        );
        let cancelled = CancellationToken::new();
        cancelled.cancel();
        assert!(
            tokio::time::timeout(
                Duration::from_secs(1),
                History::prepare_cached(&session, &cancelled, &pending)
            )
            .await
            .expect("cancellation must not wait for mutex")
            .is_none()
        );
        assert_eq!(guard.as_ref().expect("retained work").scope, scope);
        drop(guard);
        let history = History::prepare_cached(&session, &cancel, &pending)
            .await
            .expect("resume after waiters");
        assert_eq!(history.scope_identity(), scope);
        assert!(history.query(Some("retained"), None, None).is_some());
    }

    #[tokio::test]
    async fn host_preparation_resumes_completed_page_and_rejects_changed_source() {
        let root = tempfile::tempdir().expect("root");
        let session = DurableSession::open_or_create(root.path(), "resume").expect("session");
        let raw: Vec<_> = (0..30)
            .map(|i| talos_core::message::Message::User {
                content: format!("resume fact {i} {}", "x".repeat(100_000)),
            })
            .collect();
        session
            .commit_turn("prior", &raw, &Default::default())
            .expect("commit");
        let cancel = CancellationToken::new();
        let mut work =
            Preparation::begin(&session, &cancel, Instant::now() + Duration::from_secs(10))
                .expect("begin");
        assert_eq!(
            work.step(&cancel, Instant::now() + Duration::from_secs(10)),
            Some(PreparationStep::Pending)
        );
        let scope = work.scope.clone();
        let reader = SessionIndex::open_read_only(&work.path).expect("read-only partial index");
        let budget = RecoveryWorkBudget::new(Duration::from_secs(2));
        assert!(
            !reader
                .recovery_owner_scope_ready(&session.id().to_string(), &scope, &budget)
                .expect("readiness")
        );
        assert!(
            reader
                .recovery_candidates(&scope, "resume", None, 20, &budget)
                .is_err(),
            "partial coverage must not appear as complete search"
        );
        drop(reader);
        let pending = tokio::sync::Mutex::new(Some(work));
        assert!(
            History::prepare_until(&session, &cancel, &pending, Instant::now())
                .await
                .is_none()
        );
        assert_eq!(
            pending
                .lock()
                .await
                .as_ref()
                .expect("retained timeout progress")
                .scope,
            scope
        );
        let history = History::prepare_cached(&session, &cancel, &pending)
            .await
            .expect("resumed");
        assert_eq!(history.scope_identity(), scope);
        let result = history.query(Some("29"), None, None).expect("last fact");
        assert!(
            result["results"][0]["text"]
                .as_str()
                .expect("text")
                .starts_with("resume fact 29")
        );
        let mut work =
            Preparation::begin(&session, &cancel, Instant::now() + Duration::from_secs(10))
                .expect("begin again");
        assert_eq!(
            work.step(&cancel, Instant::now() + Duration::from_secs(10)),
            Some(PreparationStep::Pending)
        );
        let bytes = std::fs::read(session.file_path()).expect("bytes");
        std::fs::write(session.file_path(), bytes).expect("change source version");
        let pending = tokio::sync::Mutex::new(Some(work));
        assert!(
            History::prepare_cached(&session, &cancel, &pending)
                .await
                .is_none()
        );
        assert!(pending.lock().await.is_none());
    }

    #[test]
    fn references_require_canonical_format_and_current_discovery_secret() {
        let state = QueryState::default();
        let reference = state.reference(23);
        assert_eq!(state.ordinal(&reference), Some(23));
        assert_eq!(QueryState::default().ordinal(&reference), None);
        for malformed in [
            reference.to_uppercase(),
            format!("{reference}x"),
            reference[1..].into(),
            "é".repeat(41),
        ] {
            assert_eq!(state.ordinal(&malformed), None);
        }
    }

    #[tokio::test]
    async fn source_budget_pages_retry_unread_candidate_without_skipping() {
        let root = tempfile::tempdir().expect("root");
        let session = DurableSession::open_or_create(root.path(), "byte-pages").expect("session");
        let raw: Vec<_> = (0..10)
            .map(|i| talos_core::message::Message::User {
                content: format!("budgetfact {i} {}", "x".repeat(300_000)),
            })
            .collect();
        session
            .commit_turn("turn", &raw, &Default::default())
            .expect("commit");
        let history = History::prepare(&session, &CancellationToken::new())
            .await
            .expect("host");
        let mut cursor = None;
        let mut seen = Vec::new();
        let mut pages = 0;
        loop {
            let page = history
                .query(Some("budgetfact"), None, cursor.as_deref())
                .expect("page");
            let results = page["results"].as_array().expect("results");
            assert!(!results.is_empty());
            for result in results {
                let text = result["text"].as_str().expect("text");
                seen.push(
                    text.split_whitespace()
                        .nth(1)
                        .expect("ordinal")
                        .parse::<usize>()
                        .expect("number"),
                );
            }
            pages += 1;
            assert!(pages <= 10, "cursor must advance");
            if page["complete"] == true {
                break;
            }
            cursor = Some(page["next_cursor"].as_str().expect("cursor").to_owned());
        }
        assert!(pages > 1, "source allowance must split the result page");
        assert_eq!(seen, (0..10).collect::<Vec<_>>());
    }

    #[tokio::test]
    async fn search_pages_preserve_order_and_reject_query_changes_and_replay() {
        let root = tempfile::tempdir().expect("root");
        let session = DurableSession::open_or_create(root.path(), "pages").expect("session");
        let raw: Vec<_> = (0..45)
            .map(|i| talos_core::message::Message::User {
                content: format!("paginated fact {i}"),
            })
            .collect();
        session
            .commit_turn("turn", &raw, &Default::default())
            .expect("commit");
        let history = History::prepare(&session, &CancellationToken::new())
            .await
            .expect("host");
        let first = history.query(Some("paginated"), None, None).expect("first");
        assert_eq!(first["results"].as_array().expect("results").len(), 20);
        let cursor = first["next_cursor"].as_str().expect("cursor");
        assert!(
            history
                .query(Some("different"), None, Some(cursor))
                .is_none()
        );
        let second = history
            .query(Some("paginated"), None, Some(cursor))
            .expect("second");
        assert!(
            history
                .query(Some("paginated"), None, Some(cursor))
                .is_none()
        );
        assert_eq!(second["results"][0]["text"], "paginated fact 20");
        let third = history
            .query(Some("paginated"), None, second["next_cursor"].as_str())
            .expect("third");
        assert_eq!(third["results"][0]["text"], "paginated fact 40");
        assert_eq!(third["results"].as_array().expect("results").len(), 5);
        assert_eq!(third["complete"], true);
        assert!(third["next_cursor"].is_null());
        let detail = history
            .query(None, third["results"][4]["reference"].as_str(), None)
            .expect("detail");
        assert_eq!(detail["text"], "paginated fact 44");
        let old_ref = first["results"][0]["reference"].as_str().expect("old ref");
        assert_eq!(
            history
                .query(None, Some(old_ref), None)
                .expect("old page detail")["text"],
            "paginated fact 0"
        );
        let mut forged = old_ref.as_bytes().to_vec();
        forged[15] = if forged[15] == b'0' { b'1' } else { b'0' };
        assert!(
            history
                .query(
                    None,
                    Some(std::str::from_utf8(&forged).expect("ascii")),
                    None
                )
                .is_none()
        );
        history.reset_queries().expect("rediscovery");
        assert!(history.query(None, Some(old_ref), None).is_none());
    }

    #[tokio::test]
    async fn next_host_preparation_removes_previous_scope_and_cancel_refuses_queries() {
        let root = tempfile::tempdir().expect("root");
        let session = DurableSession::open_or_create(root.path(), "scope").expect("session");
        session
            .commit_turn(
                "turn",
                &[talos_core::message::Message::User {
                    content: "fact".into(),
                }],
                &Default::default(),
            )
            .expect("commit");
        let cancel = CancellationToken::new();
        let first = History::prepare(&session, &cancel).await.expect("first");
        assert!(first.query(Some("fact"), None, None).is_some());
        let second = History::prepare(&session, &cancel).await.expect("second");
        let index = SessionIndex::open_read_only(&second.path).expect("index");
        let budget = RecoveryWorkBudget::new(Duration::from_secs(2));
        assert!(
            index
                .recovery_build_cursor(&first.scope, &budget)
                .expect("old cursor")
                .is_none()
        );
        assert!(
            index
                .recovery_candidates(&first.scope, "", None, 20, &budget)
                .expect("old entries")
                .is_empty()
        );
        assert!(
            first.query(Some("fact"), None, None).is_none(),
            "retired scope is unavailable, not complete empty results"
        );
        assert!(second.query(Some("fact"), None, None).is_some());
        cancel.cancel();
        assert!(second.query(Some("fact"), None, None).is_none());
    }
}

#[cfg(all(test, windows))]
mod windows_lease_tests {
    use super::*;

    #[tokio::test]
    async fn prepared_history_releases_source_before_next_durable_turn() {
        let root = tempfile::tempdir().expect("root");
        let session = DurableSession::open_or_create(root.path(), "live-history").expect("session");
        session
            .commit_turn(
                "seed",
                &[talos_core::message::Message::User {
                    content: "retained original constraint".into(),
                }],
                &Default::default(),
            )
            .expect("seed");
        let cancel = CancellationToken::new();
        let pending = tokio::sync::Mutex::new(None);
        let history = History::prepare_cached(&session, &cancel, &pending)
            .await
            .expect("real preparation");
        let result = history
            .query(Some("constraint"), None, None)
            .expect("query");
        assert!(
            result["results"]
                .as_array()
                .expect("results")
                .iter()
                .any(|item| item["text"]
                    .as_str()
                    .is_some_and(|text| text.contains("retained original constraint")))
        );
        assert!(
            std::fs::OpenOptions::new()
                .write(true)
                .open(session.file_path())
                .is_err()
        );
        cancel.cancel();
        assert!(history.query(Some("constraint"), None, None).is_none());
        // Cancellation revokes queries; dropping the owner releases source custody.
        drop(history);
        assert!(pending.lock().await.is_none());
        session
            .commit_turn(
                "next",
                &[talos_core::message::Message::User {
                    content: "normal continuation".into(),
                }],
                &Default::default(),
            )
            .expect("commit after recovery owner drops");
    }

    #[tokio::test]
    async fn expired_preparation_drops_pending_lease_before_normal_persistence() {
        let root = tempfile::tempdir().expect("root");
        let session =
            DurableSession::open_or_create(root.path(), "expired-lease").expect("session");
        session
            .commit_turn(
                "seed",
                &[talos_core::message::Message::User {
                    content: "original fact".into(),
                }],
                &Default::default(),
            )
            .expect("seed");
        let cancel = CancellationToken::new();
        let work = Preparation::begin(&session, &cancel, Instant::now() + Duration::from_secs(10))
            .expect("prepared lease");
        assert!(
            std::fs::OpenOptions::new()
                .write(true)
                .open(session.file_path())
                .is_err()
        );
        let pending = tokio::sync::Mutex::new(Some(work));
        assert!(
            History::prepare_until(&session, &cancel, &pending, Instant::now())
                .await
                .is_none()
        );
        assert!(
            pending.lock().await.is_none(),
            "expired preparation must not retain custody"
        );
        session
            .commit_turn(
                "after-timeout",
                &[talos_core::message::Message::User {
                    content: "normal continuation".into(),
                }],
                &Default::default(),
            )
            .expect("persistence after actual timeout path");
    }

    #[test]
    fn dropping_partial_preparation_releases_before_next_commit() {
        let root = tempfile::tempdir().expect("root");
        let session =
            DurableSession::open_or_create(root.path(), "lease-release").expect("session");
        session
            .commit_turn(
                "seed",
                &[talos_core::message::Message::User {
                    content: "original fact".into(),
                }],
                &Default::default(),
            )
            .expect("seed commit");
        let cancel = CancellationToken::new();
        let work = Preparation::begin(&session, &cancel, Instant::now() + Duration::from_secs(10))
            .expect("preparation with live source lease");
        // Both timeout exits drop this same owning value on Windows. Exercise
        // the release against real durable persistence, not metadata equality.
        drop(work);
        session
            .commit_turn(
                "next",
                &[talos_core::message::Message::User {
                    content: "normal continuation".into(),
                }],
                &Default::default(),
            )
            .expect("append after preparation drop");
    }
}

#[cfg(all(test, any(unix, windows)))]
mod persisted_preparation_tests {
    use super::*;

    #[test]
    fn host_preparation_restores_completed_scan_pages_and_cancellation_cleans_receipt() {
        let root = tempfile::tempdir().expect("root");
        let session = DurableSession::open_or_create(root.path(), "scan-hooks").expect("session");
        session
            .commit_turn(
                "seed",
                &[talos_core::message::Message::User {
                    content: "original evidence ".repeat(200_000),
                }],
                &Default::default(),
            )
            .expect("seed");
        let cancel = CancellationToken::new();
        let deadline = Instant::now() + Duration::from_secs(20);
        let mut first = Preparation::begin(&session, &cancel, deadline).expect("prepare");
        while first.scan_store.saved_bytes == 0 {
            assert_eq!(
                first.step(&cancel, deadline),
                Some(PreparationStep::Pending)
            );
            first
                .persist(&cancel, deadline)
                .expect("save completed page");
        }
        let saved = first.scan_store.saved_bytes;
        assert!(saved > 0);
        drop(first); // All old source leases are released; uncancelled progress survives.
        let second = Preparation::begin(&session, &cancel, deadline).expect("restore preparation");
        assert_eq!(second.scan_store.saved_bytes, saved);
        assert_eq!(
            second
                .scan
                .as_ref()
                .expect("resumed scan")
                .completed_bytes(),
            saved
        );
        cancel.cancel();
        drop(second); // Agent cancellation cleanup uses the exact restored receipt.
        assert!(
            session
                .restore_recovery_scan(&RecoveryWorkBudget::new(Duration::from_secs(2)))
                .expect("check cleanup")
                .is_none()
        );
        session
            .commit_turn(
                "next",
                &[talos_core::message::Message::User {
                    content: "normal continuation after cancelled preparation".into(),
                }],
                &Default::default(),
            )
            .expect("source leases released");
    }
}
