//! Runtime-owned identity for a raw-history prefix in a derived request.

use talos_core::message::Message;

/// Mapping established before prompt construction; synthetic suffixes are excluded.
#[derive(Debug, Clone)]
pub struct HistorySource {
    generation: u64,
    revision: u64,
    deadline: Option<tokio::time::Instant>,
    entry_ids: Vec<String>,
    digests: Vec<[u8; 32]>,
    continuation: std::collections::BTreeMap<usize, (String, [u8; 32])>,
}

impl HistorySource {
    /// Derive a generation-qualified identifier from the ordinal and exact
    /// raw message bytes. The ordinal disambiguates identical adjacent messages;
    /// the digest prevents a replacement at the same position from aliasing it.
    pub fn stable_entry_id(generation: u64, ordinal: usize, message: &Message) -> Option<String> {
        let digest = message_digest(message)?;
        Some(format!(
            "generation:{generation}:entry:{ordinal}:{}",
            digest
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        ))
    }
    /// Bind generation-qualified content IDs in one bounded cooperative scan.
    /// These are in-memory source references, not durable TLOG entry identities.
    pub(crate) async fn bind_content_until(
        generation: u64,
        revision: u64,
        raw: &[Message],
        deadline: tokio::time::Instant,
    ) -> Option<Self> {
        if raw.len() > 8192 {
            return None;
        }
        let mut source = Self::bind_until(
            generation,
            revision,
            raw,
            (0..raw.len()).map(|index| index.to_string()).collect(),
            deadline,
        )
        .await?;
        for (index, digest) in source.digests.iter().enumerate() {
            tokio::task::yield_now().await;
            if tokio::time::Instant::now() >= deadline {
                return None;
            }
            let id = format!(
                "generation:{generation}:entry:{index}:{}",
                digest
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>()
            );
            if id.len() > 128 {
                return None;
            }
            source.entry_ids[index] = id;
        }
        Some(source)
    }

    /// Advance the Actor snapshot after an atomic turn persistence receipt.
    /// Durable IDs trace the live message to its stored (possibly redacted)
    /// representation; digests always bind the current exact model bytes.
    /// Missing or inconsistent receipts retain explicit in-memory identities.
    pub(crate) fn bind_committed(
        generation: u64,
        raw: &[Message],
        previous: Option<&Self>,
        start: usize,
        namespace: Option<&str>,
        committed_ids: &[String],
    ) -> Option<Self> {
        if raw.len() > 8192 || start > raw.len() {
            return None;
        }
        let mut source = Self::bind(
            generation,
            raw.len() as u64,
            raw,
            (0..raw.len()).map(|index| index.to_string()).collect(),
        )?;
        for (index, digest) in source.digests.iter().enumerate() {
            source.entry_ids[index] = format!(
                "generation:{generation}:entry:{index}:{}",
                digest
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>()
            );
        }
        if let Some(previous) = previous.filter(|old| old.generation == generation) {
            for index in 0..start.min(previous.digests.len()) {
                if source.digests[index] != previous.digests[index] {
                    break;
                }
                source.entry_ids[index] = previous.entry_ids[index].clone();
            }
        }
        if let Some(namespace) = namespace
            && !namespace.is_empty()
            && committed_ids.len() == raw.len() - start
            && committed_ids.iter().all(|id| !id.is_empty())
        {
            for (index, id) in committed_ids.iter().enumerate() {
                source.entry_ids[start + index] = format!("tlog:{namespace}:{id}");
            }
        }
        let mut seen = std::collections::HashSet::new();
        if source
            .entry_ids
            .iter()
            .any(|id| id.len() > 128 || !seen.insert(id))
        {
            return None;
        }
        Some(source)
    }

    /// Reuse verified durable identities for the unchanged recovered prefix.
    /// New or changed entries retain their generation/content identities.
    pub(crate) async fn bind_recovered_until(
        generation: u64,
        revision: u64,
        raw: &[Message],
        recovered: Option<&Self>,
        deadline: tokio::time::Instant,
    ) -> Option<Self> {
        let mut source = Self::bind_content_until(generation, revision, raw, deadline).await?;
        if let Some(recovered) = recovered.filter(|old| old.generation == generation) {
            for index in 0..source.digests.len().min(recovered.digests.len()) {
                tokio::task::yield_now().await;
                if tokio::time::Instant::now() >= deadline {
                    return None;
                }
                // Stop at the first divergence: a matching message after a
                // changed prefix does not establish the same branch ancestry.
                if source.digests[index] != recovered.digests[index] {
                    break;
                }
                source.entry_ids[index] = recovered.entry_ids[index].clone();
            }
        }
        Some(source)
    }

