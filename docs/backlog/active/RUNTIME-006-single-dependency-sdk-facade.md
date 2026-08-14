# RUNTIME-006: Single-Direct-Dependency Runtime SDK Facade

**Status**: Refinement
**Type**: Public API / SDK Story
**Parent Epic**: ARCH-031

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Define and implement a supported `talos-runtime` facade that lets third-party consumers use the core runtime with no other direct Talos dependency. |
| Claimed At | Not applicable |
| Source Issue | #234 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-14 |
| Handoff / Release Condition | Refine the provider strategy and pre-1.0 compatibility treatment, create a new runnable iteration, and establish its own effective claim before API implementation. |

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
- The provider integration strategy and migration treatment must be resolved before Ready.
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

The unresolved question is whether provider construction belongs entirely behind facade re-exports
or whether `talos-provider` is documented as an optional convenience package while custom providers
use facade-owned traits. Resolve that choice in requirement refinement and prove it with the external
fixture before moving this Story to Ready.

## State/Status Owners

- Story truth: this file.
- External intake: GitHub Issue #234.
- Parent publication architecture: ARCH-031.
- Iteration: none selected.
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
