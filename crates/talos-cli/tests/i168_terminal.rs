use talos_conversation::{ConversationEngine, TipKind, UiOutput};
use talos_core::message::{AgentEvent, StopReason, Usage};
use talos_core::session::TurnCompletionStatus;

#[test]
fn terminal_cli_projection_surfaces_truncation_and_clears_processing() {
    let mut engine = ConversationEngine::new("fixture-model".into(), "fixture-provider".into());
    engine.handle_turn_started();
    engine.handle_agent_event(&AgentEvent::TurnStart);
    let deltas = engine.handle_agent_event(&AgentEvent::TextDelta {
        delta: "partial response".into(),
    });
    let terminal = engine.handle_agent_event(&AgentEvent::TurnEnd {
        stop_reason: StopReason::MaxTokens,
        usage: Usage::default(),
    });

    assert!(deltas.iter().any(|output| matches!(
        output,
        UiOutput::Content(talos_conversation::ContentOutput::Delta { text })
            if text == "partial response"
    )));
    assert!(terminal.iter().any(|output| matches!(
        output,
        UiOutput::Tip {
            text,
            kind: TipKind::Error,
        } if text.contains("truncated") && text.contains("Partial response preserved")
    )));
    assert!(engine.is_processing());

    let completed = engine.handle_turn_completed(&TurnCompletionStatus::Success {
        final_text: "partial response".into(),
        new_messages: vec![],
    });
    assert!(completed.iter().any(|output| matches!(
        output,
        UiOutput::Status(status) if !status.is_processing
    )));
    assert!(!engine.is_processing());
}

#[test]
fn terminal_cli_projection_keeps_explicit_completion_quiet() {
    let mut engine = ConversationEngine::new("fixture-model".into(), "fixture-provider".into());
    engine.handle_turn_started();
    engine.handle_agent_event(&AgentEvent::TurnStart);
    engine.handle_agent_event(&AgentEvent::TextDelta {
        delta: "complete".into(),
    });
    let terminal = engine.handle_agent_event(&AgentEvent::TurnEnd {
        stop_reason: StopReason::EndTurn,
        usage: Usage::default(),
    });

    assert!(
        terminal
            .iter()
            .all(|output| !matches!(output, UiOutput::Tip { .. }))
    );
}

#[test]
fn terminal_cli_projection_preserves_known_provider_policy_causes() {
    for message in [
        "provider response filtered by content policy (finish_reason=content_filter)",
        "provider paused turn (stop_reason=pause_turn); automatic continuation is not supported",
        "provider refused request (stop_reason=refusal)",
        "unsupported provider stop_reason: fixture_unknown_reason",
    ] {
        let mut engine = ConversationEngine::new("fixture-model".into(), "fixture-provider".into());
        engine.handle_turn_started();
        engine.handle_agent_event(&AgentEvent::TurnStart);
        engine.handle_agent_event(&AgentEvent::TextDelta {
            delta: "partial response".into(),
        });
        let outputs = engine.handle_agent_event(&AgentEvent::Error {
            message: message.into(),
        });

        assert!(outputs.iter().any(|output| matches!(
            output,
            UiOutput::Tip {
                text,
                kind: TipKind::Error,
            } if text == message
        )));
        assert!(outputs.iter().any(|output| matches!(
            output,
            UiOutput::Content(talos_conversation::ContentOutput::Block { text, .. })
                if text.contains(message)
        )));
        assert!(!engine.is_processing());
    }
}
