use talos_core::message::{AgentEvent, Message, StopReason, Usage};
use talos_session::compaction_engine::CompactionResult;
use talos_session::{
    CompactTextSessionStore, ProviderTerminalDiagnostic, ProviderTerminalOutcome,
    ProviderTerminalSource, SessionManager, export_json, export_markdown, read_transcript,
};

fn diagnostic(turn_id: &str, ordinal: u32, event: &AgentEvent) -> ProviderTerminalDiagnostic {
    ProviderTerminalDiagnostic::from_agent_event(
        turn_id,
        ordinal,
        event,
        Some("fixture-provider"),
        Some("fixture-model"),
    )
    .expect("terminal diagnostic")
}

#[test]
fn max_tokens_is_visible_and_durable_as_truncated() {
    let temp_dir = tempfile::tempdir().expect("operation should succeed");
    let manager = SessionManager::with_dir(temp_dir.path().to_path_buf());
    let session = manager
        .create_session("max-tokens", "")
        .expect("operation should succeed");
    let terminal = AgentEvent::TurnEnd {
        stop_reason: StopReason::MaxTokens,
        usage: Usage::default(),
    };
    let diagnostic = diagnostic("turn-max", 1, &terminal);

    assert_eq!(diagnostic.outcome, ProviderTerminalOutcome::Truncated);
    assert_eq!(diagnostic.source, ProviderTerminalSource::Explicit);
    assert_eq!(diagnostic.reason.as_deref(), Some("max_tokens"));

    session
        .append_terminal_diagnostic(&diagnostic)
        .expect("operation should succeed");
    let reopened = session
        .read_terminal_diagnostics()
        .expect("operation should succeed");
    assert_eq!(reopened, vec![diagnostic]);
}

#[test]
fn interactive_tlog_terminal_diagnostic_round_trips_with_turn_correlation() {
    let temp_dir = tempfile::tempdir().expect("operation should succeed");
    let manager = SessionManager::with_dir(temp_dir.path().to_path_buf());
    let session = manager
        .create_session("round-trip", "")
        .expect("operation should succeed");
    let terminal = AgentEvent::Error {
        message: "unsupported provider finish_reason: fixture_unknown_reason".into(),
    };
    let diagnostic = diagnostic("turn-correlated", 3, &terminal);

    session
        .append_terminal_diagnostic(&diagnostic)
        .expect("operation should succeed");
    let reopened = manager
        .get_session(&session.id)
        .expect("operation should succeed");
    let diagnostics = reopened
        .read_terminal_diagnostics()
        .expect("operation should succeed");

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].turn_id, "turn-correlated");
    assert_eq!(diagnostics[0].response_ordinal, 3);
    assert_eq!(
        diagnostics[0].source,
        ProviderTerminalSource::UnsupportedReason
    );
    assert_eq!(
        diagnostics[0].reason.as_deref(),
        Some("fixture_unknown_reason")
    );
    assert_eq!(diagnostics[0].provider.as_deref(), Some("fixture-provider"));
    assert_eq!(diagnostics[0].model.as_deref(), Some("fixture-model"));
}

#[test]
fn known_provider_policies_remain_distinct_from_truly_unknown_reasons() {
    let known_cases = [
        (
            "provider response filtered by content policy (finish_reason=content_filter)",
            "content_filter",
        ),
        (
            "provider requested deprecated legacy function_call (finish_reason=function_call); use tool_calls",
            "legacy_function_call",
        ),
        (
            "provider paused turn (stop_reason=pause_turn); automatic continuation is not supported",
            "pause_turn",
        ),
        ("provider refused request (stop_reason=refusal)", "refusal"),
    ];

    for (index, (message, expected_reason)) in known_cases.into_iter().enumerate() {
        let terminal = AgentEvent::Error {
            message: message.into(),
        };
        let diagnostic = diagnostic("turn-known-policy", index as u32 + 1, &terminal);
        assert_eq!(diagnostic.outcome, ProviderTerminalOutcome::Error);
        assert_eq!(diagnostic.source, ProviderTerminalSource::ProviderError);
        assert_eq!(diagnostic.reason.as_deref(), Some(expected_reason));
    }

    let unknown = diagnostic(
        "turn-unknown-policy",
        1,
        &AgentEvent::Error {
            message: "unsupported provider stop_reason: fixture_unknown_reason".into(),
        },
    );
    assert_eq!(unknown.source, ProviderTerminalSource::UnsupportedReason);
    assert_eq!(unknown.reason.as_deref(), Some("fixture_unknown_reason"));

    let stop_sequence = diagnostic(
        "turn-stop-sequence",
        1,
        &AgentEvent::TurnEnd {
            stop_reason: StopReason::EndTurn,
            usage: Usage::default(),
        },
    );
    assert_eq!(stop_sequence.outcome, ProviderTerminalOutcome::Completed);
    assert_eq!(stop_sequence.source, ProviderTerminalSource::Explicit);
    assert_eq!(stop_sequence.reason, None);
}

