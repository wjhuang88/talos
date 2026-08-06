# ARCH-034-R02: CLI/TUI Bridge Decomposition

| Field | Value |
|---|---|
| Parent | ARCH-034 |
| Finding | ARCH-034-F21 |
| Status | Ready |
| Priority | P2 |
| Selected Iteration | Not selected |
| Preserved behavior | Event order, custody receipts, cancellation, session transitions, and TUI output |

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
