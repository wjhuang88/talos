# Iteration I227: Tombstone-Pruning Fixture Performance

> Document status: Active / Claimed proposal
> Published plan date: 2026-08-25
> Planned objective: preserve permanent submission idempotency coverage while removing the production-sized SQLite setup from the pruning fixture.
> Baseline rule: preserve this target; changed objectives require a new iteration ID.
> MVP deliverable: the focused pruning/idempotency test exercises the real storage path below the 60-second slow-test threshold without changing production retention or behavior.

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

## Published Baseline

### Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| TEST-002 | Issue #396 | Ready / Unclaimed | Existing pending-submission pruning and idempotency contract | The real pruning path is tested with a bounded fixture and no Windows 60-second warning. |

### Non-Terminal Iteration Inventory And Disposition

| Iteration | State At Selection | Disposition |
|---|---|---|
| I197 | Review / Claimed | Retain in Review; TUI-059 owns its failed docking acceptance. No overlap. |
| I198 | Review / Claimed | Retain in Review; SKILL-005 owns its diagnostic residual. No overlap. |
| I201 | Review / Claimed | Retain in Review; TUI-058 owns its permission-correlation residual. No overlap. |
| I206-I208 | Planned / Unclaimed | Keep unactivated. Steering scope is unrelated. |
| I210 | Review / Claimed | Retain in Review; TUI-060 owns its sequencing residual. No overlap. |
| I223 | Planned / Unclaimed | Keep dependency-gated behind Issue #59 implementation evidence. No overlap. |
| I226 / PR #394 | Implementation in progress after effective claim | Continue independently. I226 owns Windows Job Object/process-hardening files; I227 must not modify them or TOOL-024 owners. |

No other current iteration owner is Active or Review in the target-branch operating inventory.
I227 is permitted to proceed concurrently with I226 because its production authority is disjoint.
`docs/BOARD.md` and `docs/iterations/README.md` are shared derived files: the later merge must
rebase or replay its row additions with owner-first union semantics and preserve both I227 and
I226 facts. A shared derived file is not permission to overwrite the other owner's row.

### Scope

- Add a test-only way to exercise the existing tombstone-pruning path at a small threshold, or an
  equivalently bounded fixture using the same production SQL and idempotency decisions.
- Preserve replay receipt, committed-state and identity-conflict assertions after payload pruning.
- Record focused elapsed time locally and on Windows CI.

### Non-Goals

- No change to `MAX_PENDING_SUBMISSIONS`, `MAX_TOMBSTONES`, schema, transaction boundaries,
  production pruning policy or public API.
- No I226, Windows launcher, Job Object, TOOL-024, permission, CLI/TUI, release or product behavior.
- No broad pending-submission performance redesign; any production finding gets a separate owner.

### Acceptance

- Given a terminal submission whose payload is pruned, when the same payload is retried, then the
  original receipt and committed result are returned without recreating work.
- Given a conflicting payload with the same identity after pruning, when it is retried, then it is
  rejected as an identity conflict.
- Given the production build, when constants and runtime paths are compared, then production
  retention and behavior are unchanged.
- Given the full Windows workspace test, when this fixture runs, then it completes without the
  60-second slow-test warning.

### Planned Validation

- `cargo fmt --all -- --check`
- focused locked `talos-session` test with elapsed time
- `cargo test -p talos-session --locked`
- `./scripts/release_preflight.sh`
- exact-head Windows CI timing and independent code/test review

### Documentation To Update

- `docs/backlog/active/TEST-002-tombstone-pruning-fixture-performance.md`
- `docs/iterations/I227-test002-tombstone-pruning-fixture-performance.md`
- `docs/backlog/PRODUCT-BACKLOG.md`, `docs/BOARD.md`, `docs/iterations/README.md`
- GitHub Issue #396

### Risks And Rollback

- Risk: a test-only seam accidentally changes production pruning behavior or weakens the invariant.
- Rollback: remove the test-only seam and retain the original production implementation and test
  assertions; do not lower production constants.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-25 | Planning | Selected TEST-002 from `main@64d5ad4c`; claim and activation remain ineffective until the finalized governance PR merges. |
| 2026-08-25 | Atomic claim+activation proposal | PR #398 proposes one bounded TEST-002 claimant and Active state. It contains no implementation and has no effect before target-branch merge. |

## Verification Evidence

- Pending effective claim and implementation candidate.

## Completion Evidence

- Completion Commit: Pending.
- A status-only documentation commit cannot self-certify completion.

## Variance And Residuals

- Any production SQLite performance defect remains outside I227 and requires a separate owner.

## Retrospective

- Pending implementation.
