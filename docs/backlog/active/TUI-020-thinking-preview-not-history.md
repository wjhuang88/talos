# TUI-020: Thinking Preview Without History Pollution

| Field | Value |
|-------|-------|
| Story ID | TUI-020 |
| Priority | P2 |
| Status | Planned |
| Source | [GitHub Issue #15](https://github.com/wjhuang88/talos/issues/15) |
| Relates To | TUI-004, SESSION-002 |

## Requirement

Model thinking content should be visible in the live preview area while streaming, but should not be
inserted into scrollback history or persisted as normal conversation history.

## Scope

- Keep active thinking state separate from finalized history.
- Clear thinking preview when the assistant response finalizes.
- Ensure persisted sessions contain final assistant output, not transient thinking text.

## Decision Point

Choose between an explicit transient message variant and a streaming-state-only field before
implementation. The first slice should prefer the smallest change that preserves session history
integrity.

## Acceptance Criteria

- [ ] Thinking is visible during active streaming.
- [ ] Thinking does not appear in finalized history.
- [ ] Thinking is not persisted as normal session history.
- [ ] Resume does not replay old thinking content.
- [ ] Tests cover stream, finalization, persistence, and resume.

## Required Reads

- [GitHub Issue #15](https://github.com/wjhuang88/talos/issues/15)
- `docs/backlog/active/TUI-004-state-model.md`
- `docs/backlog/active/SESSION-002-session-integrity-lifecycle-hardening.md`
- `crates/talos-conversation/src/`
- `crates/talos-tui/src/`
- `crates/talos-session/src/`
