# Iteration I231: Initial Connection And Queue Status Sequencing

> Document status: Complete / Closed
> Published plan date: 2026-08-28
> Planned objective: close TUI-060/#332 by making initial connection and first-submission queue status truthful and observable.
> Baseline rule: preserve this target; changed objectives require a new iteration ID.
> MVP deliverable: runnable conversation/TUI paths proving one observable initial Connecting phase and no false queue hint for an idle first submission.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 Sol mainline TUI-060 session |
| Work Slice | Implement only TUI-060: truthful queue hint semantics for idle versus active-turn input, and observable initial Connecting before real retry facts, preserving retry policy, submission identity, steering FIFO, cancellation and persistence. Exclude provider policy/protocol, permission, persistence schema, release, Dashboard, Desktop and `/auto`. |
| Claimed At | 2026-08-28 |
| Source Issue | #332 |
| Governance Claim PR | #420 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer-directed long-task objective; finalized claim requires exact-head CI, validators and review before merge. |
| Implementation PR | #421 |
| Last Updated | 2026-08-28 |
| Handoff / Release Condition | Claim PR #420 merged as `27f64b70`; implementation PR #421 merged as `e4cbb714` after exact-head CI and CAS. |

## Published Baseline

### Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| TUI-060 | I211 corrective owner / Issue #332 | Ready / Unclaimed | PROVIDER-006/I210 merged progress contract; steering submission boundary | Observable initial connection phase and queue hints only for truly queued active-turn input. |

### Scope

- Keep initial Connecting presentation observable without fabricating provider facts or changing retry policy.
- Emit queue hint only when input is accepted behind an already active turn.
- Preserve submission identity, steering FIFO, cancellation, persistence and terminal cleanup.
- Cover idle first submit, active steering, fast retry, success, failure and cancellation.

### Non-Goals

- No provider retry policy/protocol, permission policy, persistence migration, dependency, release, Dashboard, Desktop or `/auto` change.

### Acceptance

- Idle first submission dispatches without `Message queued and will send after current turn.`.
- Active-turn steering retains the queue hint and FIFO behavior.
- Fast retry presents initial Connecting before Reconnecting attempt facts without delaying cancellation.
- Success, final failure and cancellation clear status correctly.

### Planned Validation

- Focused conversation/bridge/TUI lifecycle tests and rebuilt binary or PTY evidence.
- Locked workspace tests, strict Clippy, release preflight, governance validators and diff check.
- Exact-head CI, independent review and merge-time CAS.

### Documentation To Update

- TUI-060, PROVIDER-006/I210 corrective disposition, README connection/queue behavior, Board, Backlog, iterations README, manifest and Issue #332.

### Risks And Rollback

- Risk: presentation timing blocks cancellation or hides real retry progress; queue fix changes custody.
- Rollback: retain typed retry facts and steering custody while reverting only status/hint projection.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-28 | Atomic claim+activation | PR #420 exact head `381fac7c` passed exact-head CI and single-maintainer claim CAS `5447355718`, then merged as `27f64b70`. I231/TUI-060 claim and Active state became effective. |

## Verification Evidence

- Candidate `6f79f559` suppresses only idle-first-submit queue hints at the bridge and defers a fast Reconnecting status for one 150ms activity interval without delaying cancellation or terminal state. PR #421 exact head `38c4f030` merged as `e4cbb714` after CI `33136176771`; independent reviewer was unavailable and bounded single-maintainer CAS is recorded in `5447796064`.
- Full standard preflight passed outside nested sandbox; TUI 563 and CLI 357 core tests plus strict Clippy passed.

## Completion Evidence

- Completion Commit: `e4cbb714` (PR #421 implementation merge).
