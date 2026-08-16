# ARCH-031-E: v0.8.0 Release-Candidate Registry Readiness

> Document status: Active / In Progress

| Field | Value |
|---|---|
| Story ID | ARCH-031-E |
| Type | Release Readiness / Version Alignment Story |
| Parent Epic | ARCH-031 |
| Priority | P1 after I162 |
| Status | Active / In Progress |
| Depends on | I162 Complete/Closed with reviewed NO-GO; current `main` at activation |
| Selected Iteration | I204 (Active/Claimed) |
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
| Authorization Evidence | Claim PR #257 merged to `main` as `e6cab51a61c3a23a1e0a6792573bcef688a3d6dd`; activation is recorded after the claim became effective. |
| Implementation PR | Not started |
| Last Updated | 2026-08-16 |
| Handoff / Release Condition | I204 starts from `main@e6cab51a`; preserve I162 baselines, produce a reviewed GO/NO-GO packet, and keep I203 Blocked/Unclaimed. Candidate version edits remain isolated and cannot authorize tag, GitHub Release, or Cargo publication. |

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

## 2026-08-16 Activation Checkpoint

Claim PR #257 is effective on `main@e6cab51a61c3a23a1e0a6792573bcef688a3d6dd`. I204 is now
`Active / In Progress` for readiness evidence only. The implementation branch must be recreated
from this activation head and may record candidate-only version alignment, package metadata,
registry visibility, dry-run, fixture, and reviewed GO/NO-GO evidence. It must not change the
published baseline, refresh release surfaces, create a tag or GitHub Release, or perform a real
Cargo publication. I203 remains `Blocked / Unclaimed` until this packet is independently reviewed.