    /// Deadline shared by source binding and the subsequent checkpoint attempt.
    /// Synchronous bindings have no execution budget until an attempt begins.
    pub(crate) fn deadline(&self) -> Option<tokio::time::Instant> {
        self.deadline
    }

    /// Runtime generation that owns this raw snapshot.
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Raw-history revision captured before request construction.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Bind stable Actor entry IDs to exact raw messages for one revision.
    /// Returns None for duplicate/oversized IDs or an unbounded history scan.
    pub fn bind(
        generation: u64,
        revision: u64,
        raw: &[Message],
        entry_ids: Vec<String>,
    ) -> Option<Self> {
        if raw.len() > 8192 || raw.len() != entry_ids.len() {
            return None;
        }
        let mut seen = std::collections::HashSet::new();
        if entry_ids
            .iter()
            .any(|id| id.is_empty() || id.len() > 128 || !seen.insert(id))
        {
            return None;
        }
        let mut remaining = 2 * 1024 * 1024;
        let digests = raw
            .iter()
            .map(|message| message_digest_bounded(message, &mut remaining))
            .collect::<Option<Vec<_>>>()?;
        Some(Self {
            generation,
            revision,
            deadline: None,
            entry_ids,
            digests,
            continuation: Default::default(),
        })
    }

    /// Bind a production snapshot cooperatively under a caller-owned deadline.
    /// Dropping this future discards partial bindings; no partial source is returned.
    pub async fn bind_until(
        generation: u64,
        revision: u64,
        raw: &[Message],
        entry_ids: Vec<String>,
        deadline: tokio::time::Instant,
    ) -> Option<Self> {
        if raw.len() > 8192 || raw.len() != entry_ids.len() {
            return None;
        }
        let mut seen = std::collections::HashSet::new();
        let mut remaining = 2 * 1024 * 1024;
        let mut digests = Vec::with_capacity(raw.len());
        for (message, id) in raw.iter().zip(&entry_ids) {
            tokio::task::yield_now().await;
            if tokio::time::Instant::now() >= deadline
                || id.is_empty()
                || id.len() > 128
                || !seen.insert(id)
            {
                return None;
            }
            digests.push(message_digest_bounded(message, &mut remaining)?);
        }
        if tokio::time::Instant::now() >= deadline {
            return None;
        }
        Some(Self {
            generation,
            revision,
            deadline: Some(deadline),
            entry_ids,
            digests,
            continuation: Default::default(),
        })
    }

    /// Bind newly appended canonical turn messages beyond a synthetic prompt gap.
    /// The caller supplies the persistence boundary, never a hook-built request.
    /// A failed or cancelled scan leaves the preceding mapping unchanged.
    pub async fn bind_continuation(&mut self, messages: &[Message], persist_start: usize) -> bool {
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(120);
        self.bind_continuation_until(messages, persist_start, deadline)
            .await
    }

    pub(crate) async fn bind_continuation_until(
        &mut self,
        messages: &[Message],
        persist_start: usize,
        deadline: tokio::time::Instant,
    ) -> bool {
        if persist_start < self.digests.len()
            || persist_start > messages.len()
            || messages.len().saturating_sub(persist_start) > 8192
        {
            return false;
        }
        let mut remaining = 2 * 1024 * 1024;
        let mut entries = std::collections::BTreeMap::new();
        for (offset, message) in messages[persist_start..].iter().enumerate() {
            tokio::task::yield_now().await;
            if tokio::time::Instant::now() >= deadline {
                return false;
            }
            let Some(digest) = message_digest_bounded(message, &mut remaining) else {
                return false;
            };
            let Some(raw_index) = self.revision.checked_add(offset as u64) else {
                return false;
            };
            let id = format!(
                "generation:{}:entry:{raw_index}:{}",
                self.generation,
                digest
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>()
            );
            entries.insert(persist_start + offset, (id, digest));
        }
        if tokio::time::Instant::now() >= deadline {
            return false;
        }
        self.continuation = entries;
        self.deadline = Some(deadline);
        true
    }

