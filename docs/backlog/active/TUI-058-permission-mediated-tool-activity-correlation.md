# TUI-058: Permission-Mediated Tool Activity Correlation

| Field | Value |
|---|---|
| Story ID | TUI-058 |
| Type | Bug / TUI / Permission Presentation Story |
| Priority | P0 corrective residual from I211 |
| Status | Review / Claimed |
| Source | [GitHub Issue #329](https://github.com/wjhuang88/talos/issues/329) |
| Selected Iteration | I229 (claim PR #413 merged as `0e0c79ba`) |
| Depends On | TUI-043/I201 merged behavior; permission approval event ordering; ADR-054 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 Sol mainline TUI-058 session |
| Work Slice | Implement only TUI-058 permission-mediated tool activity correlation: preserve pending compatibility marker until correlated real ToolCall, suppress it only when known, and render each approval/result outcome with one named request identity. Add approve/deny/cancel/failure/retry/timeout and direct-event negative coverage. Exclude permission policy, provider, persistence schema, renderer rewrite, release, publication, Dashboard, Desktop and `/auto`. |
| Claimed At | 2026-08-27 |
| Source Issue | #329 |
| Governance Claim PR | #413 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer-directed long-task objective; finalized claim requires exact-head CI, governance validators and independent review before merge. |
| Implementation PR | Not started |
| Last Updated | 2026-08-27 |
| Handoff / Release Condition | Claim PR #413 merged as `0e0c79ba`; implementation is limited to this Work Slice and requires exact-head CI/review. Protected permission-surface changes require independent security review. |

## Identity / Goal / Value

When a structured tool call crosses an approval boundary, retain one coherent named activity
sequence without leaking the standalone compatibility marker or adding an unnamed approved/denied
outcome row.

## Scope

- Preserve a pending exact `Calling tools...` or `Calling tools…` marker across approval-wait state
  until the correlated structured tool call is established or the sequence terminates.
- Suppress the marker only for a real correlated `ToolCall`.
- Associate approved, denied, cancelled, failed, retry and timeout presentation with the tool name
  and request identity, without a duplicate unnamed outcome row.
- Preserve one execution and one durable named result in their existing order.
- Add event-level and real-terminal coverage for approve, deny, cancel, failure, retry/timeout and
  direct `ToolResult`/`ToolApprovalRequest` negative paths.

## Exclusions

- No permission-policy, default decision, sandbox, provider protocol or execution-semantics change.
- No broad renderer rewrite, persistence migration, dependency, release or publication work.
- No global phrase filter; legitimate marker-containing assistant text remains visible.

## Evidence And Dependency Facts

Issue #302 natural-person checkpoint `5341637918` on integrated `main@ec794515` observed an approved
write and approved missing-file read retaining `Calling tools…`, then an unnamed approved row, then
the named structured tool row. I201 final head `d8d414ce` merged as `7f5a6df2`; I197 final head
`9fce4f13` merged as `d98f37e7`. Both source owners remain Review.

## Acceptance For Behavior

- Given an exact standalone compatibility marker followed by a permission-mediated tool call,
  when approval is allowed or denied, then the marker is not leaked after the real call is known
  and every visible outcome is associated with that tool exactly once.
- Given legitimate longer assistant text containing either marker spelling, when no correlated
  tool call follows, then the text remains unchanged.
- Given direct result or approval events without a preceding real tool call, when the sequence
  finishes, then the pending marker is preserved and no false correlation is created.
- Cancellation, failure, retry and timeout preserve request identity, ordering, permission
  semantics, execution count and durable history.

## Required Reads

- `docs/backlog/active/TUI-043-tool-placeholder-suppression.md`
- `docs/backlog/active/TUI-045-permission-prompt-layout-anchor.md`
- `docs/iterations/I201-tui043-tool-placeholder-suppression.md`
- `docs/decisions/054-alternate-screen-app-owned-transcript-rendering.md`
- Issue #302 comment `5341637918`

## State / Status Owners

- Story scope and acceptance: this file.
- Remote corrective report: Issue #329.
- Failed source evidence: VALIDATION-002/I211 and Issue #302.
- Derived views: Product Backlog, Board and Issue status matrix.

## User-Facing Documentation

Update TUI behavior documentation in the future implementation iteration. This intake changes no
runtime behavior.

## Residual Destination

Permission policy, grant semantics or security decisions remain in PERM owners; general live
activity headers remain in TUI-057.