#[test]
fn terminal_diagnostic_is_excluded_from_messages_copy_export_and_provider_history() {
    let temp_dir = tempfile::tempdir().expect("operation should succeed");
    let manager = SessionManager::with_dir(temp_dir.path().to_path_buf());
    let session = manager
        .create_session("exclusion", "")
        .expect("operation should succeed");
    session
        .append(&Message::User {
            content: "question".into(),
        })
        .expect("operation should succeed");
    session
        .append(&Message::Assistant {
            content: "answer".into(),
            tool_calls: vec![],
            reasoning: None,
        })
        .expect("operation should succeed");
    let terminal = AgentEvent::Error {
        message:
            "provider stream closed without explicit terminal signal ([DONE] or finish_reason)"
                .into(),
    };
    session
        .append_terminal_diagnostic(&diagnostic("turn-exclusion", 1, &terminal))
        .expect("operation should succeed");

    let provider_history = session.read_messages().expect("operation should succeed");
    assert_eq!(provider_history.len(), 2);
    let copy_projection = format!("{provider_history:?}");
    assert!(!copy_projection.contains("TERMINAL_DIAGNOSTIC"));
    assert!(!copy_projection.contains("missing_explicit_terminal"));

    let entries = session.read_entries().expect("operation should succeed");
    let json = export_json(&entries).expect("operation should succeed");
    let markdown = export_markdown(&entries);
    let transcript = read_transcript(&CompactTextSessionStore, &session.file_path)
        .expect("operation should succeed");
    for projection in [json, markdown, format!("{transcript:?}")] {
        assert!(!projection.contains("TERMINAL_DIAGNOSTIC"));
        assert!(!projection.contains("missing_explicit_terminal"));
    }
}

#[test]
fn recent_terminal_diagnostic_survives_compaction() {
    let temp_dir = tempfile::tempdir().expect("operation should succeed");
    let manager = SessionManager::with_dir(temp_dir.path().to_path_buf());
    let session = manager
        .create_session("compaction", "")
        .expect("operation should succeed");
    for index in 0..8 {
        session
            .append(&Message::User {
                content: format!("old message {index}"),
            })
            .expect("operation should succeed");
    }
    let terminal = AgentEvent::Error {
        message: "provider stream transport read error".into(),
    };
    session
        .append_terminal_diagnostic(&diagnostic("turn-recent", 2, &terminal))
        .expect("operation should succeed");

    let result = session
        .compact_archived(3)
        .expect("operation should succeed");
    assert!(matches!(result, CompactionResult::Compacted { .. }));
    let diagnostics = session
        .read_terminal_diagnostics()
        .expect("operation should succeed");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].turn_id, "turn-recent");
    assert_eq!(
        diagnostics[0].source,
        ProviderTerminalSource::TransportError
    );
}

#[test]
fn terminal_diagnostic_redacts_and_bounds_untrusted_fields() {
    let event = AgentEvent::Error {
        message: format!(
            "unsupported provider stop_reason: {}",
            "secret\n".repeat(100)
        ),
    };
    let diagnostic = ProviderTerminalDiagnostic::from_agent_event(
        &"t".repeat(300),
        0,
        &event,
        Some(&format!("provider\n{}", "p".repeat(300))),
        Some(&format!("model\r{}", "m".repeat(300))),
    )
    .expect("operation should succeed");

    assert!(diagnostic.turn_id.chars().count() <= 128);
    assert_eq!(diagnostic.response_ordinal, 1);
    assert!(
        diagnostic
            .reason
            .as_deref()
            .expect("operation should succeed")
            .chars()
            .count()
            <= 160
    );
    assert!(
        !diagnostic
            .reason
            .as_deref()
            .expect("operation should succeed")
            .contains('\n')
    );
    assert!(
        !diagnostic
            .provider
            .as_deref()
            .expect("operation should succeed")
            .contains('\n')
    );
    assert!(
        !diagnostic
            .model
            .as_deref()
            .expect("operation should succeed")
            .contains('\r')
    );
}
