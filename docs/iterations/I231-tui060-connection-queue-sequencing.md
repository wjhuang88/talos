# Iteration I231: Initial Connection And Queue Status Sequencing

> Document status: Planned
> Published plan date: 2026-08-28
> Planned objective: close TUI-060/#332 by making initial connection and first-submission queue status truthful and observable.
> Baseline rule: preserve this target; changed objectives require a new iteration ID.
> MVP deliverable: runnable conversation/TUI paths proving one observable initial Connecting phase and no false queue hint for an idle first submission.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #332 |
| Governance Claim PR | Pending |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-28 |
| Handoff / Release Condition | Establish an effective claim before implementation. |

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
| 2026-08-28 | Claim preparation | Prepared from `main@34fcfeab`; I230 is Complete/Closed, I210 remains Review and no overlapping implementation PR exists. Claim and activation remain ineffective until governance PR merge. |

## Verification Evidence

- Pending.

## Completion Evidence

- Completion Commit: Pending.
