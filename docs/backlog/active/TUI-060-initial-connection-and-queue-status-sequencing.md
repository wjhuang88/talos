# TUI-060: Initial Connection And Queue Status Sequencing

| Field | Value |
|---|---|
| Story ID | TUI-060 |
| Type | Bug / Conversation Status / TUI |
| Priority | P0 corrective residual from I211 |
| Status | Complete / Closed |
| Source | [GitHub Issue #332](https://github.com/wjhuang88/talos/issues/332) |
| Selected Iteration | I231 (claim PR #420 merged as `27f64b70`) |
| Depends On | PROVIDER-006/I210 merged progress contract; steering submission boundary |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 Sol mainline TUI-060 session |
| Work Slice | Implement only TUI-060 initial connection and queue status sequencing; preserve provider retry policy, submission identity, steering FIFO, cancellation and persistence. Exclude permission, release, Dashboard, Desktop and `/auto`. |
| Claimed At | 2026-08-28 |
| Source Issue | #332 |
| Governance Claim PR | #420 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer-directed long-task objective; finalized claim requires exact-head CI, validators and review before merge. |
| Implementation PR | #421 |
| Last Updated | 2026-08-28 |
| Handoff / Release Condition | Claim PR #420 merged as `27f64b70`; implementation PR #421 merged as `e4cbb714` after CI `33136176771` and bounded single-maintainer CAS `5447796064`. |

## Identity / Goal / Value

Make connection and submission status truthful and understandable during fast provider retries.

## Observed Failures

- `Connecting...` can be replaced by `Reconnecting... (attempt 1/2)` before the initial state is
  observably rendered.
- An idle first user submission is routed through the steering queue and emits
  `Message queued and will send after current turn.` even though no prior turn is active.

## Scope

- Give the initial connection state an observable, cancellation-safe presentation without
  fabricating provider progress or changing retry policy.
- Emit a queue hint only when input is accepted behind an already active turn; an idle first
  submission must dispatch without a queue hint.
- Preserve submission identity, steering order, cancellation, persistence and terminal cleanup.
- Add focused tests for idle first submission, active-turn steering, fast retry, success, failure
  and cancellation.

## Exclusions

- No provider retry-policy or protocol change.
- No permission-policy, persistence migration, dependency, release or publication work.

## Evidence And Required Reads

- I210 implementation head `c984ec48`, merged as `9d5c8a71`.
- I211 integrated validation evidence in Issue #302 and PR #331.
- `docs/backlog/active/PROVIDER-006-bounded-retry-progress-contract.md`
- `docs/iterations/I210-provider-retry-progress-contract.md`

## Residual Destination

This intake changes no runtime behavior. Select a separate iteration and effective claim before
implementation; I211 remains evidence-only.

## Completion Checkpoint (2026-08-28)

TUI-060 / I231 is Complete / Closed. The existing implementation merge `e4cbb714` is the
Completion Commit. Idle first submissions no longer show a false queue hint; active-turn steering
retains its hint and FIFO custody. Fast retries keep Connecting visible for one activity interval,
while cancellation and terminal states preempt deferred reconnect immediately. Provider retry
policy, protocol, submission identity and persistence are unchanged. I210/#278 retains its
separate deferred natural-person acceptance.

Completion Commit: `e4cbb714`.
