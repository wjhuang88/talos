//! Bounded, request-directed projection from the live, source-bound message view.
//!
//! Preparation has no disclosure authority. The tool revalidates identity,
//! snapshot, cancellation and permission at delivery, just as for cached details.
use super::*;

#[derive(Clone)]
pub(super) struct PreparedRead {
    pub value: Value,
    pub next_cursor: Option<(String, String, usize)>,
}

pub(super) fn key(input: &HistoryInput) -> String {
    json!([
        input.mode,
        input.snapshot,
        input.query,
        input.reference,
        input.cursor
    ])
    .to_string()
}

pub(super) fn valid(input: &HistoryInput) -> bool {
    match input.mode.as_str() {
        "source_detail" => {
            input.query.is_none()
                && input.cursor.is_none()
                && input
                    .reference
                    .as_ref()
                    .is_some_and(|v| !v.is_empty() && v.len() <= 128)
        }
        "source_search" => {
            input.reference.is_none()
                && input
                    .query
                    .as_ref()
                    .is_some_and(|v| !v.is_empty() && v.len() <= 1024)
                && input.cursor.as_ref().is_none_or(|v| v.len() <= 128)
        }
        _ => false,
    }
}

impl Execution {
    /// Project only requested source reads while the runtime owns the exact raw
    /// message view. Never retain that view or trust model-supplied message text.
    pub(crate) async fn prepare_source_reads(
        &self,
        messages: &[Message],
        calls: &[talos_core::message::ToolCall],
    ) {
        let (issued, epoch, cursor) = {
            let Ok(mut state) = self.state.lock() else {
                return;
            };
            state.source_reads.clear();
            if !calls.iter().any(|call| {
                call.name == "history_search"
                    && matches!(
                        call.input.get("mode").and_then(Value::as_str),
                        Some("source_search" | "source_detail")
                    )
            }) {
                return;
            }
            let (Some(issued), Some(current)) = (&state.issued, &state.current) else {
                return;
            };
            if current.end < issued.end
                || messages.len() < issued.end
                || !(0..issued.end).all(|i| issued.source.same_entry(&current.source, i))
            {
                return;
            }
            (
                issued.clone(),
                state.refresh_epoch,
                state.source_cursor.clone(),
            )
        };
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(2);
        let by_id: BTreeMap<_, _> = issued
            .refs
            .iter()
            .map(|(reference, id)| (id.as_str(), reference.as_str()))
            .collect();
        let mut reads = BTreeMap::new();
        // Reserve within the existing 2 MiB text budget; the legacy cache uses
        // the remainder. Never retain an unbounded full-history text copy.
        let mut remaining = SOURCE_READ_BUDGET;
        for call in calls
            .iter()
            .filter(|call| call.name == "history_search")
            .take(16)
        {
            let Ok(input) = serde_json::from_value::<HistoryInput>(call.input.clone()) else {
                continue;
            };
            if !valid(&input) || input.snapshot.as_deref() != Some(&issued.token) {
                continue;
            }
            let start = if let Some(token) = &input.cursor {
                let Some((expected, query, start)) = &cursor else {
                    continue;
                };
                if token != expected || input.query.as_ref() != Some(query) {
                    continue;
                }
                *start
            } else {
                0
            };
            let mut results = Vec::new();
            let mut detail_result = None;
            let mut next = issued.end;
            for (index, message) in messages.iter().enumerate().take(issued.end).skip(start) {
                tokio::task::yield_now().await;
                if tokio::time::Instant::now() >= deadline {
                    return;
                }
                let Some(id) = issued.source.resolve(
                    issued.source.generation(),
                    issued.source.revision(),
                    index,
                    message,
                ) else {
                    return;
                };
                let Some(reference) = by_id.get(id) else {
                    continue;
                };
                if input.mode == "source_detail" && input.reference.as_deref() != Some(*reference) {
                    continue;
                }
                let Some(detail) = checkpoint_source_text(message) else {
                    continue;
                };
                if input.mode == "source_detail" {
                    detail_result = Some(
                        json!({"reference":reference,"text":detail.text,"truncated":detail.truncated,"checkpoint_source":true}),
                    );
                    break;
                }
                if !input
                    .query
                    .as_ref()
                    .is_some_and(|query| detail.text.contains(query))
                {
                    continue;
                }
                let mut end = detail.text.len().min(256);
                while !detail.text.is_char_boundary(end) {
                    end -= 1;
                }
                results.push(json!({"reference":reference,"text":&detail.text[..end],"truncated":detail.truncated || end < detail.text.len(),"checkpoint_source":true}));
                if results.len() == 5 {
                    next = index + 1;
                    break;
                }
            }
            let next_cursor = (input.mode == "source_search" && next < issued.end).then(|| {
                (
                    uuid::Uuid::new_v4().to_string(),
                    input.query.clone().unwrap_or_default(),
                    next,
                )
            });
            let value = if input.mode == "source_search" {
                json!({"results":results,"complete":next_cursor.is_none(),"next_cursor":next_cursor.as_ref().map(|v| &v.0),"checkpoint_source":true})
            } else if let Some(detail) = detail_result {
                detail
            } else {
                continue;
            };
            let bytes = value.to_string().len();
            if bytes > remaining {
                continue;
            }
            remaining -= bytes;
            reads.insert(key(&input), PreparedRead { value, next_cursor });
        }
        if let Ok(mut state) = self.state.lock()
            && state.refresh_epoch == epoch
            && state
                .issued
                .as_ref()
                .is_some_and(|snapshot| snapshot.token == issued.token)
            && state.current.as_ref().is_some_and(|current| {
                current.end >= issued.end
                    && (0..issued.end).all(|i| issued.source.same_entry(&current.source, i))
            })
        {
            state.source_reads = reads;
        }
    }
}
