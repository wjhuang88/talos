from pathlib import Path

path = Path('crates/talos-cli/src/tests.rs')
text = path.read_text()

old_first = '''    #[test]
    fn engine_snapshot_empty_after_drain() {
        let mut engine = new_engine();
        engine.enqueue_steering("a".into());
        engine.enqueue_steering("b".into());

        let drained = engine.drain_steering_queue_batched();
        let snap = engine.steering_queue_snapshot();
        assert_eq!(drained, Some("a\n\nb".into()));
        assert_eq!(snap.total_count, 0);
        assert_eq!(snap.omitted_count, 0);
        assert!(snap.entries.is_empty());

        let empty = build_empty_snapshot();
        assert_eq!(empty.total_count, 0);
        assert!(empty.entries.is_empty());
    }
'''
new_first = '''    #[test]
    fn engine_snapshot_empty_after_structured_commit() {
        let mut engine = new_engine();
        engine.enqueue_steering("a".into());
        engine.enqueue_steering("b".into());

        let prepared = engine
            .prepare_steering_submission()
            .expect("queued messages should prepare as a structured submission");
        let texts: Vec<&str> = prepared
            .items
            .iter()
            .map(|item| item.text.as_str())
            .collect();
        assert_eq!(texts, vec!["a", "b"]);
        assert_eq!(
            engine.steering_queue_snapshot().total_count,
            2,
            "prepare must retain the authoritative queue until acknowledgement"
        );
        assert!(engine.commit_prepared_steering(&prepared.id));

        let snap = engine.steering_queue_snapshot();
        assert_eq!(snap.total_count, 0);
        assert_eq!(snap.omitted_count, 0);
        assert!(snap.entries.is_empty());

        let empty = build_empty_snapshot();
        assert_eq!(empty.total_count, 0);
        assert!(empty.entries.is_empty());
    }
'''
if old_first in text:
    text = text.replace(old_first, new_first, 1)
elif new_first not in text:
    raise SystemExit('expected first legacy batched-drain CLI test')

old_second = '''    #[test]
    fn engine_batched_drain_joins_multiple_messages() {
        let mut engine = new_engine();
        engine.enqueue_steering("a".into());
        engine.enqueue_steering("b".into());
        engine.enqueue_steering("c".into());

        let msg = engine
            .drain_steering_queue_batched()
            .expect("batched drain should join all messages");
        assert_eq!(msg, "a\n\nb\n\nc");

        let snap = engine.steering_queue_snapshot();
        assert_eq!(
            snap.total_count, 0,
            "queue must be empty after batched drain"
        );
        assert!(snap.entries.is_empty());
    }
'''
new_second = '''    #[test]
    fn engine_structured_prepare_preserves_multiple_messages() {
        let mut engine = new_engine();
        engine.enqueue_steering("a".into());
        engine.enqueue_steering("b".into());
        engine.enqueue_steering("c".into());

        let prepared = engine
            .prepare_steering_submission()
            .expect("queued messages should prepare as one structured submission");
        let texts: Vec<&str> = prepared
            .items
            .iter()
            .map(|item| item.text.as_str())
            .collect();
        assert_eq!(texts, vec!["a", "b", "c"]);
        assert_eq!(
            engine.steering_queue_snapshot().total_count,
            3,
            "prepare must not destructively drain the authoritative queue"
        );
        assert!(engine.commit_prepared_steering(&prepared.id));

        let snap = engine.steering_queue_snapshot();
        assert_eq!(snap.total_count, 0, "queue must be empty after commit");
        assert!(snap.entries.is_empty());
    }
'''
if old_second in text:
    text = text.replace(old_second, new_second, 1)
elif new_second not in text:
    raise SystemExit('expected second legacy batched-drain CLI test')

path.write_text(text)
