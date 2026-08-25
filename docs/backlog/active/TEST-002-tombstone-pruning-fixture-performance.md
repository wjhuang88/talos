# TEST-002: Tombstone-Pruning Fixture Performance

**Status**: Complete / Closed

| Field | Value |
|---|---|
| Story ID | TEST-002 |
| Type | Test Infrastructure Story |
| Priority | P1 |
| Status | Complete / Closed |
| Source | [GitHub Issue #396](https://github.com/wjhuang88/talos/issues/396) |
| Selected Iteration | I227 |
| Depends On | Existing pending-submission idempotency and tombstone-pruning contract |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 Sol mainline performance slice |
| Work Slice | TEST-002 test-only pruning-threshold injection or equivalently bounded storage fixture, focused timing evidence, and owner synchronization only. |
| Claimed At | 2026-08-25 |
| Source Issue | #396 |
| Governance Claim PR | #398 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer requested investigation and remediation of the repeated Windows 60-second delay. No separate natural-person reviewer is currently available in the shared-account operating setup; the limitation is disclosed explicitly, and Agent-role review plus exact-head CI and both governance validators are required before merge. |
| Implementation PR | #399 |
| Last Updated | 2026-08-25 |
| Handoff / Release Condition | Closed after PR #399 merge `d02915e0`; production pending-submission authority remains unchanged. |

## Completion Evidence

| Field | Value |
|---|---|
| Completion Commit | `7b64a08b13cd75bd0ad43843707770919dc9ebec` |
| Implementation PR | #399, merged as `d02915e0` |
| Exact-head CI | `32839820741` on `6429febf` (5/5 success, Windows workspace included) |
| Independent Review | Agent-role APPROVE `5409698923` on exact head |
| Result | Focused test passed in `0.02s`; production `MAX_TOMBSTONES`, schema, transactions and behavior unchanged. |

Completion Commit: `7b64a08b13cd75bd0ad43843707770919dc9ebec`

## Identity / Goal / Value

Keep full Windows validation useful as a stage gate by proving permanent submission idempotency
after payload pruning without spending more than one minute in one SQLite fixture.

## Scope

- Preserve the existing delayed-retry and identity-conflict assertions after terminal payload
  pruning.
- Replace the production-bound 256-submission setup with a test-only injectable pruning limit or
  an equivalently bounded storage fixture.
- Record focused elapsed time on macOS/Linux and Windows before claiming completion.

## Exclusions

- No reduction of `MAX_PENDING_SUBMISSIONS`, `MAX_TOMBSTONES`, or other production retention limits.
- No pending-submission schema, transaction, runtime behavior, or idempotency-policy change.
- No I226 Windows Job Object, TOOL-024, permission, release, or product behavior change.

## Acceptance

- The focused test still proves that a pruned terminal payload returns its original receipt and
  cannot become fresh Provider work.
- A conflicting delayed payload for the same identity remains rejected.
- Production pending-submission and tombstone limits remain byte-for-byte unchanged.
- Full Windows workspace CI no longer emits the 60-second warning for this fixture.
- Focused locked tests and the full locked workspace validation pass.

## State / Status Owners

- Story scope and status: this file.
- Remote report and Windows timing evidence: GitHub Issue #396.
- Derived views: `docs/backlog/PRODUCT-BACKLOG.md`, `docs/BOARD.md`, and the current open-Issue
  reconciliation matrix.

## User-Facing Documentation

None. This is test-infrastructure-only and cannot claim a user behavior change.

## Required Reads

- `docs/sop/TESTING.md`
- `crates/talos-session/src/pending_submission/tests.rs`
- `crates/talos-session/src/pending_submission/storage.rs`
- `.github/workflows/ci.yml`

## Residual Destination

Any production SQLite or pending-submission performance defect discovered while bounding the
fixture requires a separate product/runtime owner and iteration.
