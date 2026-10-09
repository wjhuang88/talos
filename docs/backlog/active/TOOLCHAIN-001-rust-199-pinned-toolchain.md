# TOOLCHAIN-001: Rust 1.99 Pinned Toolchain Maintenance

> Document status: Planned / Unclaimed

| Field | Value |
|---|---|
| Type | Toolchain and CI maintenance |
| Priority | P2 |
| Source | Maintainer-authorized maintenance request |
| Selected Iteration | I296 (proposed; activation pending) |
| Depends On | Current `main`; effective I296 Collaboration Claim |

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
| Authorization Evidence | Maintainer authorized preparation on 2026-10-09; implementation remains prohibited until an effective target-branch claim exists. |
| Implementation PR | Not started |
| Last Updated | 2026-10-09 |
| Handoff / Release Condition | Atomic claim+activation must merge to `main` before implementation begins. |

## Scope

- Pin the developer, CI, and release Rust toolchain to Rust 1.99.
- Update the current-toolchain assertion and installation documentation to the same version.
- Address only direct source incompatibilities reported by Rust 1.99.

## Non-Goals

- No Cargo dependency or `Cargo.lock` update.
- No workspace MSRV change; it remains Rust 1.95.
- No release, tag, publication, runtime behavior change, or unrelated cleanup.

## Acceptance

- `rust-toolchain.toml`, CI, release workflow, assertion and installation documentation agree on Rust 1.99.
- The workspace continues to compile, test and lint using the pinned toolchain with locked resolution.
- Direct Rust 1.99 compatibility edits preserve existing behavior and remain compatible with the declared MSRV.

## Planned Validation

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets --locked`
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- `./scripts/release_preflight.sh`
- `./scripts/validate_public_site.sh v0.10.1`
- `./scripts/validate_installers.sh`

## Residuals

- Existing third-party future-incompatibility notice for transitive `block v0.1.6` is not changed by this maintenance slice.
