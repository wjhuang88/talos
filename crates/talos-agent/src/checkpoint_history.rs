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
            state: Mutex::new(QueryState::default()),
        })
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
                    last = Some(candidate.ordinal);
                    continue;
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
