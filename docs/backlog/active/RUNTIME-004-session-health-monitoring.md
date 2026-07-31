# RUNTIME-004: Session Health Monitoring And Recovery Boundary

| Field | Value |
|---|---|
| Story ID | RUNTIME-004 |
| Type | Runtime / Reliability Spike |
| Priority | P2 |
| Status | Refinement — monitoring and recovery authority require design |
| Source | [GitHub Issue #32](https://github.com/wjhuang88/talos/issues/32) |
| Selected Iteration | None |
| Depends On | RUNTIME-002 turn health; RUNTIME-003 terminal integrity; OBS-002 pipeline observability |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #32 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-01 |
| Handoff / Release Condition | Establish an effective claim and select an iteration before implementation. |

## Identity / Goal / Value

Give operators a bounded way to detect stalled turns and unhealthy runtime state without adding an independent lifecycle authority that can race the canonical session loop.

## Scope

- Inventory existing provider/tool timeouts, progress events, cancellation, diagnostics, and terminal outcomes.
- Define passive health signals and user-visible warnings.
- Define which recovery actions are advisory, user-triggered, or safe to automate.

## Exclusions

- No watchdog that force-mutates session state outside the ordered runtime seam.
- No automatic replay of side effects, silent model switching, or unbounded background thread.

## Dependencies

RUNTIME-002 turn health; RUNTIME-003 terminal integrity; OBS-002 pipeline observability

## Decision Links And Constraints

- The canonical session/turn owner remains authoritative.
- Monitoring is read-only unless a reviewed command is routed through the existing lifecycle API.
- Timeout and terminal classifications must not be duplicated.

## Uncertainty And Validation Path

Refine against current RUNTIME-002/RUNTIME-003 behavior, then decide whether any unmet monitoring gap justifies a new ADR and iteration.

## State / Status Owners

- Story status and acceptance: this file.
- Remote request state and discussion: GitHub Issue #32.
- Current operating view: `docs/BOARD.md`.
- Compact selection view: `docs/backlog/PRODUCT-BACKLOG.md`.

## User-Facing Documentation

Update user or SDK documentation only when observable behavior or a public integration contract changes.
Do not present this Story as shipped while it remains Refinement.

## Required Reads

- docs/backlog/active/RUNTIME-002-turn-health-and-stuck-processing.md
- docs/backlog/active/RUNTIME-003-provider-terminal-outcome-integrity.md
- docs/backlog/active/OBS-002-turn-pipeline-boundary-observability.md
- crates/talos-agent/src/session/

## Acceptance For Behavior / Technical Work

- A current-state trace distinguishes already-shipped detection from residual gaps.
- Any recovery action has a single owner, bounded deadline, and no replay ambiguity.
- User-visible diagnostics are redacted and clear processing deterministically.

## Residual Destination

If passive diagnostics are sufficient, close as not selected; otherwise create a bounded child Story with an ADR.
