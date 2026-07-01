# TUI-018: Context Limit Million-Unit Format

| Field | Value |
|-------|-------|
| Story ID | TUI-018 |
| Priority | P3 |
| Status | Planned |
| Source | [GitHub Issue #11](https://github.com/wjhuang88/talos/issues/11) |
| Relates To | TUI-011 |

## Requirement

Format million-token context windows as `1M ctx`, `2M ctx`, etc. instead of `1000k ctx`.

## Scope

- Update context limit formatting in the status bar.
- Preserve existing `k ctx` display below one million tokens.
- Keep unknown limits hidden.

## Acceptance Criteria

- [ ] `1_000_000` renders as `1M ctx`.
- [ ] `2_000_000` renders as `2M ctx`.
- [ ] `200_000` remains `200k ctx`.
- [ ] Unit tests cover M, k, raw, and none cases.

## Required Reads

- [GitHub Issue #11](https://github.com/wjhuang88/talos/issues/11)
- `crates/talos-tui/src/scrollback_status.rs`
