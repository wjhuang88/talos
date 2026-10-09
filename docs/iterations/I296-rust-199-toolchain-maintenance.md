# Iteration I296: Rust 1.99 Toolchain Maintenance

> Document status: Planned
> Published plan date: 2026-10-09
> Planned objective: advance the pinned development, CI and release compiler to Rust 1.99 without changing the Rust 1.95 workspace MSRV or dependency graph.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a locked, runnable workspace using the Rust 1.99 pinned toolchain.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | None |
| Governance Claim PR | Pending |
| Authorization Mode | Not applicable |
| Authorization Evidence | Maintainer authorized the maintenance request on 2026-10-09. This proposal establishes no ownership; implementation waits for an effective target-branch claim. |
| Implementation PR | Not started |
| Last Updated | 2026-10-09 |
| Handoff / Release Condition | Atomic claim+activation must merge before code/configuration commits are created. |

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| TOOLCHAIN-001 | None | Planned / Unclaimed | Effective I296 claim | Rust 1.99 is the consistent pinned development, CI and release toolchain. |

### Selection Inventory And Disposition

- I293 and I294 are Active / Claimed under separate Search work slices; no overlap.
- I291 and I290 are Review / Claimed; no overlap.
- DEPENDENCY-003-A is Review / Claimed; its `libc` migration is excluded.
- I295 is Complete / Closed; I249 remains deferred Planned; I164 remains Paused.

### Scope

- Update the Rust pin in local development, CI and release configuration to 1.99.
- Keep documentation and the toolchain measurement assertion aligned with that pin.
- Make only compiler-required direct source compatibility corrections.

### Non-Goals

- No package/MSRV/dependency-resolution changes, including `Cargo.lock`.
- No Rust business, permission, sandbox, process-hardening or release behavior change.
- No tag or publication.

### Acceptance

- Given a clean checkout using the pinned toolchain, when locked workspace validation runs, then format, check, Clippy, tests and release preflight pass.
- Given the declared Rust 1.95 workspace MSRV, when the direct compatibility edits are compiled with it, then the edits remain source-compatible.
- Given installation documentation and automation, when they name the current compiler, then they name Rust 1.99.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets --locked`
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- `./scripts/release_preflight.sh`
- `./scripts/validate_public_site.sh v0.10.1`
- `./scripts/validate_installers.sh`
- `scripts/validate_project_governance.sh .`
- `bash scripts/validate_collaboration_claims.sh .`

### Documentation To Update

- `rust-toolchain.toml`, CI and release workflow pins.
- `site/install.html`, `site/zh/install.html`, and the version assertion used by the locale measurement script.
- TOOLCHAIN-001 owner, I296 execution evidence, Backlog, iteration index and Board.

### Risks And Rollback

- Risk: Rust 1.99 identifies source/API incompatibilities or exposes a dependency future-incompatibility notice.
- Rollback: retain the existing pin and route any behavior-changing repair to a separately governed owner.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-10-09 | Proposed selection | Local candidate exploration passed the listed checks, but is uncommitted and non-authoritative until the atomic claim+activation reaches `main`. |

## Verification Evidence

- Pre-claim local exploration: candidate passed all planned validation commands, including full `./scripts/release_preflight.sh`; this is not implementation evidence until recreated from the effective claim merge.

## Completion Evidence

- Completion Commit: pending

## Variance And Residuals

- Existing transitive `block v0.1.6` future-incompatibility notice remains outside scope.
