use talos_conversation::ConversationEngine;
use talos_core::session::SubmissionKind;

fn prepare(
    engine: &mut ConversationEngine,
    text: &str,
) -> talos_core::session::StructuredSubmission {
    let (accepted, _) =
        engine.enqueue_structured_steering(text.to_owned(), SubmissionKind::UserTurn, Vec::new());
    assert!(accepted);
    engine
        .prepare_steering_submission()
        .expect("one queued steering item must produce a prepared submission")
}

#[test]
fn engine_rebuild_does_not_reuse_item_or_batch_identity() {
    let mut first = ConversationEngine::new("model".into(), "provider".into());
    let first_submission = prepare(&mut first, "same text");
    assert_eq!(first_submission.items[0].enqueue_sequence, 0);
    assert!(
        first.prepare_steering_submission().is_none(),
        "an existing reservation must remain frozen instead of being recreated"
    );

    let mut second = ConversationEngine::new("model".into(), "provider".into());
    let second_submission = prepare(&mut second, "same text");
    assert_eq!(second_submission.items[0].enqueue_sequence, 0);

    assert_ne!(first_submission.id, second_submission.id);
    assert_ne!(first_submission.items[0].id, second_submission.items[0].id);
    assert_eq!(
        first_submission.items[0].text,
        second_submission.items[0].text
    );
}

#[test]
fn distinct_engine_inputs_keep_fifo_sequences_but_never_share_durable_ids() {
    let mut first = ConversationEngine::new("model".into(), "provider".into());
    let first_submission = prepare(&mut first, "first payload");

    let mut second = ConversationEngine::new("model".into(), "provider".into());
    let second_submission = prepare(&mut second, "second payload");

    assert_eq!(first_submission.items[0].enqueue_sequence, 0);
    assert_eq!(second_submission.items[0].enqueue_sequence, 0);
    assert_ne!(first_submission.id, second_submission.id);
    assert_ne!(first_submission.items[0].id, second_submission.items[0].id);
}
