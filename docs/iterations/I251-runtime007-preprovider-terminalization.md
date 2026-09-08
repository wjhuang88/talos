# Iteration I251: Pre-Provider Rejection Terminalization

> Document status: Complete / Closed
> Published plan date: 2026-09-08
> Planned objective: Convert deterministic pre-Provider rejection into an idempotent terminal lifecycle outcome so model/provider switching is not blocked by stale active submission state.
> Baseline rule: preserve this target; a changed objective requires a new iteration ID.
> MVP deliverable: A runnable regression path where `ContextBudgetExceeded` terminalizes before Provider start, preserves unrelated queue identity/order, and permits model switching when no legitimate work remains.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex Agent / mainline unattended session |
| Work Slice | Runtime submission lifecycle only: deterministic non-resumable pre-Provider rejection terminalization, exactly-once release, model-switch guard, and focused regression evidence for Issue #499. |
| Claimed At | 2026-09-08 |
| Source Issue | #499 |
| Governance Claim PR | #504 |
| Authorization Mode | Independent review |
| Authorization Evidence | Prior head fa1c40f7: CI 34188252548 passed; review comment 5579449346 requested changes. Corrected head requires fresh CI and independent APPROVE before merge; no approval is claimed. |
| Implementation PR | #505 (Merged) |
| Last Updated | 2026-09-09 |
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

### 2026-09-08 Inventory And Review Correction Checkpoint

Target baseline: `main@891c7800943c8337df6e2a0f74f2361ef9c086d2`.
The current iteration headers contain no Active, Review or Blocked iteration on that baseline.
I162's Complete / Review-outcome header is terminal, not an open Review iteration.

| Item | Current state | Disposition |
|---|---|---|
| I251 / RUNTIME-007 / #499 / PR #505 | Active / Claimed | Implementation candidate under review; claim #504 is effective on main. |
| I249 | Planned / Unclaimed | Defer the one-package dependency pilot; I250's full upgrade is Complete and no dependency change is selected here. |
| I164 | Paused / superseded by I165 | Preserve pause; do not resume or repurpose. |
| Legacy status-less and superseded iterations | Historical records | Confer no current authority; do not activate or restore. |
| #408 | Closed; related Esc-cancellation symptom | Preserve its completion; characterize shared queue/model-switch guards without reopening or substituting for #499 acceptance. |
| CAP-001 / #466 | Refinement / Unclaimed parent Epic | Handle architecture governance serially after #499; no parent implementation authority. |
| DEPENDENCY-003 / #502 | Refinement / Unclaimed residual | Keep dependency risk work separate; no activation or Cargo changes. |
| Open PR inventory | #504 only | No overlapping implementation PR; repeat this check at merge-time CAS. |

This checkpoint supplies the previously omitted inventory and corrects current owner/derived
activation fields. The Published Baseline remains unchanged. Review comment `5579449346`
and the independent GLM-5.3 review on #504 both requested changes; neither is approval.
Execution and review share a GitHub account and claim Agent-role separation only, not
natural-person separation. A fresh APPROVE must bind the corrected candidate before merge.

- Pending effective claim, implementation, focused runtime tests, and exact-head review.

## Completion Evidence

### 2026-09-09 Closeout Checkpoint

Implementation PR #505 merged to `main` as `c9225abf`; exact head `7d770822` had
five successful CI jobs and an independent APPROVE bound to that head.

### 2026-09-08 Candidate Validation Checkpoint

Implementation candidate PR #505 currently points to `8de70a58`. Local locked validation
passed for the focused agent/session/CLI regression paths; prior exact-head CI run
`34242505749` failed during test compilation because of the import correction, so it provides no valid exact-head evidence;
its reconciliation job also predated the current owner synchronization. Issue #499 reconciliation
comment `5587908427` records the current owner and candidate state. A fresh exact-head CI and
independent review are required before merge.

- Completion Commit: c9225abf

## Variance And Residuals

- Windows lifecycle fixture diagnostics remain a CI residual: prior failures timed out before
  the marker was observed, so no production behavior change is inferred; investigate fixture
  event draining separately before treating a Windows failure as an implementation defect.
- #408 remains a related symptom and may share lifecycle primitives, but its acceptance is not silently subsumed.
- CAP-001 / #466 remains an unclaimed architecture parent; follow-up governance is separate and serial after #499.

## Retrospective

- Outcome: Complete / Closed
- Documentation: Updated
- Lessons: Exact-head evidence was re-bound after substantive corrections.
