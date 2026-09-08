# RUNTIME-007: Pre-Provider Rejection Terminalization

> Document status: Active / Claimed (proposed; ineffective until atomic claim record merges)

| Field | Value |
|---|---|
| Story ID | RUNTIME-007 |
| Type | Runtime / Session Story |
| Priority | P1 |
| Status | Refinement / Unclaimed |
| Source | [GitHub Issue #499](https://github.com/wjhuang88/talos/issues/499) |
| Selected Iteration | I251 |
| Depends On | Runtime submission lifecycle characterization; #408 relationship review |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed (proposed; ineffective until merge) |
| Responsible Actor | `@wjhuang88` (shared account; Agent-role execution) |
| Executing Agent | Codex Agent / mainline unattended session |
| Work Slice | Pre-Provider non-resumable rejection terminalization and model-switch guard correctness |
| Claimed At | 2026-09-08 |
| Source Issue | #499 |
| Governance Claim PR | Pending atomic claim PR |
| Authorization Mode | Independent review |
| Authorization Evidence | Pending exact-head governance validation and independent review |
| Implementation PR | Not started |
| Last Updated | 2026-09-08 |
| Handoff / Release Condition | Claim/activation are ineffective until the finalized record reaches `main`; implementation starts from that merge or later. |

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

- I251 selection and atomic claim preparation recorded; effective claim, implementation and exact-head evidence remain pending.

## Completion Evidence

- Completion Commit: pending
