# Iteration I251: Pre-Provider Rejection Terminalization

> Document status: Planned
> Published plan date: 2026-09-08
> Planned objective: Convert deterministic pre-Provider rejection into an idempotent terminal lifecycle outcome so model/provider switching is not blocked by stale active submission state.
> Baseline rule: preserve this target; a changed objective requires a new iteration ID.
> MVP deliverable: A runnable regression path where `ContextBudgetExceeded` terminalizes before Provider start, preserves unrelated queue identity/order, and permits model switching when no legitimate work remains.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | `@wjhuang88` (shared account; Agent-role execution, no natural-person identity claim) |
| Executing Agent | Codex Agent / mainline unattended session |
| Work Slice | Runtime submission lifecycle only: deterministic non-resumable pre-Provider rejection terminalization, exactly-once release, model-switch guard, and focused regression evidence for Issue #499. |
| Claimed At | 2026-09-08 |
| Source Issue | #499 |
| Governance Claim PR | Pending atomic claim PR |
| Authorization Mode | Independent review |
| Authorization Evidence | Pending exact-head governance validation and independent review |
| Implementation PR | Not started |
| Last Updated | 2026-09-08 |
| Handoff / Release Condition | Claim becomes effective only when this activation record reaches `main`; implementation starts from that merge or a later `main` commit. #466 remains a separate architecture-only parent. |

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| RUNTIME-007 | #499 | Planned | Runtime submission lifecycle characterization; #408 stale-queue relationship review | Deterministic pre-Provider rejection is terminal, idempotent, queue-preserving, and no longer blocks model/provider switching. |

### Scope

- Characterize the actor, pending-submission store, TUI bridge, and model-switch guard.
- Terminalize only deterministic non-resumable pre-Provider rejection (`ContextBudgetExceeded`); retain genuinely resumable paused/blocked states.
- Release active submission state exactly once and preserve unrelated queued work identity/order.
- Add focused runtime/session/model-switch regression tests and update behavior-facing documentation where required.

### Non-Goals

- No automatic context compaction, provider selection, permission/sandbox changes, dependency upgrade, or TUI-only shortcut.
- No queue clearing, replay ghost turn, public API redesign, Desktop/Dashboard work, or #466 capability architecture implementation.

### Acceptance

- `ContextBudgetExceeded` before Provider start emits one terminal outcome without a Provider call or synthetic user cancel.
- Model/provider switching succeeds when no legitimate queued/runnable work remains.
- Unrelated steering/scheduler queue entries retain identity and order.
- Duplicate/stale lifecycle events are harmless; replay/resume creates no ghost turn.
- Provider-started behavior remains unchanged and covered by regression tests.

### Planned Validation

- Focused pending-submission, session custody, model-switch and TUI bridge tests.
- `cargo fmt --all -- --check`, locked focused tests, then locked workspace check/clippy/test after local convergence.
- Governance validators, `git diff --check`, changed-file inventory, and exact-head CI/review for the stable candidate.

### Documentation To Update

- RUNTIME-007 owner and Issue #499 evidence.
- `docs/BOARD.md`, `docs/backlog/PRODUCT-BACKLOG.md`, `docs/iterations/README.md`, and `.agent-governance/manifest.yaml` as derived synchronization after owner activation/closeout.

### Risks And Rollback

- Risk: terminalizing a resumable state or releasing the active lock twice could lose work or corrupt lifecycle state.
- Rollback: revert the implementation commit while retaining the published baseline and characterization evidence; do not clear queues or rewrite history.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-09-08 | Claim preparation | Prepared atomic claim+activation for #499 after inventorying non-terminal iterations. Both claim and Active state remain ineffective until the governance record merges to `main`. |

## Verification Evidence

- Pending effective claim, implementation, focused runtime tests, and exact-head review.

## Completion Evidence

- Completion Commit: pending

## Variance And Residuals

- #408 remains a related symptom and may share lifecycle primitives, but its acceptance is not silently subsumed.
- CAP-001 / #466 remains an unclaimed architecture parent; follow-up governance is separate and serial after #499.

## Retrospective

- Outcome: pending
- Documentation: pending
- Lessons: pending
