# ARCH-031-D: v0.6 SDK Fixture And Publication Readiness

> Document status: Complete (2026-08-15; readiness qualification NO-GO)

| Field | Value |
|---|---|
| Story ID | ARCH-031-D |
| Type | Release Readiness / SDK Validation Story |
| Parent Epic | ARCH-031 |
| Priority | P1 after I161 |
| Status | Complete |
| Depends on | I159-I161 Complete; workspace green; maintainer versioning review |
| Selected Iteration | I162 (Complete/Closed) |
| Value | Prove that the documented SDK and dependency closure work outside the workspace before any publication decision |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | `Codex / GPT-5 mainline release-governance session` |
| Work Slice | `ARCH-031-D / I162` only: external consumer fixture, metadata-derived publishable closure, per-crate package and `cargo publish --dry-run` evidence, and explicit GO/NO-GO packet for the candidate v0.8.0 release. No runtime behavior, version bump, tag, GitHub Release, or real Cargo publication. |
| Claimed At | 2026-08-15 |
| Source Issue | None |
| Governance Claim PR | #253 |
| Authorization Mode | Independent review |
| Authorization Evidence | I161 closeout is effective at `main@2301434a`; claim PR #253 exact head `913a3318` passed CI `31886470911`, independent approval `5302451715`, governance validators, and merge-time CAS, then became effective on `main` as `38127228`. |
| Implementation PR | #255 |
| Last Updated | 2026-08-15 |
| Handoff / Release Condition | Readiness is complete with reviewed `NO-GO`; I203 remains blocked until a separate claim and reviewed GO packet authorize release work. |

Completion Commit: `077b347dff25f60e6fbd84b22548f58c72163f65`

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

## Activation Checkpoint 2026-08-15

I162/ARCH-031-D is Active/In Progress after claim PR #253 merged as `381272289eb3d87204f022a562be847bc649cd97`.
The active Work Slice remains readiness-only: external SDK fixture, metadata-derived closure,
per-crate package and `cargo publish --dry-run` evidence, and a candidate v0.8.0 GO/NO-GO packet.
No version bump, runtime behavior change, tag, GitHub Release, or real Cargo publication is
authorized. Registry metadata access is an explicit external validation gate; a registry failure
must remain a named blocker rather than being inferred away.

## Closeout Checkpoint 2026-08-15

ARCH-031-D/I162 is Complete/Closed with reviewed `NO-GO` publication qualification. Completion
Commit: `077b347dff25f60e6fbd84b22548f58c72163f65`; implementation PR #255 merged as
`16564ba01fe69ee95297898c8faab1c1701e5bb2`; exact-head CI `31891263313` passed 5/5; independent
approval is comment `5302842269`. The 20-member closure and four guarded product crates are in the
readiness packet. No version, tag, release, or publication action occurred.
