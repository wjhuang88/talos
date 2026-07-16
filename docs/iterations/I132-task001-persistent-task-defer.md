# Iteration I132: TASK-001 Persistent Task Runtime — Defer Decision

> Document status: Complete
> Published plan date: 2026-07-16
> Planned objective: Decide task/turn/session identity, checkpoints, crash recovery, cancellation, retention, and permission re-authorization after resume.
> MVP deliverable: ADR-043 (Defer) — reviewed decision documenting that the persistent task runtime is NOT implemented; I128 + I124-I127 provide reusable primitives but do not constitute a task runtime.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `TASK-001` | none | Refinement — ADR-gated | RUNTIME-001, SESSION-004, PERM-005 | ADR-043 Defer: task runtime NOT implemented; reusable components exist but task lifecycle, phase checkpoints, incomplete-task recovery, and durable scheduling are unsatisfied. |

### Scope

- Evaluate existing I128 (durable sessions) and I124-I127 (scheduled follow-ups) against TASK-001 requirements.
- Produce ADR-043 or explicit defer/reject.

### Acceptance

- ✅ ADR-043 states data model, storage boundary, resume semantics, permission re-authorization, and cleanup policy.
- ✅ ADR-043 proves no global event bus, direct tool-execution bypass, or autonomous background process is required.
- ✅ Implementation work remains deferred; TASK-001 owner doc updated with decision link.

### Non-Goals

- No task engine, scheduler, daemon, multi-agent orchestration, or automatic execution.
- No ApprovalBridge or authorization-semantic change.

## Verification Evidence

- ADR-043 accepted as Defer with full evidence mapping.
- No code changes (decision-only package).
- Governance validation and diff checks pass.

## Variance And Residuals

- ADR-043 was revised after architecture review v1: original version incorrectly claimed
  the capability was 'substantially delivered.' Corrected to state the task runtime is NOT
  implemented.
- TASK-001 capability gap (task lifecycle, phase checkpoints, incomplete-task recovery,
  durable scheduling) remains Open via [Issue #38](https://github.com/wjhuang88/talos/issues/38)
  and the ADR-043 reversal trigger.
- SESSION-006 (P120 audit finding) remains a separate Open owner.

## Retrospective

- Outcome: met. ADR-043 (revised) provides a reviewed Defer with honest gap analysis.
- Documentation: TASK-001 owner doc, ADR index, Board, iterations README, execution package,
  PRODUCT-BACKLOG, Issue #38.
- Lessons: The initial ADR overclaimed existing infrastructure coverage. Architecture review
  correctly identified that session persistence ≠ task runtime. The revised ADR distinguishes
  reusable primitives (satisfied) from task-runtime requirements (unsatisfied).
