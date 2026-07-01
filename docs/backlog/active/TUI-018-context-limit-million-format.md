# TUI-018: Context Limit Million-Unit Format

| Field | Value |
|-------|-------|
| Story ID | TUI-018 |
| Priority | P3 |
| Status | Complete |
| Source | [GitHub Issue #11](https://github.com/wjhuang88/talos/issues/11) |
| Relates To | TUI-011 |

## Requirement

Format million-token context windows as `1M ctx`, `2M ctx`, etc. instead of `1000k ctx`.

## Scope

- Update context limit formatting in the status bar.
- Preserve existing `k ctx` display below one million tokens.
- Keep unknown limits hidden.

## Acceptance Criteria

- [x] `1_000_000` renders as `1M ctx`.
- [x] `2_000_000` renders as `2M ctx`.
- [x] `200_000` remains `200k ctx`.
- [x] Unit tests cover M, k, raw, and none cases.

## Execution Notes

- 2026-07-01: Activated in I076/T102. Status bar formatting implementation is in progress; verification pending.
- 2026-07-01: Moved to Review. Million-token context limits now render with `M ctx`; sub-million and unknown limits retain prior behavior.
- 2026-07-01: Moved to Complete during I076/T109 closeout after full workspace validation passed.

## Verification Evidence

- 2026-07-01: `cargo test -p talos-tui status_bar` passed: 14 status-bar tests.
- 2026-07-01: `cargo test -p talos-tui` passed: 180 unit tests, 2 doc tests.
- 2026-07-01: `cargo clippy -p talos-provider -p talos-tui -- -D warnings` passed.
- 2026-07-01: `cargo test --workspace` passed during I076/T109 closeout.
- 2026-07-01: `scripts/validate_project_governance.sh .` passed with 0 warnings during I076/T109 closeout.

## Required Reads

- [GitHub Issue #11](https://github.com/wjhuang88/talos/issues/11)
- `crates/talos-tui/src/scrollback_status.rs`
