# Iteration I258: Bundle Manifest Migration Contract

> Document status: Complete / Closed
> Completion Commit: `9994397b580d9edd38c0b81f6939a845ef772e4a`
> Planned objective: Accept ADR-073 as the migration contract for BUNDLE-001 without changing runtime or persisted behavior.
> MVP deliverable: a reviewable, testable compatibility matrix and rollback contract for the later Bundle implementation.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex mainline execution Agent |
| Work Slice | Decision and migration-contract documentation only; no schema or runtime implementation. |
| Claimed At | 2026-09-11 |
| Source Issue | #514 |
| Governance Claim PR | #536 |
| Authorization Mode | Independent review |
| Authorization Evidence | Claim #536 effective at 9994397b580d9edd38c0b81f6939a845ef772e4a; independent approval 5630320484 and CI 34568546795 bound to a3351dba4cb584a9d3eeea9af213084c8b88ff3a. Shared account provides Agent-role separation only. No BUNDLE implementation authorization. |
| Implementation PR | #536 (decision documents only) |
| Last Updated | 2026-09-11 |
| Handoff / Release Condition | Decision merged and accepted. BUNDLE-001 still requires its own implementation claim and security/API review. |

## Published Baseline

### Scope

- Define dual-read, controlled-write, unknown-field, identity, and rollback rules for Bundle manifests.

### Non-Goals

- No Rust/Cargo changes, persisted schema changes, installation, activation, network, release, or publication.

### Acceptance

- ADR-073 contains a deterministic compatibility matrix and explicit rollback boundary.
- BUNDLE-001 and DIST-001-A dependency boundaries are recorded.

## Selection Inventory And Correction Checkpoint (2026-09-11)

Verified base: `main@b8faf39aa6fcd7cfec9c701a9436e0ef1c2cf7a0`. Current owner headers
were enumerated across iteration files; historical logs, templates and empty-status legacy plans
do not establish active authority. The Published Baseline above is unchanged.

| Owner / work | Current state | Disposition |
|---|---|---|
| Existing Active / Review / Blocked iterations | None | No overlapping active implementation; I162 is Complete with a historical Review outcome. |
| I164 | Paused / superseded by I165 | Preserve; do not resume. |
| I249 | Planned / Unclaimed | Retain unselected dependency pilot; no dependency upgrade here. |
| I258 | Complete / Closed | #536 merged and ADR-073 accepted; no implementation authority transferred. |
| BUNDLE-001 / #514 | Refinement / Unclaimed | I258 supplies only its migration decision prerequisite; no implementation activation. |
| DIST-001-A / #509 | Refinement / Unclaimed | Dependency-blocked on BUNDLE-001 implementation; CAP-001-C is complete. |
| CAP-001-A/B/C/G, TEXT-001, LANG-001 | Terminal (owners record Complete) | Preserve I252-I257 evidence; no reopening. |
| LANG-002 / #516 | Refinement / Unclaimed | Wait for DIST-001-A; LANG-001 and CAP-001-C are complete. |
| LANG-003 / #517 | Refinement / Unclaimed | Wait for LANG-002, BUNDLE-001 and DIST-001-A. |
| DIST-001-B / #515 | Refinement / Unclaimed | Wait for DIST-001-A and BUNDLE-001; completed CAP dependencies transfer no authority. |
| BROWSER-001 / #508 | Refinement / Unclaimed | Separate unselected Provider slice. |
| INTEGRATION-001 / #520 | Intake / Unclaimed | Separate clarification; excluded. |

All other terminal iterations remain terminal and all unselected backlog items retain owner-defined
gates. One worktree, no stash, and only #536 open were observed; retained branches are historical
references, not active claim authority.

Review `5630168797` requested the proposed claim and inventory above; it found ADR-073's
contract acceptable but did not approve the candidate. Corrections stay in #536. Obtain fresh
exact-head review/CI after the batch. ADR-073 remains Proposed; after independent approval,
record explicit maintainer acceptance before closing I258. This status correction is not completion
evidence. BUNDLE-001 must separately decide legacy-reader retirement criteria before any future
removal; this ADR authorizes no retirement.

## Verification Evidence

- Source review: ADR-072 and I253 migration matrix cross-checked against ADR-073.
- Runtime behavior: intentionally unchanged; implementation remains unauthorized.

## Completion Evidence

Independent architecture review and maintainer acceptance completed; the decision merge is recorded above as completion evidence.

## Post-Merge Checkpoint (2026-09-11)

PR #536 merged as `9994397b580d9edd38c0b81f6939a845ef772e4a` after approval
[5630320484](https://github.com/wjhuang88/talos/pull/536#issuecomment-5630320484)
and CI `34568546795` (four successful jobs, Windows Rust skipped) on exact head
`a3351dba4cb584a9d3eeea9af213084c8b88ff3a`, base
`b8faf39aa6fcd7cfec9c701a9436e0ef1c2cf7a0`. Merge-time CAS confirmed unchanged
head/base, CLEAN merge state and clean worktree. The local cleanup failure occurred after the
remote merge and did not invalidate it; local main was subsequently aligned to the merge commit.

This checkpoint supersedes the pending-review and ineffective-claim wording in historical records.
ADR-073 is Accepted by maintainer authorization; I258 is Complete / Closed. The existing decision merge above is the completion evidence;
the future status commit must not cite itself. BUNDLE-001 and DIST-001-A remain unclaimed.
