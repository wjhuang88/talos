# ARCH-011: Architecture Watchlist

**Status**: Tracking (reviewed 2026-06-19)
**Priority**: P4
**Source**: Post-ARCH-005 architecture follow-up
**Depends on**: I029 complete

## Problem

After the I029 decomposition pass, several files are not yet clear execution stories but are worth
watching because future feature work may grow them into new god modules. These files should be
tracked explicitly so Agents do not turn observation-only concerns into speculative refactors.

## Watchlist

| File | Current Concern | Promotion Trigger |
| --- | --- | --- |
| `crates/talos-agent/src/tests.rs` | Large test aggregation after agent decomposition. | Promote only if test maintenance becomes difficult or repeated features touch unrelated test clusters. |
| `crates/talos-agent/src/prompt.rs` | Prompt rendering, template slots, cache boundary, and provider-specific prompt concerns are close together. | Promote if new prompt features add another independent responsibility or cache/provider logic starts leaking into call sites. |
| `crates/talos-tui/src/scrollback.rs` | Central rendering path for history cells and Markdown output. | Promote if rendering modes, hidden tool output, or approval/history display introduce separable responsibilities. |

## Scope

- Keep this as an observation record.
- Do not refactor these files merely because they are listed here.
- Promote a file to its own backlog item only when there is concrete evidence: file growth,
  repeated unrelated changes, test brittleness, or an architectural boundary violation.

## Acceptance Criteria

- [x] Watchlist is reviewed during future architecture cleanup sessions.
- [x] A watched file is promoted only with a new owner story and explicit acceptance criteria.
- [x] No code changes are made directly under this item.

## Verification Notes

This is a governance tracking item. Validation is documentation consistency plus future review
evidence, not code execution.

2026-06-19 review after ARCH-010/I032 closure:

- `crates/talos-agent/src/tests.rs` remains watch-only. It is large, but no current architecture
  task is blocked by its shape.
- `crates/talos-agent/src/prompt.rs` remains watch-only. Prompt template/cache work is stable after
  I026/I029; no new provider-specific leakage was observed in this architecture pass.
- `crates/talos-tui/src/scrollback.rs` remains watch-only. Rendering growth is expected to be driven
  by future TUI input-layer and hidden-output work, so promotion should wait for concrete change
  pressure.
- No watched file was promoted to a new owner story in this pass.
