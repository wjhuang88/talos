#!/usr/bin/env python3
"""Remove the delimiter-authoritative steering batch compatibility surface.

The structured prepare/commit/rollback transaction remains authoritative. The
legacy single-item FIFO drain remains available for downstream compatibility.
Every anchor is validated so repository drift fails closed.
"""

from pathlib import Path


ENGINE = Path("crates/talos-conversation/src/engine.rs")
TESTS = Path("crates/talos-conversation/src/engine_tests.rs")


def replace_once(source: str, old: str, new: str, label: str) -> str:
    count = source.count(old)
    if count != 1:
        raise SystemExit(f"expected exactly one {label}, found {count}")
    return source.replace(old, new, 1)


def function_span(source: str, signature: str, *, include_attribute: bool = False) -> tuple[int, int]:
    signature_index = source.find(signature)
    if signature_index < 0:
        raise SystemExit(f"missing function signature: {signature}")
    start = signature_index
    if include_attribute:
        attribute = source.rfind("#[test]", 0, signature_index)
        if attribute < 0 or source[attribute:signature_index].strip() != "#[test]":
            raise SystemExit(f"missing adjacent #[test] for {signature}")
        start = attribute
    opening = source.find("{", signature_index)
    if opening < 0:
        raise SystemExit(f"missing opening brace for {signature}")
    depth = 0
    in_string = False
    escaped = False
    for index in range(opening, len(source)):
        char = source[index]
        if in_string:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                in_string = False
            continue
        if char == '"':
            in_string = True
        elif char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                end = index + 1
                while end < len(source) and source[end] == "\n":
                    end += 1
                return start, end
    raise SystemExit(f"unterminated function: {signature}")


def remove_function(source: str, signature: str, *, include_attribute: bool = False) -> str:
    start, end = function_span(source, signature, include_attribute=include_attribute)
    return source[:start] + source[end:]


def replace_test(source: str, name: str, replacement: str) -> str:
    signature = f"fn {name}()"
    start, end = function_span(source, signature, include_attribute=True)
    return source[:start] + replacement.rstrip() + "\n\n" + source[end:]


def main() -> None:
    engine = ENGINE.read_text()
    old_doc = '''    /// Drains the oldest steering message while preserving FIFO order.
    ///
    /// This method remains available for downstream callers that need the
    /// original single-item behavior. Talos's interactive runtime uses
    /// [`Self::drain_steering_queue_batched`] for TUI-041 / GitHub Issue #50.
'''
    new_doc = '''    /// Drains the oldest steering message while preserving FIFO order.
    ///
    /// This method remains available only for legacy single-item callers.
    /// Transactional TUI steering uses structured prepare/commit/rollback.
'''
    engine = replace_once(engine, old_doc, new_doc, "legacy drain documentation")

    signature = "pub fn drain_steering_queue_batched(&mut self) -> Option<String>"
    function_start, function_end = function_span(engine, signature)
    doc_start = engine.rfind("    /// Drains the entire steering queue", 0, function_start)
    if doc_start < 0:
        raise SystemExit("missing delimiter batch documentation")
    between = engine[doc_start:function_start]
    if "joins them" not in between or "`\\n\\n`" not in between:
        raise SystemExit("unexpected delimiter batch documentation")
    engine = engine[:doc_start] + engine[function_end:]

    if "drain_steering_queue_batched" in engine:
        raise SystemExit("delimiter batch symbol remains in Engine")
    if '.join("\\n\\n")' in engine:
        raise SystemExit("delimiter-joined steering representation remains in Engine")
    ENGINE.write_text(engine)

    tests = TESTS.read_text()
    tests = replace_once(
        tests,
        "// drain_steering_queue / drain_steering_queue_batched",
        "// drain_steering_queue / structured steering transaction",
        "steering test section header",
    )
    tests = replace_once(
        tests,
        '''    assert_eq!(
        engine.drain_steering_queue_batched(),
        Some("queued".to_string())
    );''',
        '''    assert_eq!(engine.drain_steering_queue(), Some("queued".to_string()));''',
        "single-item enqueue drain assertion",
    )

    for name in (
        "drain_steering_queue_batched_joins_all_with_separator",
        "drain_steering_queue_batched_single_message",
        "drain_steering_queue_batched_none_when_empty",
    ):
        tests = remove_function(tests, f"fn {name}()", include_attribute=True)

    tests = replace_test(
        tests,
        "steering_queue_snapshot_reflects_drain",
        '''#[test]
fn steering_queue_snapshot_reflects_legacy_single_drain() {
    let mut engine = new_engine();
    engine.enqueue_steering("first".into());
    engine.enqueue_steering("second".into());

    let drained = engine.drain_steering_queue();
    assert_eq!(drained, Some("first".to_string()));

    let snap = engine.steering_queue_snapshot();
    assert_eq!(snap.total_count, 1);
    assert_eq!(snap.entries.len(), 1);
    assert_eq!(snap.entries[0].text, "second");
}''',
    )

    if "drain_steering_queue_batched" in tests:
        raise SystemExit("delimiter batch references remain in Engine tests")
    TESTS.write_text(tests)


if __name__ == "__main__":
    main()
