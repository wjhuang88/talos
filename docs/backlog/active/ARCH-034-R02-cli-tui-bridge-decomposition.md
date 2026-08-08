# ARCH-034-R02: CLI/TUI Bridge Decomposition

> Document status: Complete

| Field | Value |
|---|---|
| Parent | ARCH-034 |
| Finding | ARCH-034-F21 |
| Status | Complete |
| Priority | P2 |
| Selected Iteration | I172 (Complete; implementation PR #144, closeout PR #147) |
| Preserved behavior | Event order, custody receipts, cancellation, session transitions, and TUI output |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 active architecture session 2026-08-06 |
| Work Slice | Extract only the legacy `TurnEvent` and structured-legacy compatibility projection handlers into a private module behind the existing `tui_bridge` entry points; preserve event ordering, state transitions, channel topology, custody, cancellation, and all output text. |
| Claimed At | 2026-08-06 |
| Source Issue | None |
| Governance Claim PR | #140 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | No independent reviewer is currently available; exact-head CI, both governance validators, merge-time CAS, and no blocking review feedback are required. |
| Implementation PR | #144 (merged at `c1dc67ae`) |
| Last Updated | 2026-08-07 |
| Handoff / Release Condition | Closed after implementation PR #144 and closeout PR #147 merged; any remaining bridge responsibility requires a separately bounded owner and claim. |

## Problem And Boundary

`talos-cli/src/tui_bridge.rs` is a 2,091-line, high-change orchestration seam. It owns the
conversation loop, session event dispatch, structured/legacy event projection, receipt/custody
translation, and cancellation. The CLI/TUI boundary remains the correct owner; the source file is
low-cohesion inside that boundary.

## Scope

- Extract private event-family and projection modules behind the existing bridge entry points.
- Preserve channel topology, select ordering, event sequencing, error strings, and visibility.
- Keep session mutation and durable custody ownership unchanged.

## Exclusions

- No new event bus, channel type, protocol, public API, command, UI behavior, or dependency.
- No session actor/persistence changes and no rewrite of I169 lifecycle logic.

## Acceptance And Validation

- Each extracted module has one named responsibility and no circular module imports.
- Before/after event sequences are identical in I169 bridge integration tests.
- `tui_bridge.rs` becomes a coordinator facade; no behavior branch is removed or reordered.
- Locked fmt/check/all-target Clippy/workspace tests, TUI smoke, governance, and diff checks pass.

## Rollback / Residual

Revert the private extraction if ordering equivalence cannot be proven. Protocol redesign belongs
to a separate ADR-backed story.

## Completion Evidence

- Completion Commit: `4084138dc0652d3200045847d42518d9ecb66231`.
- Implementation PR #144 merged at `c1dc67ae8e3a117dd39ede91143c5f6bcd2d17c4`.
- Exact-head CI run `31137882248` passed Unix format/check/clippy/test, Windows workspace,
  Windows installer, and remote issue/owner reconciliation checks.
- Closeout PR #147 merged accepted head `19fa33262daa5ed78aa30db5c00818495bff5b82` as
  `cc74360147753ab75685f76f9abaeed6b990fa52`; exact-head CI `31138811524` passed.
- `crates/talos-cli/src/tui_bridge/legacy_projection.rs` owns the extracted legacy and
  structured-legacy handlers while `tui_bridge.rs` retains the facade and runtime entry points.
- Source-layout and focused CLI tests passed; public API, event order, custody, cancellation,
  permissions, persistence, and output behavior were preserved.
