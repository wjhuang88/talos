# TUI-047: Delete Cleanup Recovery Diagnostics

| Field | Value |
|---|---|
| Story ID | TUI-047 |
| Type | TUI / Session Storage Diagnostics Story |
| Priority | P1 |
| Status | Ready |
| Source Issue | [GitHub Issue #136](https://github.com/wjhuang88/talos/issues/136) |
| Parent Evidence | TUI-044 / I169 / ADR-056 |
| Responsible Actor | Unclaimed |
| Selected Iteration | None |

## Problem

When TUI `/delete <session-uuid>` encounters a retryable Session artifact cleanup failure, Talos
correctly emits an error, preserves transcript-last ownership, reports retryability and avoids a
false success. The direct error surface does not yet print the two executable recovery commands that
the user can immediately run.

## Required Outcome

A retryable direct-delete failure includes the exact commands:

```text
/delete <session-uuid>
talos storage maintenance --reconcile
```

The diagnostic explains their different scopes:

- retry `/delete <session-uuid>` while the transcript remains discoverable;
- use `talos storage maintenance --reconcile` for transcript-less orphan sidecars or general
  reconciliation maintenance.

## Acceptance

- Forced cleanup failure emits `[Error]` and never emits `Deleted session ...`.
- The diagnostic contains the exact target UUID and executable `/delete <uuid>` command.
- The diagnostic contains the exact nested maintenance command.
- Production parser/handler tests execute the emitted retry syntax after the artifact path is
  restored.
- Production CLI parsing and execution validate the emitted maintenance syntax.
- Unsupported legacy forms such as CLI-style `--delete` and `--storage-maintenance` are absent.
- Successful deletion output remains unchanged.
- Transcript-last cleanup, partial-removal facts, index consistency, retryability and
  no-false-success behavior remain unchanged.
- Focused tests, locked workspace format/check/Clippy/tests and exact-head CI pass.

## Scope

This Story owns diagnostic/actionability wording and formatter convergence only. It does not reopen
or alter the Accepted ADR-056 custody model, TUI-044/I169 completion, deletion order, orphan-scan
safety, Session fork isolation or transactional steering semantics.

## Required Reads

- `docs/decisions/056-transactional-steering-submission-boundary.md`
- `docs/backlog/active/TUI-044-transactional-batched-steering-turn.md`
- `docs/iterations/I169-batched-steering-turn.md`
- GitHub Issue #136

## Selection Gate

Before implementation, claim a fresh current-main branch and verify that no other PR owns Issue
#136. Keep the change bounded to one shared recovery-diagnostic formatter plus production-path tests.
