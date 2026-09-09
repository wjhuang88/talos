# RUNTIME-007: Pre-Provider Rejection Terminalization

> Document status: Complete / Closed

| Field | Value |
|---|---|
| Story ID | RUNTIME-007 |
| Type | Runtime / Session Story |
| Priority | P1 |
| Status | Complete / Closed |
| Source | [GitHub Issue #499](https://github.com/wjhuang88/talos/issues/499) |
| Selected Iteration | I251 |
| Depends On | Runtime submission lifecycle characterization; #408 relationship review |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex Agent / mainline unattended session |
| Work Slice | Pre-Provider non-resumable rejection terminalization and model-switch guard correctness |
| Claimed At | 2026-09-08 |
| Source Issue | #499 |
| Governance Claim PR | #504 |
| Authorization Mode | Independent review |
| Authorization Evidence | Corrected implementation head `7d770822a0f5dd72d97897777c2786adcc2f3362`: CI `34248498571` passed and independent APPROVE `5588712770` bound to that head; closeout head `cb407ca004dea7d8cd021281de30d6d8d35beda4`: CI `34254709125` passed and approval `5588909334` bound to that head. |
| Implementation PR | #505 (Merged) |
| Last Updated | 2026-09-09 |
| Handoff / Release Condition | Claim and implementation are closed on `main`; any further runtime lifecycle work requires a separate effective claim. |

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

- Implementation PR #505 merged as `c9225abf`; exact-head CI and independent review
  were successful for `7d770822`.

## Closeout Checkpoint (2026-09-09)

Implementation PR #505 merged as `c9225abf13c299d38e9eadc1335b59098d8d6d3f`; exact-head CI
`34248498571` and independent approval `5588712770` were bound to
`7d770822a0f5dd72d97897777c2786adcc2f3362`. Closeout PR #506 merged as
`8ccd020e948e874a51437bd735955fe8282b8885`; exact-head CI `34254709125` and approval
`5588909334` were bound to `cb407ca004dea7d8cd021281de30d6d8d35beda4`. Completion Commit
remains the pre-existing implementation commit `c9225abf13c299d38e9eadc1335b59098d8d6d3f`.

## Completion Evidence

- Completion Commit: c9225abf
