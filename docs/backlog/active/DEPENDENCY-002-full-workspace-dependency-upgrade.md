# DEPENDENCY-002: Full Workspace Dependency Upgrade

> Document status: Review

| Field | Value |
|---|---|
| Story ID | DEPENDENCY-002 |
| Type | Dependency Upgrade Story |
| Priority | P0 |
| Status | Review / Claimed |
| Source | [GitHub Issue #474](https://github.com/wjhuang88/talos/issues/474) |
| Selected Iteration | I250 |
| Depends On | I248 complete; fresh dependency audit and candidate handoff |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 mainline governance session |
| Work Slice | Full workspace dependency-version upgrade; every I248 identity row must be upgraded or explicitly dispositioned |
| Claimed At | 2026-09-07 |
| Source Issue | #474 |
| Governance Claim PR | #500 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | #500 merged as `ea9a4c37c129c2eb77825191b9e86baced4c91d4`; claim and activation effective |
| Implementation PR | #501 |
| Last Updated | 2026-09-07 |
| Handoff / Release Condition | #501 awaits final independent review and merge-time CAS, then accepted-baseline advancement and owner-first closeout |

## Scope And Definition Of Done

Use the I248 audit as build truth, then process all 56 package names / 67 identity rows. Latest stable, including majors, is the default destination. Compatible batches may be grouped; major, pre-1.0, native, runtime, persistence, provider, permission, sandbox and public-API impacts require isolated validation. Every non-upgraded item must be explicitly `current`, `blocked`, or `exception` with owner, reason, revisit trigger, evidence gap and rollback.

The work is complete only when manifests and lockfile are validated, default features and `cargo run`/`cargo build` behavior are unchanged, all affected domains pass their tests and exact-head review gates, and the accepted baseline is advanced in a separate owner-first closeout using an already-merged implementation commit.

## Required Reads

- `docs/sop/DEPENDENCY-UPGRADE.md`
- `docs/iterations/I250-full-workspace-dependency-upgrade.md`
- `docs/reference/dependency-audit-2026-09-06.json`
- `docs/reference/dependency-upgrade-candidates-2026-09-06.json`
- root and member `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`
- `AGENTS.md`, `REQUIREMENT-INTAKE.md`, `START-ITERATION.md`, `ITERATION-WORKFLOW.md`, `AGENT-COLLABORATION.md`, `TESTING.md`, `CHANGE-CONTROL.md`, `DOC-CHECK.md`

## Residual Destination

Candidate-specific blockers belong in I250's final disposition table or a separately selected child iteration. Do not hide them in the baseline or close #474 while any row lacks a disposition.