    /// Compare already-bound identities without copying or rehashing raw history.
    pub(crate) fn same_entry(&self, other: &Self, index: usize) -> bool {
        fn entry(source: &HistorySource, index: usize) -> Option<(&str, &[u8; 32])> {
            if let Some((id, digest)) = source.continuation.get(&index) {
                Some((id.as_str(), digest))
            } else {
                Some((
                    source.entry_ids.get(index)?.as_str(),
                    source.digests.get(index)?,
                ))
            }
        }
        self.generation == other.generation
            && entry(self, index).is_some_and(|left| Some(left) == entry(other, index))
    }

    /// Return a source ID only when generation, revision, position and bytes match.
    /// Hook replacement, reordering and synthetic request entries fail closed.
    pub fn resolve(
        &self,
        generation: u64,
        revision: u64,
        index: usize,
        message: &Message,
    ) -> Option<&str> {
        if generation != self.generation || revision != self.revision {
            return None;
        }
        if let Some((id, digest)) = self.continuation.get(&index) {
            return (*digest == message_digest(message)?).then_some(id.as_str());
        }
        let expected = self.digests.get(index)?;
        if *expected != message_digest(message)? {
            return None;
        }
        self.entry_ids.get(index).map(String::as_str)
    }
}

fn message_digest(message: &Message) -> Option<[u8; 32]> {
    message_digest_bounded(message, &mut (2 * 1024 * 1024))
}

