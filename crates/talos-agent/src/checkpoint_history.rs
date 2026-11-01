//! Host-prepared durable history queries, scoped to one configured execution.
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    sync::Mutex,
    time::{Duration, Instant},
};
use talos_session::sqlite::{
    RecoveryBuildPage, RecoveryCandidate, RecoveryIndexSource, RecoveryWorkBudget,
};
use talos_session::{DurableSession, RecoveryPrefixSeal, SessionIndex};
use tokio_util::sync::CancellationToken;

pub(crate) struct History {
    path: std::path::PathBuf,
    scope: String,
    owner: String,
    cancel: CancellationToken,
    seal: RecoveryPrefixSeal,
    refs: Mutex<BTreeMap<String, RecoveryCandidate>>,
}
tokio::task_local! {
    pub(crate) static HISTORY: Option<History>;
}
impl History {
    /// Host maintenance runs before provider/tool execution, never inside a query.
    pub(crate) async fn prepare(
        session: &DurableSession,
        cancel: &CancellationToken,
    ) -> Option<Self> {
        let source = session.capture_recovery_source().ok()?;
        if !session.owns_recovery_source(&source) {
            return None;
        }
        let mut scan = source.begin_prefix_seal().ok()?;
        let path = session.recovery_index_path().ok()?;
        let mut index = SessionIndex::new(&path).ok()?;
        let stopped = cancel.clone();
        let budget =
            RecoveryWorkBudget::with_cancellation_check(Duration::from_secs(10), move || {
                stopped.is_cancelled()
            });
        let deadline = Instant::now() + Duration::from_secs(10);
        index.init_recovery_schema_with_budget(&budget).ok()?;
        let scope = uuid::Uuid::new_v4().to_string();
        index
            .replace_recovery_owner_scope(&session.id().to_string(), &scope, &budget)
            .ok()?;
        let mut cursor = index
            .begin_recovery_build(&scope, source.source_end(), &budget)
            .ok()?;
        let mut parent = None;
        while !cursor.complete() {
            if cancel.is_cancelled() || Instant::now() >= deadline {
                return None;
            }
            let page = source
                .read_page(cursor.next_offset(), deadline, || cancel.is_cancelled())
                .ok()?;
            // Embedded durable transcripts are linear. Refuse unsupported tree
            // layout instead of indexing sibling records into one scope.
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
                    &scope,
                    &cursor,
                    RecoveryBuildPage {
                        next_offset: page.next_offset,
                        scanned_entries: page.entries.len() as u64,
                        sources: &sources,
                    },
                    &budget,
                )
                .ok()?;
            cursor = index.recovery_build_cursor(&scope, &budget).ok()??;
            tokio::task::yield_now().await;
        }
        loop {
            if cancel.is_cancelled() || Instant::now() >= deadline {
                return None;
            }
            if index
                .resolve_recovery_pending(&scope, &source, &budget)
                .ok()?
            {
                break;
            }
            tokio::task::yield_now().await;
        }
        loop {
            if scan.step(deadline, || cancel.is_cancelled()).ok()? {
                break;
            }
            tokio::task::yield_now().await;
        }
        let seal = scan.finish().ok()?;
        Some(Self {
            path,
            scope,
            owner: session.id().to_string(),
            cancel: cancel.clone(),
            seal,
            refs: Mutex::new(BTreeMap::new()),
        })
    }

    pub(crate) fn cancelled(&self) -> bool {
        self.cancel.is_cancelled()
    }

    pub(crate) fn current(&self) -> bool {
        !self.cancel.is_cancelled() && self.seal.check_current().is_ok()
    }

    pub(crate) fn query(&self, query: Option<&str>, reference: Option<&str>) -> Option<Value> {
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
        let mut refs = self.refs.lock().ok()?;
        let result = if let Some(reference) = reference {
            let candidate = refs.get(reference)?;
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
            let candidates = index
                .recovery_candidates(&self.scope, query?, None, 20, &budget)
                .ok()?;
            let mut incomplete = candidates.len() == 20;
            let mut results = Vec::new();
            let mut next_refs = BTreeMap::new();
            // Bound all source/terminal record reads for this query together.
            let mut remaining = talos_session::history_page::HISTORY_PAGE_BYTES as u64;
            for candidate in candidates {
                let Some(text) = index
                    .recovery_candidate_text_with_source_budget(
                        &self.scope,
                        &candidate,
                        &self.seal,
                        talos_session::history_projection::RecoveryTextKind::Snippet,
                        &budget,
                        &mut remaining,
                    )
                    .ok()?
                else {
                    continue;
                };
                let reference = uuid::Uuid::new_v4().to_string();
                results.push(
                    json!({"reference":reference,"text":text.text,"truncated":text.truncated}),
                );
                if serde_json::to_vec(&results).ok()?.len() > 15_000 {
                    results.pop();
                    incomplete = true;
                    break;
                }
                next_refs.insert(reference, candidate);
            }
            *refs = next_refs;
            json!({"status":"Ready","results":results,"complete":!incomplete,"pagination_available":false})
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
        assert!(first.query(Some("fact"), None).is_some());
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
            first.query(Some("fact"), None).is_none(),
            "retired scope is unavailable, not complete empty results"
        );
        assert!(second.query(Some("fact"), None).is_some());
        cancel.cancel();
        assert!(second.query(Some("fact"), None).is_none());
    }
}
