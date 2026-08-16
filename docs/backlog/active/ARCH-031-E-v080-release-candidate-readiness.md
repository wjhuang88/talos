# ARCH-031-E: v0.8.0 Release-Candidate Registry Readiness

> Document status: Planned

| Field | Value |
|---|---|
| Story ID | ARCH-031-E |
| Type | Release Readiness / Version Alignment Story |
| Parent Epic | ARCH-031 |
| Priority | P1 after I162 |
| Status | Planned |
| Depends on | I162 Complete/Closed with reviewed NO-GO; current `main` at activation |
| Selected Iteration | I204 (Planned/Claimed) |
| Value | Turn the reviewed I162 NO-GO into a network-verified v0.8.0 GO packet without performing an irreversible release or publication |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | `Codex / GPT-5 mainline release-governance session` |
| Work Slice | `ARCH-031-E / I204` only: candidate v0.8.0 version alignment in an isolated readiness branch, registry visibility checks, metadata-derived 20-package closure, package and `cargo publish --dry-run` evidence, and a reviewed GO/NO-GO packet. No tag, GitHub Release, real Cargo publication, or I203 implementation. |
| Claimed At | 2026-08-16 |
| Source Issue | None |
| Governance Claim PR | #257 |
| Authorization Mode | Independent review |
| Authorization Evidence | I162 closeout is effective at `main@9fc2c7f1`; this proposed claim remains ineffective until independently reviewed and merged to the target branch. |
| Implementation PR | Not started |
| Last Updated | 2026-08-16 |
| Handoff / Release Condition | I204 must start from current `main`, preserve I162 baselines, and produce a reviewed GO before I203 can be claimed. Version edits are candidate-only until I203. |

## Problem

I162 correctly stopped at NO-GO because the workspace was `0.7.0` and the registry was unavailable.
I203 requires a reviewed GO packet, but I162 was forbidden from changing versions. This follow-up
is the smallest separate readiness slice that validates the actual `v0.8.0` candidate without
publishing or creating an immutable tag.

## Scope

- use an isolated candidate worktree and synchronize workspace/internal versions to `0.8.0` only
  for validation evidence;
- recompute the normal `talos-cli`/`talos-runtime` closure from locked metadata;
- verify crates.io visibility and compatible dependency resolution;
- run `cargo package --locked` and `cargo publish --locked --dry-run` for every authorized package;
- retain `talos-models` quarantine and explicitly account for product guards;
- produce an exact GO/NO-GO packet and migration notes for I203.

## Explicit Exclusions

- no real `cargo publish`;
- no `git tag` or GitHub Release;
- no force push or tag movement;
- no runtime/API behavior changes;
- no RUNTIME-006 single-dependency facade work;
- no activation or implementation of I203.

## Acceptance

- [ ] candidate-only `0.8.0` version alignment is reproducible and does not modify the published baseline;
- [ ] crates.io registry access and all compatible internal versions are independently verified;
- [ ] metadata-derived 20-member closure is recorded with guard and blocker state;
- [ ] package and dry-run results are recorded for every authorized closure member;
- [ ] reviewed GO/NO-GO packet names every remaining blocker and explicitly authorizes no release;
- [ ] locked validation and an external registry-mode fixture pass, or failures are truthfully retained.

## Stop Conditions

Stop and keep I203 blocked if registry access, credentials, compatible versions, package metadata,
or closure scope cannot be verified. Do not convert a failed or unavailable registry check into GO.

## Required Reads

- `docs/iterations/I162-v0.6-sdk-publication-readiness.md`
- `docs/reference/I162-PUBLICATION-READINESS-2026-08-15.md`
- `docs/iterations/I203-v080-github-and-crates-publication.md`
- `docs/backlog/active/REL-003-v080-github-and-crates-publication.md`
- `docs/sop/RELEASE.md` and `docs/sop/RELEASE-WORKFLOW.md`

## Completion Evidence

- Completion Commit: pending
- A status-only closeout cannot certify the candidate packet.

## 2026-08-16 Execution Checkpoint

Candidate worktree: `/private/tmp/talos-i204-impl` at `e6cab51a61c3a23a1e0a6792573bcef688a3d6dd`.
Workspace and internal path dependencies are aligned to `0.8.0` only in this isolated candidate;
the target branch remains unchanged and no tag, GitHub Release, or real Cargo publication was
performed.

The 16 current release-surface documents (README and bilingual site pages) are synchronized to
candidate `v0.8.0` so the repository's release preflight can validate the same candidate that Cargo
metadata reports. This is documentation/version alignment only; it does not assert that the GitHub
Release already exists.

The metadata-derived closure contains 20 members: 16 registry-enabled crates plus the four
intentional `publish = false` product guards (`talos-cli`, `talos-dashboard`, `talos-evolution`,
and `talos-tui`). `talos-models` remains outside the closure and quarantined. `cargo metadata
--locked` confirms every candidate workspace package reports `0.8.0`; the external fixture lock is
updated to the same internal versions.

Package-list checks passed for all 16 registry-enabled members. Network-enabled dry-runs passed for
the dependency-free members `talos-core`, `talos-exploration`, `talos-memory`, and `talos-skill`.
The remaining 12 dry-runs stop at the expected registry visibility gate because their internal
`0.8.0` dependencies do not yet exist on crates.io (the index currently exposes only historical
versions or no package). This is a publish-wave prerequisite, not a candidate source failure.

The independent fixture passed in both local registry-shaped candidate modes:

```text
cargo check --locked --manifest-path tests/fixtures/runtime-sdk-external/Cargo.toml
cargo check --locked --manifest-path tests/fixtures/runtime-sdk-external/Cargo.toml --features coding
cargo run --locked --manifest-path tests/fixtures/runtime-sdk-external/Cargo.toml
cargo run --locked --manifest-path tests/fixtures/runtime-sdk-external/Cargo.toml --features coding
talos-runtime external fixture passed
```

The exact candidate also passed `TMPDIR=/private/tmp/talos-model-tests cargo test --workspace
--locked` with loopback access enabled for mock-server tests. A separate release-scoped model
catalog refresh was validated independently and is intentionally excluded from this I204 claim;
it must enter through the effective I203 release claim.
