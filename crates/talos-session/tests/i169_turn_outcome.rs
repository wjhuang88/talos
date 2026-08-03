use talos_core::message::Message;
use talos_session::{SessionManager, TurnTranscriptOutcome, TurnTranscriptOutcomeRecord};

#[test]
fn transcript_outcome_markers_are_hidden_and_only_success_binds_turn_identity() {
    let temp = tempfile::tempdir().unwrap();
    let manager = SessionManager::with_dir(temp.path().join("sessions"));
    let session = manager
        .create_session("i169", temp.path().to_string_lossy().as_ref())
        .expect("create session");

    session
        .append(&Message::User {
            content: "visible user message".into(),
        })
        .expect("append user message");
    session
        .append_turn_transcript_outcome(&TurnTranscriptOutcomeRecord::new(
            "turn-success",
            TurnTranscriptOutcome::Success,
        ))
        .expect("append success outcome");
    session
        .append_turn_transcript_outcome(&TurnTranscriptOutcomeRecord::new(
            "turn-error",
            TurnTranscriptOutcome::Error,
        ))
        .expect("append error outcome");
    session
        .append_turn_transcript_outcome(&TurnTranscriptOutcomeRecord::new(
            "turn-cancelled",
            TurnTranscriptOutcome::Cancelled,
        ))
        .expect("append cancelled outcome");

    assert_eq!(
        session.read_messages().expect("read visible transcript"),
        vec![Message::User {
            content: "visible user message".into(),
        }]
    );

    assert_eq!(
        session
            .read_turn_transcript_outcomes()
            .expect("read outcome evidence"),
        vec![
            TurnTranscriptOutcomeRecord::new("turn-success", TurnTranscriptOutcome::Success),
            TurnTranscriptOutcomeRecord::new("turn-error", TurnTranscriptOutcome::Error),
            TurnTranscriptOutcomeRecord::new("turn-cancelled", TurnTranscriptOutcome::Cancelled,),
        ]
    );

    let outcome_entries: Vec<_> = session
        .read_entries()
        .expect("read raw entries")
        .into_iter()
        .filter(|entry| {
            entry
                .content
                .starts_with("__TALOS_TURN_TRANSCRIPT_OUTCOME__:")
        })
        .collect();
    assert_eq!(outcome_entries.len(), 3);
    assert_eq!(
        outcome_entries[0].metadata.turn_id.as_deref(),
        Some("turn-success")
    );
    assert_eq!(outcome_entries[1].metadata.turn_id, None);
    assert_eq!(outcome_entries[2].metadata.turn_id, None);
}
