# RUNTIME-005: Bounded Graceful Shutdown And Structured Finalization

| Field | Value |
|---|---|
| Story ID | RUNTIME-005 |
| Type | Runtime / Lifecycle Story |
| Priority | P1 |
| Status | In Progress / Unclaimed — B/I216 Review; C remains Blocked |
| Source | [GitHub Issue #49](https://github.com/wjhuang88/talos/issues/49) |
| Selected Iteration | None |
| Depends On | SESSION-008 partial persistence; RUNTIME-001 embedded API |

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
| Last Updated | 2026-08-21 |
| Handoff / Release Condition | Epic parent is not an implementation unit. Complete B/I216 through its effective child claim, then establish a separate runnable C iteration/claim; keep Issue #49 open through C. |

## Identity / Goal / Value

Provide embedding hosts one idempotent, deadline-bounded shutdown contract that stops admission, resolves the active turn, finalizes durable state and runtime-owned resources, and returns a redacted structured report.

## Scope

- Caller-selected active-turn policy and total deadline.
- Exactly-once shared shutdown state for concurrent/repeated requests.
- Ordered durable reconciliation and bounded resource finalizers.
- Backward-compatible default `shutdown()` wrapper where feasible.

## Current Implementation Baseline (2026-08-09)

- `RuntimeHandle::shutdown(self)` sends `SessionOp::Shutdown` and waits for the
  actor task without a caller deadline or structured result.
- The actor cancels the active token, releases/pauses pending submissions and
  exits through its existing state machine. New admission, concurrent callers,
  an explicit active-turn policy, ordered generic finalizers and deadline
  exhaustion are not represented by one shared shutdown state.
- No background-job finalizer exists. TOOL-024 is a future consumer of this
  Story's generic finalizer contract, not a prerequisite for it.

## Exclusions

- No process signal ownership, SIGKILL guarantees, side-effect replay, or desktop UI.
- No second partial-turn format independent of SESSION-008.

## Dependencies

SESSION-008 partial persistence; RUNTIME-001 embedded API. TOOL-024 consumes the
completed shutdown/finalizer boundary and must not be a dependency of this Story.

## Executable Split

| ID | Deliverable | Status | Depends On |
|---|---|---|---|
| RUNTIME-005-A | Shutdown policy, arbitration, finalizer and report ADR | Complete / Closed through I214; Completion Commit `6719c876` | SESSION-008-A/B Complete; RUNTIME-001 Complete |
| RUNTIME-005-B | Shared admission/start arbiter, bounded active-turn policy and structured report | Review / Claimed in implementation PR #345; claim effective through #344 merge `2016acce` | RUNTIME-005-A Accepted; SESSION-008-B Complete |
| RUNTIME-005-C | Ordered bounded finalizer registry, durable reconciliation and compatibility wrapper | Blocked | RUNTIME-005-B Complete |

RUNTIME-005-A Completion Commit: `6719c876fe9f190e47fba5ef62f3263e782d6e8b`. This is child
decision evidence only; parent RUNTIME-005 remains In Progress/Unclaimed until B and C complete.

The finalizer registry must be proven with runtime-owned test finalizers. A
specific TOOL-024 implementation registers as a later consumer and cannot hold
#49 in a dependency cycle.

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
- docs/decisions/039-runtime-event-semantic-single-flow.md
- crates/talos-runtime/src/
- crates/talos-agent/src/session.rs
- crates/talos-agent/src/session/turn.rs

## Acceptance For Behavior / Technical Work

- Idle and active runtimes shut down within the supplied deadline.
- Concurrent/repeated shutdown calls observe one terminal report and run hooks once.
- Racing submissions are rejected deterministically.
- Durable finalization follows SESSION-008/ADR-042 filtering and never replays side effects.

## Residual Destination

If a public or durable-format break is required, stop and create an accepted ADR/migration plan.

## 2026-08-21 T10 Gate Reassessment

SESSION-008-A/B and RUNTIME-001 are Complete, so RUNTIME-005-A is now the smallest runnable
decision-only prerequisite and is selected as Planned I214 with its claim proposed in PR #336.
The proposal is ineffective before merge. RUNTIME-005-B remains
blocked until that decision is Accepted; C remains blocked on B. This checkpoint changes no
runtime behavior and does not authorize TOOL-024 or permission work.

## 2026-08-21 I214 Activation

PR #336 passed exact-head claim gates and merged as `7de582a3`; RUNTIME-005-A/I214 is now
Active/Claimed for the decision-only matrix and ADR. The parent remains Refinement/Unclaimed,
RUNTIME-005-B/C remain blocked, and no runtime or TOOL-024 implementation authority transfers.

## 2026-08-21 A Decision Execution

I214 produced the current-path matrix at
`docs/reference/I214-RUNTIME-SHUTDOWN-CURRENT-PATH.md` and Proposed ADR-063. The proposal makes B
the coordinator/admission/active-turn/deadline/report slice and C the ordered-finalizer/durable
reconciliation/compatibility slice. A remains Active pending exact-head independent architecture
review; B/C remain blocked and no production behavior is claimed.

PR #338 architecture review required B's admission fence to include the actor's start-commit
linearization and required invalid shutdown options to fail before a primary handle can be
consumed. The corrections remain decision-only and do not unblock B until ADR-063 is accepted.

## 2026-08-21 A Decision Acceptance

PR #338 corrected exact head `6719c876` passed CI `32449605985`, independent architecture review
`5365529351` and merge-time CAS, then merged as `fc70e396`. ADR-063 is Accepted and A/I214 is
Complete/Closed at that pre-existing Completion Commit. B is now Ready/Unclaimed but remains
unselected and unactivated until a runnable iteration and effective claim reach `main`; C remains
Blocked on B. Issue #49 stays open and no TOOL-024, permission, release or publication authority
transfers.

## 2026-08-21 B Claim Preparation

I216 and the child owner `RUNTIME-005-B` define the independently runnable coordinator/admission
slice already fixed by ADR-063. PR #344 proposes one atomic Claimed/Active record; it is
governance-only and ineffective until finalized exact-head CI, both validators, independent runtime
architecture review and merge-time CAS pass and the claim reaches `main`. Parent RUNTIME-005
remains Unclaimed; C and Issue #59 remain blocked, and no I189, TOOL-024, release or publication
authority transfers.

## 2026-08-21 B Claim Effective And Implementation Started

PR #344 exact head `e0f572a0` passed CI `32454558957`, independent architecture review
`5366165116`, both validators and merge-time CAS, then merged as `2016acce`. RUNTIME-005-B/I216 is
the only Active child and implementation started from that merge under its bounded Work Slice.
Parent RUNTIME-005 remains Unclaimed and In Progress; C remains Blocked until B is Complete. Issue
#49 stays open, and no I189, TOOL-024, release or publication authority transfers.
