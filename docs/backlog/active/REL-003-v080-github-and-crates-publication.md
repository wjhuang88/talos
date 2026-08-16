# REL-003: v0.8.0 GitHub And Crates.io Publication

**Status**: Active / In Progress
**Type**: Release / Distribution Story
**Parent Epic**: ARCH-031

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | `Codex / GPT-5 mainline release-governance session` |
| Work Slice | `REL-003 / I203` only: synchronize the v0.8.0 release candidate and release surfaces, refresh `models.toml` from the reviewed upstream snapshot, run exact release preflight, create one immutable GitHub tag/Release first, then publish the authorized Cargo closure in dependency waves and verify external CLI/SDK consumption. No Desktop, Dashboard, Work Graph, Evaluator or RUNTIME-006 work. |
| Claimed At | 2026-08-16 |
| Source Issue | None |
| Governance Claim PR | #262 |
| Authorization Mode | Independent review |
| Authorization Evidence | I204 closeout is effective at `main@8eaa22a2` with reviewed conditional GO for preparing I203 claim; I203 claim PR #262 merged as `f6b2d243`, and this activation checkpoint records the now-effective claim. |
| Implementation PR | Not started |
| Last Updated | 2026-08-16 |
| Handoff / Release Condition | I203 claim is effective at `main@f6b2d243`; this activation checkpoint precedes a fresh implementation branch. Preserve I204/I162 baselines; implementation PR must separately authorize release-surface/version changes. GitHub Release must complete before any real Cargo publish. |

## Identity / Goal / Value

Release Talos v0.8.0 through the existing five-platform GitHub distribution path, then make the CLI
installable with Cargo and publish `talos-runtime` plus every required dependency to crates.io.

## Scope

- synchronize v0.8.0 workspace/internal versions and release surfaces;
- run the repository release preflight and create an immutable annotated `v0.8.0` tag;
- wait for the GitHub Release, five platform archives and checksums to complete successfully;
- only after that gate, publish the authorized Cargo closure in metadata-derived dependency order;
- verify registry visibility between dependency waves;
- verify `cargo install talos-cli --version 0.8.0 --bin talos --locked` externally;
- verify an external current-contract `talos-runtime = "0.8.0"` fixture;
- close owner, Board and release evidence without claiming REL-002 or v1.0 readiness.

## Exclusions

- `talos-models` remains quarantined and `publish = false`;
- no RUNTIME-006 / Issue #234 API implementation;
- no Desktop, Dashboard, Work Graph or Evaluator work;
- no tag move or force push;
- no partial-publish rollback claim: crates.io releases are immutable.

## Dependencies

- I159, I160, I161 and I162 Complete in order.
- I204 readiness packet gives a reviewed conditional GO for the exact v0.8.0 candidate and 20-package closure; I162 remains the reviewed historical NO-GO predecessor.
- GitHub Release success is a hard predecessor to every real `cargo publish` command.
- Cargo credentials, verified crates.io ownership and rate-limit capacity must be available at the
  publication checkpoint.

## Decision Links And Constraints

- ADR-052 controls SDK/publication composition.
- `docs/sop/RELEASE-WORKFLOW.md` controls the tag and GitHub Release.
- `AGENTS.md` forbids moving failed tags; a failure uses a new patch release.
- crates.io publication is irreversible. Retry only an unpublished package/version after registry
  visibility checks; never overwrite or yank as an automated rollback.

## State/Status Owners

- Story truth: this file.
- Execution: I203 and the 2026-08-14 v0.8.0 publication task.
- Readiness predecessor: I162 / ARCH-031-D.
- Derived views: Product Backlog, iteration index and Board.

## User-Facing Documentation

- `README.md`, `README.zh-CN.md`;
- paired site installation/documentation surfaces;
- `docs/releases/v0.8.0.md`;
- crate docs and `docs/reference/RUNTIME-SDK-CONTRACT.md`.

## Required Reads

- `AGENTS.md`
- `docs/sop/RELEASE.md`
- `docs/sop/RELEASE-WORKFLOW.md`
- ARCH-031 and ARCH-031-D
- I159-I162 and I203
- publication matrix and I162 GO packet

## Acceptance For Behavior

- Given I162 has produced a reviewed GO packet
  When v0.8.0 is released
  Then the GitHub Release and all five assets/checksums complete before the first Cargo package is
  published.
- Given the GitHub Release gate passed
  When the metadata-derived package waves are published
  Then a clean external environment can install the `talos` binary from `talos-cli` and compile the
  documented v0.8.0 runtime SDK contract.

## Acceptance For Technical/Governance Work

- [ ] `./scripts/release_preflight.sh v0.8.0` passes on the exact release commit.
- [ ] Annotated tag `v0.8.0` is pushed once and the GitHub Release has five archives plus checksum.
- [ ] No `cargo publish` command runs before GitHub Release success is recorded.
- [ ] All 20 authorized packages are visible at version 0.8.0 in dependency order.
- [ ] External Cargo install and runtime SDK fixtures pass without workspace path resolution.
- [ ] Partial publication, retry and new-patch recovery evidence is recorded truthfully.
- [ ] Completion cites pre-existing release/publish evidence commits and external run identifiers.

## Residual Destination

RUNTIME-006 owns the stronger one-direct-dependency SDK facade. REL-002 retains stable/v1
qualification. Any failed or omitted package receives a new release recovery task and patch version.

## 2026-08-16 I203 Claim Preparation Checkpoint

Fresh exact-main inventory at `main@8eaa22a296edd3b657511e3ebe72a2d2b8afa2e2` found no Active or
Review iteration. I204 is Complete/Closed with Completion Commit `f46094e3`; I164 remains Paused;
I188, I189, I195 and I196 remain Planned/Claimed and unactivated; I203 is the proposed release
claim and remains ineffective until PR #262 merges. The claim includes the pre-release
`models.toml` refresh, workspace version alignment, release surfaces, immutable tag, GitHub-first
Release, dependency-ordered Cargo publication, `talos-cli --bin talos`, and `talos-runtime`
external fixture. It excludes `talos-models`, RUNTIME-006, and unrelated product work.

No version, model, tag, GitHub Release, or Cargo publication change is authorized by this claim
preparation record. Implementation must start from the claim merge head and use a fresh
irreversible-action checkpoint with merge-time CAS.

## 2026-08-16 I203 Activation Checkpoint

Claim PR #262 merged as `f6b2d2439a3bad7732f4f0b046569d97f8b9f73e`, making the I203 claim
effective. Owner status is now `Active / In Progress / Claimed`; implementation remains `Not
started`. A fresh implementation branch must start from this exact activation main. The Work Slice
may now implement the reviewed release candidate, including `models.toml` refresh, version/release
surface alignment, release preflight, immutable GitHub-first Release, dependency-ordered Cargo
publication, and external CLI/SDK acceptance. No implementation commit, tag, GitHub Release, or
real Cargo publish is authorized by this activation record alone; each requires its own exact-head
review and merge-time CAS.
