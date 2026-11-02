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
    source: talos_session::DurableRecoverySource,
    scan: talos_session::RecoveryPrefixScan,
    path: std::path::PathBuf,
    scope: String,
    parent: Option<String>,
    resolved: bool,
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
        let scope = uuid::Uuid::new_v4().to_string();
        index
            .replace_recovery_owner_scope(&session.id().to_string(), &scope, &budget)
            .ok()?;
        index
            .begin_recovery_build(&scope, source.source_end(), &budget)
            .ok()?;
        Some(Self {
            source,
            scan,
            path,
            scope,
            parent: None,
            resolved: false,
        })
    }
    fn step(
        &mut self,
        cancel: &CancellationToken,
        operation_deadline: Instant,
    ) -> Option<PreparationStep> {
        let deadline = operation_deadline.min(Instant::now() + Duration::from_secs(2));
        if self.resolved {
            return match self.scan.step(deadline, || cancel.is_cancelled()) {
                Ok(true) => Some(PreparationStep::Complete),
                Ok(false) => Some(PreparationStep::Pending),
                Err(talos_session::history_page::HistoryPageError::Stopped)
                    if !cancel.is_cancelled() =>
                {
                    Some(PreparationStep::Expired)
                }
                Err(_) => None,
            };
        }
        let stopped = cancel.clone();
        let budget = RecoveryWorkBudget::with_cancellation_check(
            deadline.saturating_duration_since(Instant::now()),
            move || stopped.is_cancelled(),
        );
        let mut index = SessionIndex::new(&self.path).ok()?;
        let cursor = index.recovery_build_cursor(&self.scope, &budget).ok()??;
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
            index
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
                .ok()?;
            self.parent = parent;
            return Some(PreparationStep::Pending);
        }
        if !self.resolved {
            self.resolved = index
                .resolve_recovery_pending(&self.scope, &self.source, &budget)
                .ok()?;
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
                return None;
            }
            if Instant::now() >= deadline {
                *pending = Some(work);
                return None;
            }
            match work.step(cancel, deadline) {
                Some(PreparationStep::Complete) => break,
                Some(PreparationStep::Pending) => tokio::task::yield_now().await,
                Some(PreparationStep::Expired) => {
                    *pending = Some(work);
                    return None;
                }
                None => return None,
            }
        }
        Some(Self {
            path: work.path,
            scope: work.scope,
            owner: session.id().to_string(),
            cancel: cancel.clone(),
            seal: work.scan.finish().ok()?,
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
        let stopped = self.cancel.clone();
        let budget =
            RecoveryWorkBudget::with_cancellation_check(Duration::from_secs(2), move || {
                stopped.is_cancelled()
            });
        self.current()
            && SessionIndex::open_read_only(&self.path)
                .ok()
                .is_some_and(|index| {
                    index
                        .recovery_owner_scope_ready(&self.owner, &self.scope, &budget)
                        .unwrap_or(false)
                })
            && self.current()
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
        while !work.resolved {
            assert_eq!(
                work.step(&cancel, Instant::now() + Duration::from_secs(2)),
                Some(PreparationStep::Pending)
            );
        }
        assert_eq!(
            work.step(&cancel, Instant::now()),
            Some(PreparationStep::Expired)
        );
        let cancelled = CancellationToken::new();
        cancelled.cancel();
        assert_eq!(
            work.step(&cancelled, Instant::now() + Duration::from_secs(2)),
            None
        );
        // Both failures leave the seal scan unchanged; only timeout is resumable by the host.
        assert_eq!(
            work.step(&cancel, Instant::now() + Duration::from_secs(2)),
            Some(PreparationStep::Complete)
        );
        assert!(work.scan.finish().is_ok());
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
