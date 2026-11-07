//! Presentation-only grouping of adjacent, correlated native process polls.

use serde_json::Value;
use talos_conversation::{ToolCallDisplay, ToolResultDisplay};
use talos_core::tool::ToolProvenance;

use crate::transcript::{TranscriptBlock, TranscriptEntry};

pub(crate) struct PollGroup {
    pub(crate) entries: usize,
    pub(crate) job: String,
    pub(crate) state: String,
    pub(crate) polls: usize,
    pub(crate) text_bytes: usize,
    pub(crate) terminal: bool,
    pub(crate) exit_code: Option<i64>,
    pub(crate) truncated: bool,
    pub(crate) dropped: bool,
    pub(crate) partial: bool,
    pub(crate) pending: bool,
}

fn poll_call(call: &ToolCallDisplay) -> Option<(&str, &str)> {
    if call.tool_name != "process" || call.provenance != ToolProvenance::Native {
        return None;
    }
    let action = call.arguments.get("action")?.as_str()?;
    if !matches!(action, "read" | "status") {
        return None;
    }
    let job = call.arguments.get("job_id")?.as_str()?;
    (!job.is_empty()).then_some((job, action))
}

fn poll_pair(entries: &[TranscriptEntry]) -> Option<PollGroup> {
    let TranscriptBlock::ToolCall(call) = &entries.first()?.block else {
        return None;
    };
    let (job, action) = poll_call(call)?;
    let TranscriptBlock::ToolResult(result) = &entries.get(1)?.block else {
        return None;
    };
    if result.is_error
        || !matches!(
            result.tool_name.as_deref(),
            Some("process" | "background:process")
        )
    {
        return None;
    }
    let value: Value = serde_json::from_str(&result.content).ok()?;
    if value.get("job_id")?.as_str()? != job {
        return None;
    }
    let state = value.get("state")?.as_str()?;
    let terminal = match state {
        "starting" | "running" => false,
        "completed" | "failed" | "timed_out" | "cancelled" | "spawn_failed"
        | "supervision_failed" => true,
        _ => return None,
    };
    let mut text_bytes = 0usize;
    if action == "read" {
        for event in value.get("events")?.as_array()? {
            text_bytes = text_bytes.saturating_add(event.get("text")?.as_str()?.len());
        }
    }
    Some(PollGroup {
        entries: 2,
        job: job.to_owned(),
        state: state.to_owned(),
        polls: 1,
        text_bytes,
        terminal,
        exit_code: serde_json::from_value::<Option<i64>>(value["exit_code"].clone()).ok()?,
        truncated: serde_json::from_value::<Option<bool>>(value["truncated"].clone())
            .ok()?
            .unwrap_or(false),
        dropped: serde_json::from_value::<Option<u64>>(value["dropped_before"].clone())
            .ok()?
            .is_some(),
        partial: serde_json::from_value::<Option<bool>>(value["partial"].clone())
            .ok()?
            .unwrap_or(false),
        pending: false,
    })
}

pub(crate) fn group_at(entries: &[TranscriptEntry]) -> Option<PollGroup> {
    let mut group = poll_pair(entries)?;
    // A terminal observation gets its own prominent row, never swallowed by
    // preceding running polls or subsequent repeated status requests.
    if group.terminal {
        return Some(group);
    }
    while let Some(next) = poll_pair(&entries[group.entries..]) {
        if next.job != group.job || next.terminal {
            break;
        }
        group.entries += next.entries;
        group.polls += next.polls;
        group.text_bytes = group.text_bytes.saturating_add(next.text_bytes);
        group.state = next.state;
        group.truncated |= next.truncated;
        group.dropped |= next.dropped;
        group.partial |= next.partial;
    }
    // Keep a newly dispatched poll inside the live summary while its result is
    // pending. An intervening event or a failed result restores ordinary rows.
    if let [entry] = &entries[group.entries..]
        && let TranscriptBlock::ToolCall(call) = &entry.block
        && poll_call(call).is_some_and(|(job, _)| job == group.job)
    {
        group.entries += 1;
        group.pending = true;
    }
    Some(group)
}

impl PollGroup {
    pub(crate) fn summary(&self, expanded: bool) -> String {
        let marker = if expanded { "▾" } else { "▸" };
        let terminal = if self.terminal {
            if self.state == "completed" {
                "✓ "
            } else {
                "! "
            }
        } else {
            ""
        };
        let mut text = format!(
            "{marker} {terminal}process · {} · {} polls · {} B read",
            self.state, self.polls, self.text_bytes
        );
        if let Some(code) = self.exit_code {
            text.push_str(&format!(" · exit {code}"));
        }
        if self.pending {
            text.push_str(" · polling…");
        }
        if self.truncated {
            text.push_str(" · truncated");
        }
        if self.dropped {
            text.push_str(" · output gap");
        }
        if self.partial {
            text.push_str(" · partial read");
        }
        text.push_str(&format!(" · {}", self.job));
        text
    }
}

