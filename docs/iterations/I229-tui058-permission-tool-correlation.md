# Iteration I229: Permission-Mediated Tool Activity Correlation

> Document status: Active / Claimed
> Published plan date: 2026-08-27
> Planned objective: close TUI-058/#329 by correlating permission-mediated tool activity without leaking compatibility markers or rendering unnamed approval outcomes.
> Baseline rule: preserve this target; changed objectives require a new iteration ID.
> MVP deliverable: a runnable CLI/TUI path and focused event fixtures where each permission-mediated tool attempt renders one correctly named activity/result sequence.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 Sol mainline TUI-058 session |
| Work Slice | Implement only TUI-058: correlate permission-mediated marker, approval outcome and named tool result by request identity, preserving permission semantics, ordering, execution count, persistence and legitimate marker-containing text; add positive and direct-event negative coverage. Exclude permission policy, provider, persistence schema, renderer rewrite, release, publication, Dashboard, Desktop and `/auto`. |
| Claimed At | 2026-08-27 |
| Source Issue | #329 |
| Governance Claim PR | #413 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer-directed long-task objective; finalized governance claim requires exact-head CI, governance validators and independent review before merge. |
| Implementation PR | Not started |
| Last Updated | 2026-08-27 |
| Handoff / Release Condition | Claim and Active state become effective only after PR #413 merges to `main`; implementation starts from that merge or later. Protected permission-surface changes require independent security review. |

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| TUI-058 | I211 corrective owner / Issue #329 | Ready / Unclaimed | I201 merged behavior; ADR-054; permission approval event ordering | One named, non-duplicated permission-mediated tool activity sequence with preserved request identity. |

### Scope

- Preserve a pending compatibility marker across approval-wait state until a correlated real `ToolCall` is established.
- Suppress the marker only after the real call is known, while preserving legitimate assistant text containing marker spellings.
- Associate approved, denied, cancelled, failed, retry and timeout outcomes with the correlated tool name and request identity exactly once.
- Add event-level, integration and real-terminal evidence for positive and direct-event negative paths.

### Non-Goals

- No permission policy, default decision, grant, sandbox, provider protocol, execution-semantics or persistence migration change.
- No broad renderer rewrite, global phrase filter, dependency, release, publication, Dashboard, Desktop or `/auto` work.

### Acceptance

- Given a standalone compatibility marker followed by a permission-mediated real tool call, when approval is allowed or denied, then the marker is not leaked and exactly one named outcome is rendered.
- Given legitimate assistant text containing either marker spelling without a correlated tool call, then the text remains unchanged.
- Given direct `ToolResult` or `ToolApprovalRequest` events without a preceding real tool call, then no false correlation or unnamed synthetic outcome is created.
- Given cancel, failure, retry or timeout, then request identity, ordering, execution count, permission semantics and durable history remain unchanged.

### Planned Validation

- Focused CLI/conversation/TUI event tests for approve, deny, cancel, failure, retry, timeout and direct-event negatives.
- `cargo test --locked --workspace`, `cargo clippy --locked --workspace --all-targets -- -D warnings`, and `./scripts/release_preflight.sh`.
- Rebuilt `talos` binary or PTY transcript proving the user-visible named sequence.
- Governance validators, exact-head CI, independent review and merge-time CAS.

### Documentation To Update

- TUI-058 story and TUI-043/I201 corrective disposition.
- `README.md`/`README.zh-CN.md` or the relevant CLI/TUI behavior reference.
- `docs/BOARD.md`, `docs/backlog/PRODUCT-BACKLOG.md`, `docs/iterations/README.md`, manifest and Issue #329 reconciliation.

### Risks And Rollback

- Risk: correlation logic hides legitimate text, duplicates execution results, or changes permission visibility.
- Rollback: retain the existing event projection and disable only the new correlation path; permission and execution authority remain unchanged.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-27 | Atomic claim+activation proposal | Prepared from `main@d440a562`; I197/I201 remain Review under this corrective owner chain, I206 is Complete, and no overlapping implementation PR exists. PR #413 proposes the single bounded claim and Active state; both remain ineffective until merge. |

## Verification Evidence

- Pending implementation and exact-head evidence.

## Completion Evidence

- Completion Commit: Pending.
- Status-only governance commits cannot self-certify implementation completion.

## Variance And Residuals

- I201 remains Review until this corrective story closes the permission-mediated failure; unrelated TUI-059, TUI-060, SKILL-005 and TUI-061 owners remain separate.

## Retrospective

- Outcome: Pending.
- Documentation: Pending implementation evidence.
- Lessons: record any new correlation or validation lesson in `EVOLUTION.md` at closeout.
