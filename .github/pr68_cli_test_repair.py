from pathlib import Path

path = Path("crates/talos-cli/src/tests.rs")
text = path.read_text()

old_empty = '''    #[test]
    fn engine_snapshot_empty_after_drain() {
        let mut engine = new_engine();
        engine.enqueue_steering("a".into());
        engine.enqueue_steering("b".into());

        let drained = engine.drain_steering_queue_batched();
        let snap = engine.steering_queue_snapshot();
        assert_eq!(drained, Some("a\\n\\nb".into()));
        assert_eq!(snap.total_count, 0);
        assert_eq!(snap.omitted_count, 0);
        assert!(snap.entries.is_empty());

        let empty = build_empty_snapshot();
        assert_eq!(empty.total_count, 0);
        assert!(empty.entries.is_empty());
    }
'''
new_empty = '''    #[test]
    fn engine_snapshot_empty_after_structured_commit() {
        let mut engine = new_engine();
        engine.enqueue_steering("a".into());
        engine.enqueue_steering("b".into());

        let submission = engine
            .prepare_steering_submission()
            .expect("structured submission should be prepared");
        assert_eq!(
            submission
                .items
                .iter()
                .map(|item| item.text.as_str())
                .collect::<Vec<_>>(),
            vec!["a", "b"]
        );
        assert_eq!(
            engine.steering_queue_snapshot().total_count,
            2,
            "prepare must retain authoritative queue ownership"
        );
        assert!(engine.commit_prepared_steering(&submission.id));

        let snap = engine.steering_queue_snapshot();
        assert_eq!(snap.total_count, 0);
        assert_eq!(snap.omitted_count, 0);
        assert!(snap.entries.is_empty());

        let empty = build_empty_snapshot();
        assert_eq!(empty.total_count, 0);
        assert!(empty.entries.is_empty());
    }
'''
if old_empty in text:
    text = text.replace(old_empty, new_empty, 1)
elif new_empty not in text:
    raise SystemExit("unexpected empty-after-drain CLI test")

old_batch = '''    #[test]
    fn engine_batched_drain_joins_multiple_messages() {
        let mut engine = new_engine();
        engine.enqueue_steering("a".into());
        engine.enqueue_steering("b".into());
        engine.enqueue_steering("c".into());

        let msg = engine
            .drain_steering_queue_batched()
            .expect("batched drain should join all messages");
        assert_eq!(msg, "a\\n\\nb\\n\\nc");

        let snap = engine.steering_queue_snapshot();
        assert_eq!(
            snap.total_count, 0,
            "queue must be empty after batched drain"
        );
        assert!(snap.entries.is_empty());
    }
'''
new_batch = '''    #[test]
    fn engine_structured_prepare_preserves_multiple_messages() {
        let mut engine = new_engine();
        engine.enqueue_steering("a".into());
        engine.enqueue_steering("b".into());
        engine.enqueue_steering("c".into());

        let submission = engine
            .prepare_steering_submission()
            .expect("structured submission should include the FIFO prefix");
        assert_eq!(
            submission
                .items
                .iter()
                .map(|item| item.text.as_str())
                .collect::<Vec<_>>(),
            vec!["a", "b", "c"],
            "items must remain distinct and ordered"
        );
        assert_eq!(
            engine.steering_queue_snapshot().total_count,
            3,
            "prepare must not destructively drain"
        );
        assert!(engine.commit_prepared_steering(&submission.id));

        let snap = engine.steering_queue_snapshot();
        assert_eq!(snap.total_count, 0, "acknowledged prefix must be committed");
        assert!(snap.entries.is_empty());
    }
'''
if old_batch in text:
    text = text.replace(old_batch, new_batch, 1)
elif new_batch not in text:
    raise SystemExit("unexpected joined-drain CLI test")

path.write_text(text)
