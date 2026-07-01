# TUI-019: Tool Output Visual Hierarchy

| Field | Value |
|-------|-------|
| Story ID | TUI-019 |
| Priority | P3 |
| Status | Review |
| Source | [GitHub Issue #14](https://github.com/wjhuang88/talos/issues/14) |
| Relates To | TUI-007, TOOL-015 |

## Requirement

Tool output rendering should distinguish primary result lines from secondary detail lines.

## Scope

- Keep primary status/result lines visually prominent.
- Render detail/preview lines with a softer semantic style.
- Prefer existing theme roles before adding new theme fields.

## Acceptance Criteria

- [x] Status/result line uses primary result style.
- [x] Detail lines use secondary result style.
- [x] Existing themes remain readable.
- [x] Tests cover style classification.

## Execution Notes

- 2026-07-01: Implemented in I076/T105. Tool result first lines now use primary result styling; non-error detail/preview lines use the existing dim semantic style.

## Verification Evidence

- 2026-07-01: `cargo test -p talos-tui tool_result` passed: 4 tests.
- 2026-07-01: `cargo test -p talos-tools file_tool_tests` passed: 22 tests.
- 2026-07-01: `cargo test -p talos-tui` passed: 182 unit tests, 2 doc tests.
- 2026-07-01: `cargo check --workspace` passed.
- 2026-07-01: `cargo clippy -p talos-tools -p talos-tui -- -D warnings` passed.

## Required Reads

- [GitHub Issue #14](https://github.com/wjhuang88/talos/issues/14)
- `docs/backlog/active/TUI-007-theme-system.md`
- `docs/backlog/active/TOOL-015-write-edit-result-visibility.md`
- `crates/talos-tui/src/tool_display.rs`
- `crates/talos-tui/src/theme.rs`
