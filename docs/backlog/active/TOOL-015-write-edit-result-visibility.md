# TOOL-015: Write And Edit Result Visibility

| Field | Value |
|-------|-------|
| Story ID | TOOL-015 |
| Priority | P2 |
| Status | Review |
| Source | [GitHub Issue #13](https://github.com/wjhuang88/talos/issues/13) |
| Relates To | TOOL-003, TUI-019 |

## Requirement

`write` and `edit` tool results should show useful write/edit content so users can verify file
changes without opening the file manually.

## Scope

- For `write`, show target path, byte count, and bounded content preview.
- For `edit`, show a bounded diff of the replacement.
- Keep model-facing raw result bounded and deterministic.
- Preserve full file content outside the display path only when explicitly returned by the tool.

## Acceptance Criteria

- [x] `write` success includes path, size, and bounded preview.
- [x] `edit` success includes a readable diff.
- [x] Large outputs are truncated predictably.
- [x] TUI rendering remains readable.
- [x] Unit tests cover write preview and edit diff output.

## Execution Notes

- 2026-07-01: Implemented in I076/T104. `write` now returns path, byte count, and bounded preview; `edit` now returns a bounded replacement diff.

## Verification Evidence

- 2026-07-01: `cargo test -p talos-tools file_tool_tests` passed: 22 tests.
- 2026-07-01: `cargo test -p talos-tools` passed: 200 unit tests, 15 document-boundary tests, 3 integration-hardening tests.
- 2026-07-01: `cargo test -p talos-tui tool_result` passed: 4 tests.
- 2026-07-01: `cargo check --workspace` passed.
- 2026-07-01: `cargo clippy -p talos-tools -p talos-tui -- -D warnings` passed.

## Required Reads

- [GitHub Issue #13](https://github.com/wjhuang88/talos/issues/13)
- `docs/backlog/active/TOOL-003-posix-tool-set.md`
- `crates/talos-tools/src/file_tools/`
- `crates/talos-tui/src/tool_display.rs`
