# RUNTIME-007: Pre-Provider Rejection Terminalization

> Document status: Refinement / Unclaimed

| Field | Value |
|---|---|
| Story ID | RUNTIME-007 |
| Type | Runtime / Session Story |
| Priority | P1 |
| Status | Refinement / Unclaimed |
| Source | [GitHub Issue #499](https://github.com/wjhuang88/talos/issues/499) |
| Selected Iteration | None |
| Depends On | Runtime submission lifecycle characterization; #408 relationship review |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Pre-Provider non-resumable rejection terminalization and model-switch guard correctness |
| Claimed At | Not applicable |
| Source Issue | #499 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-09-07 |
| Handoff / Release Condition | Characterize runtime owner and preserve queued-work identity before selecting an iteration |

## Scope

Terminalize deterministic failures such as `ContextBudgetExceeded` before Provider start, release
the active submission lock exactly once, preserve unrelated queued work and existing Provider-started
guards, and prove model switching through runtime tests. No TUI-only exception or queue clearing shortcut.

## Exclusions

No dependency upgrade, I250 authority, automatic context compaction, provider selection, or unrelated
permission/sandbox behavior. Any dependency change requires separate dependency governance.

## Acceptance

- Pre-Provider `ContextBudgetExceeded` reaches terminal state without synthetic cancel and makes model switching available when no legitimate work remains.
- Provider receives zero calls; unrelated queued work retains identity/order.
- Repeated lifecycle events are idempotent; replay/resume creates no ghost turn.
- Provider-started behavior remains unchanged and regression tests cover both paths.

## Verification Evidence

- Pending characterization, iteration selection and effective claim.

## Completion Evidence

- Completion Commit: pending
