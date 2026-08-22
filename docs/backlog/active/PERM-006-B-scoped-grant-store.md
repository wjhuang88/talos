# PERM-006-B: Centralized Grant Compiler And Scoped Grant Store

**Status**: Complete / Closed

| Field | Value |
|---|---|
| Story ID | PERM-006-B |
| Type | Permission / Technical Story |
| Priority | P0 |
| Status | Complete |
| Source | [GitHub Issue #54](https://github.com/wjhuang88/talos/issues/54) |
| Selected Iteration | I219 |
| Depends On | PERM-006-A Complete; preserves PERM-004/PERM-005 and SEC-001 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 mainline implementation session 2026-08-22 |
| Work Slice | Implement only PERM-006-B / I219: first-class grant/compiler/store types in `talos-permission`; explicit in-memory Session ownership; proposal/approval revision and restriction CAS; consuming Once and pre-admission fencing; exact-only path scope; full provenance and multi-facet matching; all-policy Deny dominance; existing Bash classifier descriptor reuse; shared CLI/TUI/Runtime compilation, preview and official wrapper integration; session transition clearing; and the ADR-066 v0.9 public API/schema migration with automated and real preflight evidence. Preserve the existing agent-owned pipeline boundary for PERM-006-C. No persistent/task/cross-process grants, typed-effect migration, model-assisted auto or `/auto`, sandbox/fallback policy change, TOOL-024/background jobs, release, version, tag, publication, Desktop or Dashboard work. |
| Claimed At | 2026-08-22 |
| Source Issue | #54 |
| Governance Claim PR | #359 |
| Authorization Mode | Independent review |
| Authorization Evidence | PR #359 candidate `96816eb9` passed exact-head CI `32558607899` and independent Agent-role permission/security/API review `5378949775`, then merged as `781bb112`. Shared GitHub identity proves Agent-role separation only, not natural-person identity separation. |
| Implementation PR | #368 |
| Last Updated | 2026-08-22 |
| Handoff / Release Condition | Closed through Completion Commits `56436027` and `d0c96048`, exact-head CI `32579790496`, independent Agent-role delta review `5381051760`, CAS and PR #368 merge `de79ad46`. C-E remain separately governed. |

## Identity / Goal / Value

Separate configured policy from user-approved runtime grants and make one compiler define ApproveOnce and session-reusable scope across every surface.

## Scope

- Explicit grant identity, scope, provenance, compiler, in-memory session store, matching, and safe descriptions.
- Shared path, command, network, remote, and multi-facet grant compilation.
- Cross-surface structural-equivalence tests.

## Exclusions

- No persistent grants, task/scheduler inheritance, trusted-workspace broadening, or agent pipeline migration.

## Dependencies

PERM-006-A Complete; preserves PERM-004/PERM-005 and SEC-001

## Decision Links And Constraints

- [ADR-066](../../decisions/066-first-class-scoped-permission-grants.md) is the required
  first-class grant, session-lifetime, precedence and public API decision. Its Accepted status
  clears the decision gate only and authorizes no implementation.
- All effective policy Deny, including Configured and SDK/Runtime Explicit rules, plus hard
  boundaries override every grant.
- External paths never gain directory-wide reusable grants.
- Bash templates reuse the audited classifier; no second parser.

## Uncertainty And Validation Path

PERM-006-A now supplies authoritative requests and provenance. Refine the compiler/store lifetime,
scope-equivalence and approval-preview contract in the selected iteration before implementation.

## State / Status Owners

- Story status and acceptance: this file.
- Remote request state and discussion: GitHub Issue #54.
- Current operating view: `docs/BOARD.md`.
- Compact selection view: `docs/backlog/PRODUCT-BACKLOG.md`.

## User-Facing Documentation

Update user or SDK documentation only when observable behavior or a public integration contract changes.
Do not present this Story as shipped while it remains an unmerged claim proposal or Active.

## Required Reads

- docs/backlog/active/PERM-006-A-structured-permission-decisions.md
- docs/backlog/active/PERM-004-workspace-trust-sandbox.md
- crates/talos-permission/
- crates/talos-cli/src/approval.rs
- crates/talos-runtime/src/

## Acceptance For Behavior / Technical Work

- Equivalent requests compile equivalent grants across CLI/TUI/runtime.
- Policy and grants have separate storage and provenance.
- Session grants expire with the session and never override Deny.
- Approval preview exactly describes the installed scope.

## Residual Destination

Persistent or task-scoped grants require a separate owner and ADR.

## 2026-08-22 Dependency Clearance Checkpoint

PERM-006-A/I189 is Complete/Closed at Completion Commit `6b577d6a`; implementation PR #356
merged as `54241bdd` after exact-head CI and independent permission/security/code review. This
clears PERM-006-B's dependency only. PERM-006-B is Ready/Unclaimed with Selected Iteration None;
no implementation branch or code change is authorized before a separate runnable iteration and
effective protected-scope Collaboration Claim reach `main`.

## 2026-08-22 ADR-066 Decision Checkpoint

Read-only assessment found that PERM-006-B changes security behavior and published SDK contracts:
it separates grants from ADR-065's compatibility rule vector, converges CLI parent-scope and
Runtime exact-scope behavior, and replaces legacy runtime-rule/lifetime APIs. ADR-066 records the
required decision, compatibility and rollback boundary. It remains Proposed until exact-head
independent permission/security/API review, CI, CAS and target-branch merge; this checkpoint does
not select an iteration, establish a claim or authorize implementation.

## 2026-08-22 ADR-066 Acceptance Checkpoint

ADR-066 decision content commit `17088d88` was independently approved at exact Proposed head
`33199bd8` in PR #358 comment `5376959300`, with CI run `32541156457`; repository-owner acceptance
is recorded in comment `5378407820`. The ADR is Accepted, clearing only this Story's decision gate.
PERM-006-B remains Ready/Unclaimed with Selected Iteration None until a separate runnable/testable
iteration and effective protected-scope Collaboration Claim reach `main`.

## 2026-08-22 I219 Claim And Activation Proposal

I219 selects one runnable cross-surface grant deliverable and PR #359 atomically proposes this
Story and iteration as Claimed/Active. The Work Slice is bounded by ADR-066 and explicitly excludes
PERM-006-C, persistent grants, typed effects, model-assisted `/auto`, TOOL-024, releases and product
lanes. This open proposal has no target-branch ownership or implementation effect before exact-head
CI, independent permission/security/API review, merge-time CAS and merge to `main`.

## 2026-08-22 I219 Claim Effectiveness And Implementation Start

PR #359 candidate `96816eb9` passed exact-head CI `32558607899` and independent Agent-role
permission/security/API review `5378949775`, then merged as `781bb112`. I219/PERM-006-B is now
Active/Claimed and implementation starts exactly at that merge under GOV-008 local convergence.
I213 may proceed concurrently only in its separate Dashboard owner; I219 does not modify Dashboard
owners or `crates/talos-dashboard/**`.

## 2026-08-22 Local Convergence And Review Checkpoint

The implementation, official CLI/TUI/Runtime integration, public API migration, documentation and
owner state converged locally from claim merge `781bb112`. Locked workspace tests, focused feature
tests, Clippy with warnings denied, the real read-only permission preflight, both governance
validators and diff/inventory checks passed. PERM-006-B/I219 is Review/Claimed pending one stable
candidate push, exact-head Unix/Windows CI, independent permission/security/API/code review and
merge-time CAS. `Completion Commit` remains Pending; C and later permission children, Issue #188
behavior, TOOL-024, release, publication and Dashboard remain outside this candidate.

## 2026-08-22 First Stable Stage And Correction Checkpoint

PR #368 first head `45a4990d` passed Unix release preflight and independent Agent-role technical
review. Remote reconciliation waited for non-overlapping I213 PR #367 to register Issue #366, and
Windows workspace tests found one test-only canonical-path expectation mismatch after its format,
check, Clippy and focused permission steps passed. PR #367 later merged as `94df427c`; I219 rebased
without conflict and commit `d0c96048` corrected only the cross-platform test expectation. The
corrected candidate remains Review/Claimed and requires a next stable push, exact-head CI/review,
fresh overlap inventory and CAS. Production permission semantics and all exclusions are unchanged.

## 2026-08-22 Completion Checkpoint

Corrected exact head `97028ac0` passed all five CI jobs in run `32579790496`, including Windows
workspace, and independent Agent-role delta review `5381051760` approved that exact head. Merge-
time CAS confirmed base `94df427c`, stable head, `MERGEABLE/CLEAN`, and zero Dashboard/I213 overlap;
PR #368 merged as `de79ad46`. Completion Commits are pre-existing implementation `56436027` and
Windows test correction `d0c96048`. PERM-006-C-E, `/auto`, TOOL-024/#59, release and product lanes
remain separately blocked or governed.

## Completion Evidence

- Completion Commit: `56436027b2ceae256d8c2741ad789b6492ec9918`,
  `d0c96048a49949801c13e9cfff5841abc926f76b`
- These pre-existing implementation and test-correction commits, not this status-only closeout,
  certify completion.