/// Expanded polls show their payload, rather than applying the ordinary
/// background summary a second time. The original stored result stays intact.
pub(crate) fn detail_result(result: &ToolResultDisplay) -> ToolResultDisplay {
    let mut display = result.clone();
    display.tool_name = None;
    display
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::transcript::{TranscriptEntryId, TranscriptStore};

    pub(crate) fn append_poll(
        store: &mut TranscriptStore,
        job: &str,
        action: &str,
        state: &str,
    ) -> TranscriptEntryId {
        let id = store.append(TranscriptBlock::ToolCall(ToolCallDisplay {
            tool_name: "process".into(),
            arguments: serde_json::json!({"action":action,"job_id":job,"cursor":0}),
            provenance: ToolProvenance::Native,
            summary_fields: vec![],
        }));
        store.append(TranscriptBlock::ToolResult(ToolResultDisplay {
            tool_name: Some("background:process".into()),
            is_error: false,
            content: serde_json::json!({"job_id":job,"state":state,
                "events":[{"text":"中文\n"}], "exit_code": if state=="completed" {Some(0)} else {None},
                "next_cursor":7}).to_string(),
            todo_presentation: None,
        }));
        id
    }

    #[test]
    fn malformed_metadata_stays_ordinary_for_reads_and_status() {
        for action in ["read", "status"] {
            for key in ["truncated", "partial", "dropped_before", "exit_code"] {
                let mut store = TranscriptStore::default();
                append_poll(&mut store, "a", action, "running");
                let mut entries = store.entries().to_vec();
                if let TranscriptBlock::ToolResult(result) = &mut entries[1].block {
                    let mut value: Value = serde_json::from_str(&result.content).expect("JSON");
                    value[key] = "invalid".into();
                    result.content = value.to_string();
                }
                assert!(group_at(&entries).is_none(), "{action}: {key}");
            }
        }
    }

    #[test]
    fn consecutive_polls_count_observed_text_not_status_cursors() {
        let mut store = TranscriptStore::default();
        for index in 0..24 {
            append_poll(
                &mut store,
                "job_a",
                if index % 2 == 0 { "read" } else { "status" },
                "running",
            );
        }
        let group = group_at(store.entries()).expect("group");
        assert_eq!(group.entries, 48);
        assert_eq!(group.polls, 24);
        assert_eq!(group.text_bytes, 12 * 7); // Delivered UTF-8 text, including repeated reads.
        assert_eq!(store.entries().len(), 48);
        assert!(
            group
                .summary(false)
                .contains("running · 24 polls · 84 B read")
        );
        assert!(!group.terminal);
    }

    #[test]
    fn other_jobs_content_and_terminal_observations_end_groups() {
        let mut store = TranscriptStore::default();
        append_poll(&mut store, "a", "read", "running");
        append_poll(&mut store, "b", "status", "running");
        assert_eq!(group_at(store.entries()).expect("first").entries, 2);
        for state in [
            "completed",
            "failed",
            "timed_out",
            "cancelled",
            "spawn_failed",
            "supervision_failed",
        ] {
            let mut store = TranscriptStore::default();
            append_poll(&mut store, "a", "read", "running");
            append_poll(&mut store, "a", "status", state);
            append_poll(&mut store, "a", "status", "running");
            assert_eq!(group_at(store.entries()).expect("running").entries, 2);
            let terminal = group_at(&store.entries()[2..]).expect("terminal");
            assert!(terminal.terminal);
            assert_eq!(terminal.entries, 2);
            assert!(terminal.summary(false).contains(state));
        }
        let mut store = TranscriptStore::default();
        append_poll(&mut store, "a", "read", "running");
        store.append(TranscriptBlock::StyledLine(
            crate::app_stream::ScrollbackLine::plain("interleaved", None),
        ));
        append_poll(&mut store, "a", "read", "running");
        assert_eq!(
            group_at(store.entries()).expect("before content").entries,
            2
        );
    }

    #[test]
    fn errors_unrelated_provenance_malformed_and_cancel_stay_ordinary() {
        for case in [
            "error",
            "json",
            "job",
            "unknown",
            "events",
            "name",
            "provenance",
            "cancel",
        ] {
            let mut store = TranscriptStore::default();
            append_poll(&mut store, "a", "read", "running");
            let mut entries = store.entries().to_vec();
            if let TranscriptBlock::ToolCall(call) = &mut entries[0].block {
                match case {
                    "name" => call.tool_name = "exec".into(),
                    "provenance" => {
                        call.provenance = ToolProvenance::McpRemote {
                            server: "fixture".into(),
                        }
                    }
                    "cancel" => call.arguments["action"] = "cancel".into(),
                    _ => {}
                }
            }
            if let TranscriptBlock::ToolResult(result) = &mut entries[1].block {
                match case {
                    "error" => result.is_error = true,
                    "json" => result.content = "not json".into(),
                    "job" => {
                        result.content =
                            r#"{"job_id":"other","state":"running","events":[]}"#.into()
                    }
                    "unknown" => {
                        result.content = r#"{"job_id":"a","state":"future","events":[]}"#.into()
                    }
                    "events" => {
                        result.content = r#"{"job_id":"a","state":"running","events":[{}]}"#.into()
                    }
                    _ => {}
                }
            }
            assert!(group_at(&entries).is_none(), "{case}");
        }
    }

    #[test]
    fn pending_poll_and_loss_signals_remain_visible() {
        let mut store = TranscriptStore::default();
        append_poll(&mut store, "a", "read", "running");
        let mut entries = store.entries().to_vec();
        if let TranscriptBlock::ToolResult(result) = &mut entries[1].block {
            let mut value: Value = serde_json::from_str(&result.content).expect("JSON");
            value["truncated"] = true.into();
            value["partial"] = true.into();
            value["dropped_before"] = 512.into();
            result.content = value.to_string();
        }
        entries.push(entries[0].clone());
        let group = group_at(&entries).expect("pending");
        assert_eq!(group.entries, 3);
        assert_eq!(group.polls, 1);
        let summary = group.summary(false);
        for marker in ["polling…", "truncated", "output gap", "partial read"] {
            assert!(summary.contains(marker), "{summary}");
        }
    }
}