fn message_digest_bounded(message: &Message, remaining: &mut usize) -> Option<[u8; 32]> {
    use sha2::{Digest, Sha256};
    struct Sink {
        hash: Sha256,
        remaining: usize,
    }
    impl std::io::Write for Sink {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > self.remaining {
                return Err(std::io::Error::other(
                    "raw source hashing byte limit exceeded",
                ));
            }
            self.remaining -= bytes.len();
            self.hash.update(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut sink = Sink {
        hash: Sha256::new(),
        remaining: *remaining,
    };
    serde_json::to_writer(&mut sink, message).ok()?;
    *remaining = sink.remaining;
    Some(sink.hash.finalize().into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn committed_sources_do_not_invent_durable_receipts_or_alias_entries() {
        let first = Message::User {
            content: "first".into(),
        };
        let raw = vec![
            first.clone(),
            Message::User {
                content: "second".into(),
            },
        ];
        let previous = HistorySource::bind(5, 1, &[first], vec!["tlog:session:first".into()])
            .expect("previous");
        let bound = HistorySource::bind_committed(
            5,
            &raw,
            Some(&previous),
            1,
            Some("session"),
            &["second".into()],
        )
        .expect("receipt");
        assert_eq!(bound.resolve(5, 2, 0, &raw[0]), Some("tlog:session:first"));
        assert_eq!(bound.resolve(5, 2, 1, &raw[1]), Some("tlog:session:second"));
        let missing =
            HistorySource::bind_committed(5, &raw, Some(&previous), 1, Some("session"), &[])
                .expect("no receipt");
        assert!(
            missing
                .resolve(5, 2, 1, &raw[1])
                .expect("memory source")
                .starts_with("generation:")
        );
        assert!(
            HistorySource::bind_committed(
                5,
                &raw,
                Some(&previous),
                1,
                Some("session"),
                &["first".into()]
            )
            .is_none()
        );
        assert!(
            HistorySource::bind_committed(
                5,
                &raw,
                Some(&previous),
                1,
                Some("session"),
                &["x".repeat(129)]
            )
            .is_none()
        );
    }

    #[tokio::test]
    async fn recovered_sources_require_generation_and_unchanged_prefix() {
        let raw = vec![
            Message::User {
                content: "first".into(),
            },
            Message::User {
                content: "second".into(),
            },
        ];
        let recovered = HistorySource::bind(
            8,
            2,
            &raw,
            vec!["tlog:session:a".into(), "tlog:session:b".into()],
        )
        .expect("snapshot");
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(10);
        let bound = HistorySource::bind_recovered_until(8, 2, &raw, Some(&recovered), deadline)
            .await
            .expect("bind");
        assert_eq!(bound.resolve(8, 2, 1, &raw[1]), Some("tlog:session:b"));
        let mut changed = raw.clone();
        changed[0] = Message::User {
            content: "different branch".into(),
        };
        let bound = HistorySource::bind_recovered_until(8, 2, &changed, Some(&recovered), deadline)
            .await
            .expect("bind");
        assert!(
            bound
                .resolve(8, 2, 1, &changed[1])
                .expect("id")
                .starts_with("generation:")
        );
        let bound = HistorySource::bind_recovered_until(9, 2, &raw, Some(&recovered), deadline)
            .await
            .expect("bind");
        assert!(
            bound
                .resolve(9, 2, 0, &raw[0])
                .expect("id")
                .starts_with("generation:9:")
        );
    }

    #[tokio::test]
    async fn content_binding_accepts_full_width_runtime_generation() {
        let raw = vec![Message::User {
            content: "retained constraint".into(),
        }];
        let source = HistorySource::bind_content_until(
            u64::MAX,
            1,
            &raw,
            tokio::time::Instant::now() + std::time::Duration::from_secs(1),
        )
        .await
        .expect("full-width generation must fit schema source reference");
        let id = source.resolve(u64::MAX, 1, 0, &raw[0]).expect("source");
        assert!(id.len() <= 128);
        assert!(id.contains(&u64::MAX.to_string()));
    }

    #[test]
    fn synthetic_reordered_changed_and_stale_messages_have_no_raw_authority() {
        let raw = vec![
            Message::User {
                content: "original".into(),
            },
            Message::User {
                content: "second".into(),
            },
        ];
        let source = HistorySource::bind(3, 9, &raw, vec!["entry-a".into(), "entry-b".into()])
            .expect("valid source");
        assert_eq!(source.resolve(3, 9, 0, &raw[0]), Some("entry-a"));
        assert_eq!(source.resolve(3, 9, 0, &raw[1]), None);
        assert_eq!(source.resolve(3, 9, 2, &raw[0]), None);
        assert_eq!(source.resolve(4, 9, 0, &raw[0]), None);
        assert_eq!(source.resolve(3, 10, 0, &raw[0]), None);
        assert_eq!(
            source.resolve(
                3,
                9,
                0,
                &Message::Context {
                    content: "original".into()
                }
            ),
            None
        );
    }
    #[test]
    fn duplicate_ids_cannot_alias_distinct_raw_entries() {
        let raw = vec![
            Message::User {
                content: "same".into()
            };
            2
        ];
        assert!(HistorySource::bind(1, 1, &raw, vec!["a".into(), "a".into()]).is_none());
        assert!(HistorySource::bind(1, 1, &raw, vec!["a".into()]).is_none());
    }

    #[tokio::test(start_paused = true)]
    async fn production_content_binding_obeys_cumulative_bounds_and_cancellation() {
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(120);
        let raw = vec![
            Message::User {
                content: "x".repeat(600_000)
            };
            4
        ];
        assert!(
            HistorySource::bind_content_until(7, 4, &raw, deadline)
                .await
                .is_none()
        );
        assert!(
            HistorySource::bind_content_until(7, 3, &raw[..3], tokio::time::Instant::now())
                .await
                .is_none()
        );
        {
            let pending = HistorySource::bind_content_until(7, 3, &raw[..3], deadline);
            tokio::pin!(pending);
            tokio::select! { biased;
                _ = &mut pending => panic!("binding must yield before hashing"),
                _ = std::future::ready(()) => {},
            }
        }
        let bound = HistorySource::bind_content_until(7, 3, &raw[..3], deadline)
            .await
            .expect("bounded history");
        for (index, message) in raw[..3].iter().enumerate() {
            let expected = HistorySource::stable_entry_id(7, index, message).expect("id");
            assert_eq!(bound.resolve(7, 3, index, message), Some(expected.as_str()));
        }
    }

    #[test]
    fn stable_ids_rebind_identically_after_restart_and_change_on_replacement() {
        let raw = vec![Message::User {
            content: "constraint".into(),
        }];
        let ids = raw
            .iter()
            .enumerate()
            .map(|(index, message)| HistorySource::stable_entry_id(11, index, message).expect("id"))
            .collect::<Vec<_>>();
        let reopened = HistorySource::bind(11, 1, &raw, ids.clone()).expect("reopened source");
        let replaced = [Message::User {
            content: "changed".into(),
        }];
        assert_eq!(reopened.resolve(11, 1, 0, &raw[0]), Some(ids[0].as_str()));
        assert_eq!(reopened.resolve(11, 1, 0, &replaced[0]), None);
    }
    #[test]
    fn source_binding_limits_total_serialized_history_not_only_each_message() {
        let raw = vec![
            Message::User {
                content: "x".repeat(600_000)
            };
            4
        ];
        assert!(
            HistorySource::bind(1, 4, &raw, (0..4).map(|i| format!("entry-{i}")).collect())
                .is_none()
        );
        assert!(
            HistorySource::bind(
                1,
                3,
                &raw[..3],
                (0..3).map(|i| format!("entry-{i}")).collect()
            )
            .is_some()
        );
    }
    #[tokio::test]
    async fn cooperative_binding_honors_deadline_and_can_be_cancelled() {
        let raw = vec![Message::User {
            content: "source".into(),
        }];
        let expired = tokio::time::Instant::now();
        assert!(
            HistorySource::bind_until(1, 1, &raw, vec!["a".into()], expired)
                .await
                .is_none()
        );
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(10);
        {
            let pending = HistorySource::bind_until(1, 1, &raw, vec!["a".into()], deadline);
            tokio::pin!(pending);
            tokio::select! { biased;
                _ = &mut pending => panic!("binding must yield before scanning"),
                _ = std::future::ready(()) => {},
            }
        }
        let source = HistorySource::bind_until(1, 1, &raw, vec!["a".into()], deadline)
            .await
            .expect("valid binding");
        assert_eq!(source.resolve(1, 1, 0, &raw[0]), Some("a"));
    }
    #[tokio::test]
    async fn continuation_bounds_new_work_instead_of_synthetic_prefix_length() {
        let raw = vec![Message::User {
            content: "original".into(),
        }];
        let mut source = HistorySource::bind(7, 1, &raw, vec!["old-id".into()]).expect("source");
        let mut messages = raw.clone();
        messages.extend((0..8300).map(|_| Message::Context {
            content: "synthetic gap".into(),
        }));
        let start = messages.len();
        messages.push(Message::User {
            content: "new canonical input".into(),
        });
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(10);
        assert!(
            source
                .bind_continuation_until(&messages, start, deadline)
                .await
        );
        assert!(source.resolve(7, 1, start, &messages[start]).is_some());
        assert!(
            source
                .resolve(7, 1, start - 1, &messages[start - 1])
                .is_none(),
            "synthetic gap is not source authority"
        );
        messages.extend((0..8192).map(|_| Message::User {
            content: "too much new work".into(),
        }));
        assert!(
            !source
                .bind_continuation_until(&messages, start, deadline)
                .await
        );
        assert!(
            source.resolve(7, 1, start, &messages[start]).is_some(),
            "failed replacement preserves previous mapping"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn continuation_carries_its_binding_deadline_and_failed_rebinding_is_atomic() {
        let raw = vec![Message::User {
            content: "old".into(),
        }];
        let mut source = HistorySource::bind(7, 1, &raw, vec!["old-id".into()]).expect("source");
        let mut messages = raw.clone();
        messages.push(Message::User {
            content: "new".into(),
        });
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(120);
        assert!(source.bind_continuation_until(&messages, 1, deadline).await);
        assert_eq!(source.deadline(), Some(deadline));
        tokio::time::advance(std::time::Duration::from_secs(120)).await;
        let mut changed = messages.clone();
        changed[1] = Message::User {
            content: "replacement".into(),
        };
        assert!(!source.bind_continuation_until(&changed, 1, deadline).await);
        assert_eq!(source.deadline(), Some(deadline));
        assert!(
            source
                .resolve(7, 1, 1, &messages[1])
                .is_some_and(|id| id.starts_with("generation:7:entry:1:"))
        );
        assert_eq!(source.resolve(7, 1, 1, &changed[1]), None);
    }

    #[tokio::test]
    async fn continuation_maps_new_raw_entries_without_authorizing_prompt_gap() {
        let raw = vec![Message::User {
            content: "old".into(),
        }];
        let mut source =
            HistorySource::bind(7, 1, &raw, vec!["generation:7:entry:0".into()]).expect("source");
        let messages = vec![
            raw[0].clone(),
            Message::Context {
                content: "synthetic workspace".into(),
            },
            Message::User {
                content: "new input".into(),
            },
            Message::Assistant {
                content: "new reply".into(),
                tool_calls: vec![],
                reasoning: None,
            },
        ];
        assert!(source.bind_continuation(&messages, 2).await);
        assert_eq!(source.resolve(7, 1, 1, &messages[1]), None);
        assert!(
            source
                .resolve(7, 1, 2, &messages[2])
                .is_some_and(|id| id.starts_with("generation:7:entry:1:"))
        );
        assert!(
            source
                .resolve(7, 1, 3, &messages[3])
                .is_some_and(|id| id.starts_with("generation:7:entry:2:"))
        );
        assert!(!source.bind_continuation(&messages, 0).await);
        assert!(
            source
                .resolve(7, 1, 3, &messages[3])
                .is_some_and(|id| id.starts_with("generation:7:entry:2:"))
        );
    }
}
