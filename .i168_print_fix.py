from pathlib import Path

path = Path("crates/talos-cli/src/mode_print.rs")
text = path.read_text()

replacements = [
    (
        "use talos_core::message::AgentEvent;\n",
        "use talos_core::message::{AgentEvent, StopReason};\n",
        "StopReason import",
    ),
    (
        "    let mut stdout = io::stdout().lock();\n    while let Some(event) = handle.eq_rx.recv().await {\n",
        "    let mut stdout = io::stdout().lock();\n    let mut terminal_notice = None;\n    while let Some(event) = handle.eq_rx.recv().await {\n",
        "terminal notice state",
    ),
    (
        '''            SessionEvent::TurnEvent {
                payload: TurnEventPayload::Completed { status },
                ..
            } => match status {
                talos_core::session::TurnCompletionStatus::Success { .. } => {
                    println!();
                    return Ok(());
                }
''',
        '''            SessionEvent::TurnEvent {
                payload:
                    TurnEventPayload::Progress {
                        event: AgentEvent::TurnEnd { stop_reason, .. },
                    },
                ..
            } => {
                terminal_notice = terminal_notice_for_stop_reason(&stop_reason);
            }
            SessionEvent::TurnEvent {
                payload: TurnEventPayload::Completed { status },
                ..
            } => match status {
                talos_core::session::TurnCompletionStatus::Success { .. } => {
                    if let Some(notice) = terminal_notice.take() {
                        eprintln!("{notice}");
                    }
                    println!();
                    return Ok(());
                }
''',
        "terminal progress handling",
    ),
    (
        "/// Choose the SessionOp for print mode based on --attach and the preview\n",
        '''fn terminal_notice_for_stop_reason(stop_reason: &StopReason) -> Option<&'static str> {
    match stop_reason {
        StopReason::MaxTokens => Some(
            "Warning: response truncated because the provider reached the output token limit; partial response preserved.",
        ),
        StopReason::EndTurn | StopReason::ToolUse => None,
    }
}

/// Choose the SessionOp for print mode based on --attach and the preview
''',
        "terminal notice helper",
    ),
    (
        '''    #[test]
    fn submit_op_plain_when_no_attach_and_no_preview() {
''',
        '''    #[test]
    fn terminal_print_notice_marks_max_tokens_as_truncated() {
        let notice = terminal_notice_for_stop_reason(&StopReason::MaxTokens)
            .expect("MaxTokens notice");
        assert!(notice.contains("truncated"));
        assert!(notice.contains("partial response preserved"));
    }

    #[test]
    fn terminal_print_notice_keeps_explicit_completion_quiet() {
        assert!(terminal_notice_for_stop_reason(&StopReason::EndTurn).is_none());
        assert!(terminal_notice_for_stop_reason(&StopReason::ToolUse).is_none());
    }

    #[test]
    fn submit_op_plain_when_no_attach_and_no_preview() {
''',
        "terminal notice tests",
    ),
]

for old, new, label in replacements:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected one match, found {count}")
    text = text.replace(old, new, 1)

path.write_text(text)
