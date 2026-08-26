use serde_json::Value;

/// Formats a bounded, display-safe summary for a background-job tool result.
///
/// The original tool result remains available to the model. This projection is
/// only for the human-facing print bridge and intentionally excludes arbitrary
/// command arguments, environment values, and unbounded output.
pub(crate) fn format_background_result(tool_name: &str, content: &str) -> Option<String> {
    if !matches!(tool_name, "bash" | "exec" | "process") {
        return None;
    }
    let value = serde_json::from_str::<Value>(content).ok()?;
    let object = value.as_object()?;
    if tool_name == "process"
        && let Some(jobs) = object.get("jobs").and_then(Value::as_array)
    {
        let truncated = object
            .get("truncated")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let entries = jobs
            .iter()
            .take(8)
            .filter_map(|job| {
                let id = job.get("job_id")?.as_str()?;
                let state = job.get("state")?.as_str().unwrap_or("unknown");
                Some(format!("{id}={state}"))
            })
            .collect::<Vec<_>>();
        let details = if entries.is_empty() {
            format!("{} job(s)", jobs.len())
        } else {
            format!(
                "{}{}",
                entries.join(", "),
                if jobs.len() > entries.len() {
                    ", …"
                } else {
                    ""
                }
            )
        };
        return Some(format!(
            "[background jobs] {details}{}",
            if truncated { ", list truncated" } else { "" }
        ));
    }
    let job_id = object.get("job_id")?.as_str()?;
    if job_id.is_empty() {
        return None;
    }
    let state = object
        .get("state")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    if let Some(events) = object.get("events").and_then(Value::as_array) {
        let next_cursor = object
            .get("next_cursor")
            .and_then(Value::as_u64)
            .unwrap_or_default();
        let eof = object.get("eof").and_then(Value::as_bool).unwrap_or(false);
        let event_count = events.len();
        let suffix = if eof { ", eof" } else { "" };
        return Some(format!(
            "[background {job_id}] {state}, {event_count} event(s), next cursor {next_cursor}{suffix}"
        ));
    }
    let detail = object
        .get("exit_code")
        .and_then(Value::as_i64)
        .map(|code| format!(", exit code {code}"))
        .unwrap_or_default();
    let deadline = object
        .get("deadline_secs")
        .and_then(Value::as_u64)
        .map(|seconds| format!(", deadline {seconds}s"))
        .unwrap_or_default();
    Some(format!("[background {job_id}] {state}{detail}{deadline}"))
}

pub(crate) fn format_background_terminal(
    summary: &talos_core::background_job::BackgroundJobTerminalSummary,
) -> String {
    let error = summary
        .cleanup_error
        .as_deref()
        .map(|value| format!(", cleanup: {}", value.chars().take(120).collect::<String>()))
        .unwrap_or_default();
    format!(
        "[background {}] terminal: {:?}, stdout {} bytes, stderr {} bytes, cursor {}..{}{}",
        summary.job_id,
        summary.state,
        summary.stdout_bytes,
        summary.stderr_bytes,
        summary.earliest_cursor,
        summary.next_cursor,
        error
    )
}

#[cfg(test)]
mod tests {
    use super::format_background_result;

    #[test]
    fn formats_start_receipt_without_exposing_raw_json() {
        let summary = format_background_result(
            "bash",
            r#"{"job_id":"job_1","state":"running","tool":"bash","deadline_secs":30}"#,
        )
        .unwrap();
        assert_eq!(summary, "[background job_1] running, deadline 30s");
        assert!(!summary.contains("deadline_secs"));
    }

    #[test]
    fn formats_process_read_cursor_and_eof() {
        let summary = format_background_result(
            "process",
            r#"{"job_id":"job_1","state":"exited","events":[{"stream":"stdout","text":"line one"}],"next_cursor":12,"eof":true}"#,
        )
        .unwrap();
        assert_eq!(
            summary,
            "[background job_1] exited, 1 event(s), next cursor 12, eof"
        );
        assert!(!summary.contains("warning"));
        assert!(!summary.contains("secret-token"));
    }

    #[test]
    fn formats_process_list_without_dumping_job_records() {
        let summary = format_background_result(
            "process",
            r#"{"jobs":[{"job_id":"job_1","state":"running"}],"truncated":false}"#,
        )
        .unwrap();
        assert_eq!(summary, "[background jobs] job_1=running");
    }

    #[test]
    fn does_not_reclassify_foreground_or_unrelated_results() {
        assert!(format_background_result("bash", "done").is_none());
        assert!(
            format_background_result("read_file", r#"{"job_id":"job_1","state":"running"}"#)
                .is_none()
        );
    }
}
