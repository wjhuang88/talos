# REL-003: v0.8.0 GitHub And Crates.io Publication

**Status**: Blocked
**Type**: Release / Distribution Story
**Parent Epic**: ARCH-031

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Execute the validated v0.8.0 GitHub Release first, then publish the authorized 20-package Cargo closure and verify external installation/consumption. |
| Claimed At | Not applicable |
| Source Issue | None |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Maintainer requested a pre-I196 release, including `talos-cli` and `talos-runtime`, and later fixed GitHub-before-Cargo ordering. I203 remains unclaimed until I162 produces a GO packet. |
| Implementation PR | Not started |
| Last Updated | 2026-08-14 |
| Handoff / Release Condition | I159-I162 Complete; I162 GO packet names the exact closure and version; I203 receives its own effective claim and fresh irreversible-action checkpoint. |

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
- I162 publication packet says GO for v0.8.0 and the exact 20-package closure.
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
