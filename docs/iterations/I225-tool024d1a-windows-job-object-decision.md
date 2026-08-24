# Iteration I225: Windows Job Object Security And OS-ABI Decision

> Document status: Active / Claimed
> Published plan date: 2026-08-24
> Planned objective: decide the assigned-before-exec Windows Job Object ownership, bounded OS-ABI,
> compatibility, migration, rollback and validation contract required before TOOL-024-D1-B.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: an independently reviewed current-path matrix and Accepted ADR-068 make a
> separate Windows implementation iteration runnable without changing executable behavior.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 Sol mainline Issue #59 decision session 2026-08-24 |
| Work Slice | Decide only the Windows Job Object prerequisite: current-path inventory; assigned-before-exec process/Job Object/primary-thread sequence; allowlisted child-handle inheritance; handle RAII, kill-on-close, nesting and fail-closed partial failures; bounded dependency/OS-ABI/`unsafe`; compatibility, migration, rollback, reversal triggers, exact D1-B authority inventory and Windows test matrix in ADR-068/current-path docs. No production or product behavior change. |
| Claimed At | 2026-08-24 |
| Source Issue | #59 |
| Governance Claim PR | #388 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer's persistent active goal authorizes Issue #59 delivery. I224 closed through PR #387 merge `3cb4eff8`; claim PR #388 exact head `e0c65c52`, CI `32729210800`, independent Windows/process/unsafe/API governance approval `5395556844`, merge-time CAS and merge `2afcdc3e` establish this claim on `main`. Shared-account review proves Agent-role separation only. |
| Implementation PR | Not started |
| Last Updated | 2026-08-24 |
| Handoff / Release Condition | Decision work starts from `main@2afcdc3e` or later. D1-B implementation remains separately claimed. |

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
| TOOL-024-D1-A | TOOL-024 / Issue #59 | Active / Claimed proposal | I224 Complete; ADR-060 and ADR-057 Accepted | Accepted Windows Job Object/OS-ABI decision and runnable D1-B boundary; zero behavior change |

### Scope

- Characterize current Windows creation, ownership, pipe, wait, cancellation, supervisor and
  shutdown paths after I224.
- Decide assigned-before-exec Job Object creation/configuration/assignment/resume and handle
  lifetime semantics, including nested Job Object and partial-failure behavior.
- Decide a race-free allowlisted child-handle inheritance boundary for raw/suspended creation and
  stdio pipes, using `STARTUPINFOEX`/`PROC_THREAD_ATTRIBUTE_HANDLE_LIST` or an equivalently proven
  safe binding; unrelated inheritable handles must never enter the child.
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
  allowlisted handle inheritance, dependency/`unsafe` bounds, compatibility, migration and rollback
  are explicit.
- D1-B has a runnable Windows test plan proving child/grandchild cleanup and no pre-assignment
  execution, no unrelated inherited-handle disclosure under concurrent spawn, and complete
  attribute-list/pipe/duplicate cleanup on partial failures; D2 and I223 remain separately gated.
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
- Broad `bInheritHandles` process creation can disclose an unrelated concurrently inheritable
  handle; post-create RAII cannot repair that leak.
- Tokio/std process abstractions may not expose the primary-thread handle required for safe resume;
  the decision must prove its selected boundary rather than assume it.
- Host Job Object nesting can reject assignment; unsupported configurations remain fail-closed.
- Rollback: reject ADR-068 and leave Windows background mode fail-closed. Existing Unix and
  foreground behavior remain unchanged.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-24 | Selection prepared | I224 closed through PR #387 merge `3cb4eff8`; fresh inventory found no D1 owner or competing proposal. |
| 2026-08-24 | Claim activated | PR #388 exact head `e0c65c52`, CI `32729210800`, independent approval `5395556844` and merge-time CAS passed; merge `2afcdc3e` makes I225 Active/Claimed on `main`. |

## Verification Evidence

- Claim activation is evidenced by PR #388 exact head `e0c65c52`, base `3cb4eff8`, CI
  `32729210800`, independent approval `5395556844`, merge-time CAS and merge `2afcdc3e`.
- ADR-068 and the current-path/authority decision evidence remain pending; no implementation or
  Windows background enablement is authorized by I225.

## Completion Evidence

- Completion Commit: Pending.
- A claim/status commit cannot certify the decision; completion requires pre-existing reviewed
  ADR-068 and current-path matrix commits.

## Variance And Residuals

- TOOL-024-D1-B, D2 and I223 remain separately governed and unauthorized.

## Retrospective

- Pending execution.
