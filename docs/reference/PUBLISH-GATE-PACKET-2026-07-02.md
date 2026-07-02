# Publish Gate Packet: `talos-cli` And `talos-runtime`

**Date**: 2026-07-02
**Plan item**: T133
**Owner**: ARCH-031 / I079
**Result**: publish remains blocked; no real publish, tag, or release action performed

## Scope

This packet evaluates the current publication posture for:

- `talos-runtime` as the SDK facade candidate;
- `talos-cli` as the future Cargo binary-install candidate for `cargo install talos-cli --bin talos`.

It is a release gate artifact only. It does not authorize publishing, crate name reservation,
removing `publish = false`, tagging, or creating a GitHub Release.

## Commands Run

- `scripts/check_publish_guard.sh .`
- `cargo metadata --no-deps --format-version 1`
- `cargo package --list -p talos-runtime`
- `cargo package --list -p talos-cli`
- `cargo publish --dry-run -p talos-cli`
- `cargo publish --dry-run -p talos-runtime`
- `scripts/validate_project_governance.sh .`

## Guard Results

| Check | Result | Notes |
|---|---|---|
| Product-only publish guard | Pass | `talos-cli`, `talos-tui`, `talos-evolution`, and `talos-dashboard` carry `publish = false`. |
| Gate-before-publish guard | Pass | `talos-sandbox`, `talos-tools`, `talos-agent`, `talos-runtime`, and `talos-mcp` remain manifest-ready and review-gated, not hard-blocked by `publish = false`. |
| `talos-cli` dry-run | Blocked as intended | Cargo reports `talos-cli` cannot be published because `package.publish` is not enabled. |
| `talos-runtime` dry-run | Blocked | Cargo reports no matching package named `talos-agent` in the crates.io index. |
| Governance validation | Pass | `scripts/validate_project_governance.sh .` passed with 0 warnings after this packet updated owner docs. |

## Blocker Matrix

| Package | Intended distribution | Current state | Blocker before real publish |
|---|---|---|---|
| `talos-cli` | Future binary package for `cargo install talos-cli --bin talos` | `publish = false`; package list includes the `talos` binary source and CLI README | Install-package gate not complete; crates.io metadata/readme/keywords/categories still need finalization; maintainer has not approved removing `publish = false`; real publish is not authorized. |
| `talos-runtime` | SDK facade crate | Manifest-ready; package list includes SDK examples | Dependency closure is not publishable because `talos-agent` is not published; `talos-agent` depends on high-risk `talos-tools`/`talos-sandbox` gates; maintainer has not approved publish. |
| `talos-dashboard` | Product-only loopback control surface | `publish = false`; added to publish guard in T133 | No crate publication planned under ADR-031/WEB-001. Any dashboard package distribution needs a new decision. |

## Required Order Before `talos-runtime`

`talos-runtime` cannot be published before its normal dependencies resolve through crates.io or are
decoupled. Current required order remains:

1. `talos-sandbox` safety gate;
2. `talos-tools` feature/permission gate;
3. `talos-agent` implementation-surface gate;
4. `talos-runtime` SDK facade dry-run and publish gate.

This packet does not say those crates should be published. It only records the dependency closure
that would have to be satisfied before `talos-runtime` can pass dry-run.

## Non-Actions

- No `cargo publish` without `--dry-run`.
- No `publish = false` removal.
- No tag.
- No GitHub Release.
- No crate name reservation.
- No version bump.

## Decision

Keep current guards in place:

- `talos-cli` remains product-only until a dedicated binary install-package gate is complete and
  the maintainer explicitly approves publish.
- `talos-runtime` remains gate-before-publish until dependency closure is safe or decoupled.
- `talos-dashboard` is explicitly product-only and covered by the publish guard.
