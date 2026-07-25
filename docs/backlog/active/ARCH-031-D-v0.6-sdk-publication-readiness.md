# ARCH-031-D: v0.6 SDK Fixture And Publication Readiness

| Field | Value |
|---|---|
| Story ID | ARCH-031-D |
| Type | Release Readiness / SDK Validation Story |
| Parent Epic | ARCH-031 |
| Priority | P1 after I161 |
| Status | Refinement — blocked on ARCH-031-C |
| Depends on | I159-I161 Complete; workspace green; maintainer versioning review |
| Selected Iteration | I162 (Planned/Blocked) |
| Value | Prove that the documented SDK and dependency closure work outside the workspace before any publication decision |

## Problem

Workspace tests do not prove that an external Rust project can depend on the supported SDK surface.
The current workspace version and crates.io versions are not aligned. A four-crate shorthand is not
a complete release order.

## Goal

Produce a reproducible external SDK fixture and a complete v0.6 publication-readiness packet based on
the actual Cargo dependency graph.

This Story does not authorize real publication.

## Scope

### Version decision

- Confirm the intended next minor is `0.6.0`.
- Do not bump versions until the maintainer explicitly authorizes the version-alignment step.
- If authorized, update workspace and internal dependency versions consistently.
- Record migration notes for feature-default and SDK API changes.

### External fixture

Create an external-to-workspace fixture under an approved repository path such as:

```text
tests/fixtures/runtime-sdk-external/
```

or a temporary test harness explicitly excluded from workspace membership.

The fixture must behave like an independent consumer and avoid implicit workspace path resolution
except in a dedicated local-path mode.

Required scenarios:

1. minimal runtime construction;
2. custom `LanguageModel`;
3. custom `AgentTool`;
4. approval handler;
5. sandbox provider;
6. default lightweight tool surface;
7. explicit coding feature/preset;
8. sandbox fallback Deny/Ask/AllowUnsandboxed security cases;
9. durable session if part of the supported contract.

### Dependency closure

Use `cargo metadata` to generate the actual topological closure. Record:

- crate name;
- workspace version;
- latest registry version;
- whether a compatible v0.6 registry version exists;
- `publish = false` state;
- blockers;
- dry-run status.

Do not infer the release order from the four logical gate crates.

### Packaging/dry-run

For every authorized publishable closure crate:

```bash
cargo package --locked -p <crate>
cargo publish --locked --dry-run -p <crate>
```

Run in computed dependency order. Record failures without widening scope.

### Documentation

Update:

- publication matrix;
- runtime SDK contract;
- crate docs/readmes;
- README EN/zh-CN distribution sections;
- architecture distribution section;
- migration/release notes;
- ARCH-031 acceptance status only where actually satisfied.

## Explicit Exclusions

- no real `cargo publish`;
- no crates.io owner/name operation;
- no `git tag`;
- no GitHub Release;
- no product release;
- no removal of `publish = false` from product-only/quarantined crates;
- no publication of `talos-models`;
- no v1.0 claim;
- no feature or runtime behavior implementation.

## External Fixture Acceptance

- [ ] fixture builds from a clean environment.
- [ ] fixture uses only documented supported APIs plus explicitly documented direct lower-level
      dependencies.
- [ ] default build does not pull disabled heavy tool families.
- [ ] coding build enables the intended feature set.
- [ ] security scenarios match ARCH-031-C.
- [ ] examples compile with `cargo doc --no-deps`.
- [ ] fixture contains no unpublished product-only dependency.

## Publication Packet Acceptance

- [ ] actual closure is generated from current metadata.
- [ ] every closure crate has package metadata and support boundary.
- [ ] product-only/quarantined guards remain enforced.
- [ ] package and dry-run results are recorded per crate.
- [ ] historical 0.2.0 evidence is clearly separated from current v0.6 readiness.
- [ ] GO/NO-GO names every remaining blocker.
- [ ] packet explicitly states that no publish/tag/release occurred.

## Validation

At minimum:

```bash
cargo metadata --locked --format-version 1
scripts/check_publish_guard.sh .
scripts/validate_project_governance.sh .
cargo fmt --all -- --check
cargo check --workspace --locked
cargo clippy --workspace --locked -- -D warnings
cargo test --workspace --locked
cargo doc --workspace --locked --no-deps
git diff --check
```

Plus fixture and per-crate package/dry-run commands.

## Stop And Escalate Conditions

Stop if:

- version bump is not explicitly authorized;
- a closure crate requires publication of a product-only/quarantined crate;
- registry name/version state conflicts with documentation;
- a package includes secrets, local paths, generated private assets, or unsupported files;
- dry-run requires changing runtime behavior;
- a real publish/tag/release command is requested without fresh authorization.

## Required Reads

- `AGENTS.md`
- program plan
- ADR-024, ADR-052, accepted ADR-053
- ARCH-031 parent and A/B/C
- publication matrix
- runtime SDK contract
- package manifests for all closure crates
- release/governance SOPs

## Residual Destination

Every failed gate receives a named Story. Do not solve unrelated blockers inside I162.
