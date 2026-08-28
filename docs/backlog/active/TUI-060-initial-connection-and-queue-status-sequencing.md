# TUI-060: Initial Connection And Queue Status Sequencing

| Field | Value |
|---|---|
| Story ID | TUI-060 |
| Type | Bug / Conversation Status / TUI |
| Priority | P0 corrective residual from I211 |
| Status | Ready / Unclaimed |
| Source | [GitHub Issue #332](https://github.com/wjhuang88/talos/issues/332) |
| Selected Iteration | I231 (claim pending; activation ineffective until merge) |
| Depends On | PROVIDER-006/I210 merged progress contract; steering submission boundary |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #332 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-20 |
| Handoff / Release Condition | Select a runnable corrective iteration and establish an effective claim before implementation. |

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
