# TEST-002: Tombstone-Pruning Fixture Performance

| Field | Value |
|---|---|
| Story ID | TEST-002 |
| Type | Test Infrastructure Story |
| Priority | P1 |
| Status | Active / Claimed proposal |
| Source | [GitHub Issue #396](https://github.com/wjhuang88/talos/issues/396) |
| Selected Iteration | I227 |
| Depends On | Existing pending-submission idempotency and tombstone-pruning contract |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 Sol mainline performance slice |
| Work Slice | TEST-002 test-only pruning-threshold injection or equivalently bounded storage fixture, focused timing evidence, and owner synchronization only. |
| Claimed At | 2026-08-25 |
| Source Issue | #396 |
| Governance Claim PR | #398 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer requested investigation and remediation of the repeated Windows 60-second delay. No separate natural-person reviewer is currently available in the shared-account operating setup; the limitation is disclosed explicitly, and Agent-role review plus exact-head CI and both governance validators are required before merge. |
| Implementation PR | Not started |
| Last Updated | 2026-08-25 |
| Handoff / Release Condition | Proposed claim and activation are ineffective until PR #398 merges; implementation starts from that merge or later and remains disjoint from I226 / PR #394. |

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
