# RUNTIME-006: Single-Direct-Dependency Runtime SDK Facade

**Status**: In Progress (proposed; effective only after #583 merges)
**Selected Iteration**: I280
**Type**: Public API / SDK Story
**Parent Epic**: ARCH-031

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 |
| Work Slice | Define and implement a supported `talos-runtime` facade that lets third-party consumers use the core runtime with no other direct Talos dependency. |
| Claimed At | 2026-09-21 |
| Source Issue | #234 |
| Governance Claim PR | #583 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer requested #234 development closure on 2026-09-21; independent Agent API/security review and exact-head checks required. |
| Implementation PR | Not started |
| Last Updated | 2026-09-21 |
| Handoff / Release Condition | I280 implements the complete facade and external acceptance after #583 establishes effective claim; close only after implementation evidence and review. |

## Identity / Goal / Value

Third-party Rust embedders should be able to declare `talos-runtime` as their only direct Talos
dependency and still implement or inject the provider, tool, message/event, permission and sandbox
types required by the supported runtime composition surface.

Today that is not true. `RuntimeBuilder` and `RuntimeHandle` expose types from `talos-core`,
`talos-permission` and `talos-sandbox`, and the quickstart documents those direct dependencies.

## Scope

- define the supported runtime-only facade and its pre-1.0 compatibility boundary;
- re-export or wrap every provider, tool, message, event, permission and sandbox type needed by the
  public builder/handle contract;
- decide how a consumer supplies a provider without forcing an internal-crate dependency;
- update rustdoc, the SDK contract, quickstart and examples;
- add an external fixture whose manifest names only `talos-runtime` among Talos packages.

## Exclusions

- no crates.io publication, tag or product release;
- no CLI/TUI behavior change;
- no credential distribution or permission/sandbox relaxation;
- no v1.0 or REL-002 readiness claim;
- no expansion of the v0.8.0 publication scope.

## Dependencies

- ADR-024 and ADR-052 remain the SDK/composition boundary.
- ARCH-031 owns the crate publication architecture.
- Provider and compatibility choices are resolved below under ADR-024/ADR-052; #583 claim merge
  remains required before implementation.
- v0.8.0 may publish the current documented multi-direct-dependency SDK contract; this stronger
  facade requirement is a separately claimed follow-up and is not a hidden release gate.

## Decision Links And Constraints

- `docs/decisions/024-embeddable-runtime-api-boundary.md`
- `docs/decisions/052-sdk-publication-and-composition-boundary.md`
- `docs/reference/RUNTIME-SDK-CONTRACT.md`
- `docs/backlog/active/ARCH-031-crate-publication-boundary.md`

Crate public APIs are semver-bound. Any breaking path replacement needs a recorded migration plan
and, if it changes the accepted public boundary, an ADR amendment before implementation.

## Uncertainty And Validation Path

2026-09-21 refinement: custom providers implement the canonical trait re-exported through runtime;
`talos-provider` remains an optional convenience package. Add explicit curated exports without
replacing canonical types or removing old imports. This follows ADR-024/ADR-052; no breaking
replacement is authorized. I280's independent fixture proves the complete associated-type closure,
permission/sandbox behavior and full turn lifecycle. No workspace dev-dependency leakage is allowed.

## State/Status Owners

- Story truth: this file.
- External intake: GitHub Issue #234.
- Parent publication architecture: ARCH-031.
- Iteration: `docs/iterations/I280-runtime006-single-dependency-sdk.md`.
- Derived views: Product Backlog and Board.

## User-Facing Documentation

- `docs/reference/RUNTIME-SDK-CONTRACT.md`
- runtime quickstart/example documentation;
- `README.md` and `README.zh-CN.md` SDK dependency examples when behavior lands.

## Required Reads

- `AGENTS.md`
- `docs/sop/REQUIREMENT-INTAKE.md`
- `docs/sop/AGENT-COLLABORATION.md`
- ADR-024 and ADR-052
- ARCH-031
- `docs/reference/RUNTIME-SDK-CONTRACT.md`
- `crates/talos-runtime/src/lib.rs`
- `crates/talos-runtime/Cargo.toml`

## Acceptance For Behavior

- Given an external Rust fixture whose manifest declares only `talos-runtime` among Talos crates
  When it implements or injects the supported provider/tool, configures permission and sandbox
  behavior, builds a runtime, submits a turn, receives typed events and shuts down
  Then it compiles and runs without importing or declaring another `talos-*` crate.

## Acceptance For Technical/Governance Work

- [ ] The external fixture has exactly one direct Talos dependency.
- [ ] Public rustdoc and the SDK contract name supported facade paths and compatibility policy.
- [ ] Provider, tool, message/event, permission and sandbox composition paths are exercised.
- [ ] Existing imports receive documented pre-1.0 migration or compatibility treatment.
- [ ] Locked workspace validation and the external fixture pass.
- [ ] A separate iteration and effective Collaboration Claim exist before implementation.

## Residual Destination

Provider convenience implementations that remain intentionally separate belong to
`talos-provider`; unrelated runtime presets, product composition and REL-002 qualification stay in
their existing owners.
