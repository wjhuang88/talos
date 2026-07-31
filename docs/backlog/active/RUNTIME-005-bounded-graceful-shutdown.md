# RUNTIME-005: Bounded Graceful Shutdown And Structured Finalization

| Field | Value |
|---|---|
| Story ID | RUNTIME-005 |
| Type | Runtime / Lifecycle Story |
| Priority | P1 |
| Status | Refinement — shutdown policy and finalization order require ADR review |
| Source | [GitHub Issue #49](https://github.com/wjhuang88/talos/issues/49) |
| Selected Iteration | None |
| Depends On | SESSION-008 partial persistence; RUNTIME-001 embedded API; TOOL-024 resource shutdown |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #49 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-01 |
| Handoff / Release Condition | Establish an effective claim and select an iteration before implementation. |

## Identity / Goal / Value

Provide embedding hosts one idempotent, deadline-bounded shutdown contract that stops admission, resolves the active turn, finalizes durable state and runtime-owned resources, and returns a redacted structured report.

## Scope

- Caller-selected active-turn policy and total deadline.
- Exactly-once shared shutdown state for concurrent/repeated requests.
- Ordered durable reconciliation and bounded resource finalizers.
- Backward-compatible default `shutdown()` wrapper where feasible.

## Exclusions

- No process signal ownership, SIGKILL guarantees, side-effect replay, or desktop UI.
- No second partial-turn format independent of SESSION-008.

## Dependencies

SESSION-008 partial persistence; RUNTIME-001 embedded API; TOOL-024 resource shutdown

## Decision Links And Constraints

- The host owns process lifecycle; Talos exposes a runtime finalization API only.
- Cancellation/completion/persistence arbitration uses the canonical session seam.
- Reports exclude prompts, secrets, reasoning, raw arguments, and raw output.

## Uncertainty And Validation Path

Choose wait/interrupt/abort semantics, finalizer registration ownership, deadline accounting, and compatibility behavior in an ADR before implementation.

## State / Status Owners

- Story status and acceptance: this file.
- Remote request state and discussion: GitHub Issue #49.
- Current operating view: `docs/BOARD.md`.
- Compact selection view: `docs/backlog/PRODUCT-BACKLOG.md`.

## User-Facing Documentation

Update user or SDK documentation only when observable behavior or a public integration contract changes.
Do not present this Story as shipped while it remains Refinement.

## Required Reads

- docs/backlog/active/SESSION-008-interrupted-turn-partial-persistence.md
- docs/backlog/active/RUNTIME-001-embeddable-agent-runtime-api.md
- docs/backlog/active/TOOL-024-background-command-jobs.md
- docs/decisions/039-session-integrity-and-lifecycle-semantics.md
- crates/talos-runtime/src/

## Acceptance For Behavior / Technical Work

- Idle and active runtimes shut down within the supplied deadline.
- Concurrent/repeated shutdown calls observe one terminal report and run hooks once.
- Racing submissions are rejected deterministically.
- Durable finalization follows SESSION-008/ADR-042 filtering and never replays side effects.

## Residual Destination

If a public or durable-format break is required, stop and create an accepted ADR/migration plan.
