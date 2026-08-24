# Iteration I225: Windows Job Object Security And OS-ABI Decision

> Document status: Planned / Unclaimed
> Published plan date: 2026-08-24
> Planned objective: decide the assigned-before-exec Windows Job Object ownership, bounded OS-ABI,
> compatibility, migration, rollback and validation contract required before TOOL-024-D1-B.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: an independently reviewed current-path matrix and Accepted ADR-068 make a
> separate Windows implementation iteration runnable without changing executable behavior.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #59 |
| Governance Claim PR | Pending |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-24 |
| Handoff / Release Condition | Claim and activation are ineffective until this governance record reaches `main`; no ADR-068 decision work or production implementation begins before then. |

## Published Baseline

Planning target: `main@3cb4eff8a7e70e9b8f2c3ed1b667b2ce58f41fe4`.

### Current Nonterminal Inventory And Disposition

| Iteration(s) | State | I225 disposition |
|---|---|---|
| I164 | Paused / superseded | Do not restore. |
| I197 | Review / Claimed; corrective validation incomplete | Preserve TUI-059/#330; no overlap. |
| I198 | Review / Claimed; corrective validation incomplete | Preserve SKILL-005/#333; no overlap. |
| I201 | Review / Claimed; corrective validation failed | Preserve TUI-058/#329; no overlap. |
| I206-I208 | Planned / Unclaimed | Preserve ordered steering work; do not activate. |
| I210 | Review / Claimed; corrective validation incomplete | Preserve TUI-060/#332; no overlap. |
| I223 | Planned / Unclaimed | Keep evidence-only cleanup inactive until B/C/D heads and final integrated main exist. |
| I225 | Planned / Unclaimed | This proposed decision-only slice; ineffective until its atomic claim reaches `main`. |

I213/Dashboard and I224/TOOL-024-C are Complete/Closed. At selection, the only open PRs targeting
`main` are archival Drafts #120/#121; PR #387 has merged. No open D1/D2 implementation PR, D1
owner, ADR-068 or competing I225 proposal exists. Historical remote branches do not establish
authority and are not reused.

### Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| TOOL-024-D1-A | TOOL-024 / Issue #59 | Planned / Unclaimed | I224 Complete; ADR-060 and ADR-057 Accepted | Accepted Windows Job Object/OS-ABI decision and runnable D1-B boundary; zero behavior change |

### Scope

- Characterize current Windows creation, ownership, pipe, wait, cancellation, supervisor and
  shutdown paths after I224.
- Decide assigned-before-exec Job Object creation/configuration/assignment/resume and handle
  lifetime semantics, including nested Job Object and partial-failure behavior.
- Select the minimal dependency and bounded `unsafe`/OS-ABI surface with checked rights,
  conversions and errors.
- Freeze compatibility, migration, rollback, reversal triggers, exact D1-B authority inventory and
  Windows child/grandchild validation matrix in ADR-068 and a current-path matrix.
- Preserve D1-B implementation, D2 projection and I223 evidence cleanup as separate owners.

### Non-Goals

- No Rust/Cargo/dependency/lockfile/build or runtime behavior changes.
- No Windows background enablement, Job Object code, `unsafe`, CLI/TUI projection or persistence.
- No Unix, foreground, permission, public API, Dashboard/I213, `/auto`, release, Desktop or
  publication change.

### Acceptance

- The current-path matrix covers every D1-B production seam and identifies the exact owner of each
  proposed change.
- ADR-068 selects a race-free assigned-before-exec sequence and complete fail-closed state machine.
- Handle RAII, kill-on-close, nested Job Object behavior, deadline/cancel/shutdown races,
  dependency/`unsafe` bounds, compatibility, migration and rollback are explicit.
- D1-B has a runnable Windows test plan proving child/grandchild cleanup and no pre-assignment
  execution; D2 and I223 remain separately gated.
- Exact-head CI, both validators, YAML/diff checks and independent Windows/process/unsafe/API review
  pass; the decision PR changes no production code.

### Planned Validation

- Source/manifest/import inventory against current main and Windows API contract cross-check.
- Independent exact-head Windows process-security, OS-ABI, `unsafe` and public-API review.
- `scripts/validate_project_governance.sh .`
- `bash scripts/validate_collaboration_claims.sh .`
- Manifest YAML parse, `git diff --check`, EOF and changed-file inventory.
- CI docs/governance route; no Rust result is claimed as decision evidence.

### Documentation To Update

- TOOL-024-D1-A and I225 owners.
- Proposed ADR-068 and a Windows Job Object current-path/migration matrix during decision execution.
- TOOL-024 parent, Issue #59 long task, decision/iteration indexes and derived Board/backlog/manifest.

### Risks And Rollback

- A spawn-then-assign design permits code execution before ownership and is unacceptable.
- A suspended child without complete failure cleanup can leak a process or handle.
- Tokio/std process abstractions may not expose the primary-thread handle required for safe resume;
  the decision must prove its selected boundary rather than assume it.
- Host Job Object nesting can reject assignment; unsupported configurations remain fail-closed.
- Rollback: reject ADR-068 and leave Windows background mode fail-closed. Existing Unix and
  foreground behavior remain unchanged.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-24 | Selection prepared | I224 closed through PR #387 merge `3cb4eff8`; fresh inventory found no D1 owner or competing proposal. I225 remains inactive pending an effective claim. |

## Verification Evidence

- Pending atomic claim merge and later decision evidence.

## Completion Evidence

- Completion Commit: Pending.
- A claim/status commit cannot certify the decision; completion requires pre-existing reviewed
  ADR-068 and current-path matrix commits.

## Variance And Residuals

- TOOL-024-D1-B, D2 and I223 remain separately governed and unauthorized.

## Retrospective

- Pending execution.
