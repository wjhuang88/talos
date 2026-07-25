# 052: SDK Publication And Runtime Composition Boundary

## Status

Accepted

## Context

Talos is intentionally both a product and a reusable Rust ecosystem:

1. the `talos` CLI/TUI/dashboard product;
2. the `talos-runtime` embeddable SDK facade for other Rust projects;
3. independently consumable capability crates such as `talos-core`, `talos-provider`,
   `talos-permission`, `talos-session`, and `talos-skill`.

ADR-024 established `talos-runtime` as the supported embedding facade and kept `talos-agent` as the
turn-loop implementation crate. The crate distribution proposal and ARCH-031 further established a
ripgrep-style publication model in which product crates aggregate reusable libraries rather than
forcing every consumer through one monolithic package.

The current publication boundary is incomplete:

- `talos-runtime` is the primary SDK in documentation but remains blocked by unpublished
  implementation dependencies;
- `talos-agent` is required by the SDK dependency closure, but exposing it as a supported SDK would
  blur the facade/implementation boundary;
- `talos-tools` needs a lightweight default surface before broad external consumption;
- sandbox unavailability has different valid answers for headless services, interactive products,
  and controlled test environments;
- external users need a convenient official coding-agent composition without making that preset the
  only or implicit runtime shape;
- the CLI and SDK must share execution and safety semantics without forcing all product-only CLI
  behavior into the public SDK facade;
- `talos-conversation` has already been published experimentally, but a general-purpose third-party
  UI SDK is not a current product commitment.

A concrete decision is required before completing the route-A publication dependency closure and
before implementation changes add new public APIs.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
| --- | --- | --- | --- |
| Other Rust projects must be able to embed Talos through a documented SDK facade. | Hard | User requirement / RUNTIME-001 / ADR-024 | No |
| Talos-owned capabilities may be independently consumable crates, not only hidden behind the full runtime. | Hard | User requirement / ARCH-031 | No |
| All write-capable tools must pass through the permission pipeline. | Hard | `AGENTS.md` Hard Constraint #4 | No |
| Sandbox and permission changes require explicit security review. | Hard | `AGENTS.md` Hard Constraint #5 | No |
| Public crate APIs are semver-bound; breaking changes need a decision record and migration plan. | Hard | `AGENTS.md` Hard Constraint #6 | No |
| No speculative feature or abstraction may be introduced without a current requirement. | Hard | `AGENTS.md` Hard Constraint #7 / Simplicity First | No |
| Every crate should retain a single clear responsibility and avoid product-layer reverse dependencies. | Hard | `AGENTS.md` Rust crate rules | No |
| `talos-runtime` remains the primary supported SDK entrypoint. | Soft | ADR-024 / runtime SDK contract | Revisit only if facade value disappears |
| `talos-agent` remains the implementation owner of the turn loop. | Soft | ADR-024 / existing architecture | Revisit if a second turn-loop implementation appears |
| CLI and SDK should behave consistently for shared runtime and safety semantics. | Soft | Architecture quality and security auditability | Yes, implementation shape may evolve |
| Default dependency weight should remain small for external library consumers. | Soft | Crate distribution proposal | Yes, based on measured consumer needs |
| A reusable third-party UI SDK may be useful later. | Assumption | Future desktop/web/IDE possibilities | Yes; not validated today |

## Reasoning

### Publication route

There are two principal ways to publish `talos-runtime`:

- **Route A:** publish the complete required dependency closure in a controlled order;
- **Route B:** first decouple the SDK from unpublished implementation dependencies through a larger
  feature/dependency redesign.

Route B may reduce SDK dependency weight eventually, but it would delay the already-designed SDK,
create a wider architecture change before real external consumption, and risk speculative
abstraction. Route A follows the existing architecture and makes the smallest change needed to
honor the current distribution design. Optional-weight feature work remains necessary, especially
for tools, but it is not used as a reason to redesign the entire runtime before publication.

The dependency closure should therefore be hardened and published in this order where dependency
edges require it:

```text
talos-sandbox
    -> talos-tools
        -> talos-agent
            -> talos-runtime
```

Actual publication remains subject to the existing security, documentation, dry-run, maintainer,
and release gates. This ADR decides architecture and sequencing; it is not itself authorization to
publish a crate.

### `talos-agent` support boundary

Publishing an implementation dependency does not require promising it as a stable external SDK.
`talos-agent` must be available in the registry for Cargo dependency resolution under route A, but
external embedders should continue to use `talos-runtime`.

```text
talos-core       canonical protocol and trait foundation
talos-agent      published implementation dependency, unsupported as the primary SDK
talos-runtime    supported embedding facade and safe construction boundary
```

Direct `talos-agent` use remains technically possible under Cargo rules, but its crate-level docs
must state that:

- it is an implementation dependency of `talos-runtime`;
- its public constructors and configuration methods are not covered by the runtime SDK contract;
- callers bypassing `RuntimeBuilder` are responsible for installing equivalent permission,
  approval, and sandbox policy;
- pre-1.0 changes may be more frequent than the facade surface.

### Sandbox fallback ownership

The sandbox crate cannot know whether a caller is an interactive local application, a headless
service, or a controlled test harness. It should report availability and typed execution failure,
not silently choose a product policy.

The SDK must therefore expose an explicit caller-selected policy:

```rust
pub enum SandboxFallbackPolicy {
    Deny,
    Ask,
    AllowUnsandboxed,
}
```

Semantics:

- `Deny`: sandbox-required execution is rejected when isolation is unavailable;
- `Ask`: the runtime routes the unsandboxed fallback decision through the configured approval
  mechanism;
- `AllowUnsandboxed`: the caller explicitly accepts direct execution for that runtime.

The default remains `Deny` so omission is fail-closed. The caller may explicitly select another
policy. The standalone `talos-sandbox` crate continues to return typed availability/errors and does
not import runtime or UI policy.

### Lightweight `talos-tools` defaults

`talos-tools` is a capability crate for external consumers as well as the Talos product. Its default
features should minimize dependency weight and attack surface. The default should include only
local read-oriented file/search capability. Mutating, process, network, and heavy code-intelligence
families require explicit features or an explicit higher-level preset.

```text
default: file-read + search
optional: file-write, shell, git, http, web-search, code-intelligence, image
aliases: coding, network, full
```

Architectural rules:

- default features must not silently enable shell, network, Git write, or destructive file tools;
- read and write file capability should be separable where dependency and permission boundaries
  remain coherent;
- the Talos product may explicitly enable a broader set;
- sibling tool crates are created only after independent consumers or measured dependency weight
  justify the split.

### Official coding preset

A minimal `RuntimeBuilder::new()` must remain composition-first and must not silently install a full
coding-agent product. External users also need a supported fast path that reproduces Talos-owned
coding defaults without copying internal registry construction.

```rust
let runtime = RuntimeBuilder::new()
    .preset(RuntimePreset::coding())
    .provider(provider)
    .workspace_root(workspace)
    .sandbox_fallback(SandboxFallbackPolicy::Ask)
    .build()?;
```

Presets are explicit, inspectable, and overridable. The coding preset may select official tool
families, prompt additions, permission defaults, and sandbox expectations, but it must not hide
write/execute/network actions from the existing permission pipeline.

### CLI and SDK composition relationship

Three options were considered:

1. force `talos-cli` to use only the public `talos-runtime` SDK facade;
2. let CLI and SDK maintain independent assembly implementations;
3. keep distinct public entrypoints while sharing the internal composition and execution semantics.

Option 1 would pressure the narrow SDK facade to expose product-only concerns. Option 2 would create
behavior and security drift. Option 3 preserves both support boundaries.

```text
                       talos-core
                           |
                       talos-agent
                           |
                 shared internal composition
                    /                    \
          talos-runtime              talos-cli
          public SDK facade          product assembly
```

The CLI and SDK are required to share the implementation of:

- Agent/session actor construction;
- tool execution pipeline;
- permission evaluation semantics;
- sandbox selection and fallback semantics;
- hook ordering where the same hooks are installed;
- canonical session command/event flow;
- common coding-preset registry construction.

The shared layer must not contain CLI parsing, TUI rendering, dashboard routes, setup flows, or other
product-only behavior. Initially it may be an internal module in an existing implementation crate.
A new `talos-runtime-core`-style crate is not authorized by this decision. Extraction into a new
crate requires evidence that module visibility or dependency direction cannot support both real
consumers cleanly.

### Deferred general-purpose UI SDK

`talos-conversation` remains useful as Talos's UI-independent product state/projection layer, and
its separation from `talos-tui` remains architecturally valid. However, Talos is not currently
committing to a general-purpose third-party UI SDK.

Therefore:

- `talos-conversation` remains an experimental pre-1.0 crate;
- existing publication is not reversed or hidden;
- documentation must not market it as a supported general-purpose UI framework;
- Talos-specific command, validation, and governance projection logic does not need to be extracted
  solely for a hypothetical external UI consumer;
- a reusable UI SDK is reconsidered only when a real second frontend outside the current product
  needs a stable contract.

This deferral does not authorize moving conversation state back into the TUI. The current
single-direction session -> conversation projection -> UI architecture remains in force.

## Decision

1. **Complete publication through route A.**
   - Harden and publish the required dependency closure in dependency order:
     `talos-sandbox` -> `talos-tools` -> `talos-agent` -> `talos-runtime`.
   - Existing publish/security/release gates remain mandatory.
   - This ADR does not authorize a real crates.io publish by itself.

2. **Publish `talos-agent` only as an implementation dependency.**
   - It is not the recommended or supported SDK entrypoint.
   - Its docs must direct embedders to `talos-runtime` and state direct-use caveats.

3. **Make sandbox fallback an explicit SDK caller choice.**
   - Add typed `Deny`, `Ask`, and `AllowUnsandboxed` semantics.
   - Default to `Deny`.
   - `talos-sandbox` remains policy-neutral.

4. **Use lightweight `talos-tools` defaults.**
   - Default features are local and read-oriented.
   - Write, shell, Git, network/web, image, and heavy code-intelligence families are opt-in or enabled
     through an explicit higher-level preset.

5. **Provide an explicit official coding preset in `talos-runtime`.**
   - Keep `RuntimeBuilder::new()` minimal and composition-first.
   - Provide `RuntimePreset::coding()` or an equivalent explicit preset value.
   - The preset is overridable and never bypasses permissions or approvals.

6. **Keep separate CLI and SDK public entrypoints with shared internal composition.**
   - Do not force product-only CLI behavior through the public runtime facade.
   - Do not maintain duplicate Agent/session/tool/security assembly implementations.
   - Share canonical construction and safety semantics through an internal module first.
   - Creating a new composition crate requires a later demonstrated need.

7. **Defer a general-purpose UI SDK.**
   - Keep `talos-conversation` experimental and product-oriented for now.
   - Do not promise third-party UI compatibility or stability.
   - Preserve the existing conversation/TUI separation and ordered session-event architecture.

## Clarifying Amendments (2026-07-26)

These amendments refine, and do not change, the main Decision above. They were added after a
documentation/governance drift audit to remove ambiguity in how the decision is read.

1. **The four-gate order is logical, not a release command sequence.**
   `talos-sandbox → talos-tools → talos-agent → talos-runtime` is the logical order of the **remaining
   gate crates only**. It is not the complete, directly-executable release command sequence. The real
   `talos-runtime` publication closure is wider — it includes already-published foundation crates
   (`talos-core`, `talos-permission`, `talos-skill`, `talos-plugin`, `talos-memory`, `talos-session`)
   in addition to the four gate crates. The actual release order MUST be generated from the current
   `cargo metadata` dependency graph, and every closure crate must ship a version compatible with the
   current workspace version before `talos-runtime` can resolve. The four-item list is a gate-ordering
   shorthand, not a publish script.

2. **`SandboxFallbackPolicy` is orthogonal to permission policy.**
   `AllowUnsandboxed` NEVER implies that any tool, path, execute, or network permission has been
   granted. Normal permission evaluation (permission rules, tool natures, `Deny` precedence) still
   runs in full. The fallback policy ONLY decides whether execution may continue when sandbox
   isolation is unavailable — it never authorizes an action that the permission pipeline would
   otherwise reject.

3. **`Ask` fallback is a distinct, scoped approval, not a tool-permission approval.**
   - The runtime MUST present an identifiable sandbox-fallback reason/context to the approval layer,
     distinct from a normal tool-permission approval.
   - A sandbox-fallback approval MUST NOT be conflated with a tool permission approval; granting one
     does not grant the other.
   - Authorization MUST be scoped to at least the current invocation/runtime; it MUST NOT silently
     become a process-wide or permanent allowance.
   - With no approval handler, `Ask` MUST fail closed (equivalent to `Deny`).
   - A normal `AlwaysApprove` tool-permission rule MUST NOT auto-permanently-allow unsandboxed
     execution. Tool permission scope and sandbox-fallback scope are independent.

4. **Preset precedence is explicit-over-implicit and cannot weaken security.**
   - Explicit caller configuration (permission rules, sandbox policy, tool selection) overrides preset
     defaults.
   - A preset MUST NOT override or weaken an explicit `Deny`, a permission rule, or a sandbox
     requirement.
   - A preset only provides a default composition; it gains NO additional authorization capability
     beyond what the caller could configure directly.

5. **`talos-tools` lightweight-default acceptance requires real optional dependencies, not hidden registration.**
   - Heavy dependencies (`gix`, `arborium`, `reqwest`/`scraper`/`rust-websearch`, `image`, shell/process
     native bindings) MUST become truly OPTIONAL Cargo dependencies gated behind features.
   - The corresponding modules AND their public re-exports MUST be behind feature gates.
   - A default build MUST NOT resolve or compile shell, network, Git, image, or code-intelligence
     heavy dependencies. Merely hiding tool registration while still compiling the dependency is NOT
     sufficient for acceptance.

These amendments describe target semantics. `SandboxFallbackPolicy`, `RuntimePreset`, and the
`talos-tools` feature gates remain **not yet implemented** until their implementation commits land
through ARCH-031 slices.

## Implementation Plan

Implementation must be activated through ARCH-031 and normal iteration governance.

### Phase 1: Contract and manifest alignment

- update crate-level docs for `talos-agent`, `talos-runtime`, `talos-tools`, `talos-sandbox`, and
  `talos-conversation`;
- update `RUNTIME-SDK-CONTRACT.md` with caller-selected sandbox fallback and the official coding
  preset;
- update the publication matrix with current workspace/published versions and route-A ordering;
- verify internal dependencies carry publish-compatible `version` plus `path` specs;
- record migration notes for any public pre-1.0 API change.

### Phase 2: Lightweight tool features

- inventory tool families, optional dependencies, process/native boundaries, and permission facets;
- define and test lightweight defaults;
- ensure `--no-default-features` and selected feature combinations compile;
- make the Talos product explicitly enable its required tool families;
- run permission-profile review for every mutating, execute, and network feature.

### Phase 3: Sandbox publication gate

- review platform behavior and escape vectors;
- document availability, unsupported platforms, fallback ownership, and typed failures;
- verify dependency failures cannot terminate the host process;
- run targeted sandbox and process-hardening tests;
- obtain explicit maintainer publication authorization.

### Phase 4: Shared composition and SDK policy

- identify duplicate CLI/runtime assembly paths;
- introduce one internal composition implementation without adding a speculative crate;
- add `SandboxFallbackPolicy` and fail-closed defaults;
- add the official coding preset using the same registry and safety pipeline as the product;
- add equivalence tests for shared tool registration, permission defaults, and session semantics.

### Phase 5: Dependency-closure publication

- dry-run and, only after explicit release authorization, publish in dependency order;
- keep `talos-agent` documentation marked implementation-only;
- publish `talos-runtime` only after all required registry dependencies resolve cleanly;
- validate examples from a clean external Cargo project rather than only the workspace.

## Validation And Acceptance

Required validation should include, as applicable:

```text
cargo metadata --locked --no-deps --format-version 1
cargo check --locked --workspace
cargo test --locked --workspace
cargo test --locked -p <affected-crate>
cargo publish --dry-run --locked -p <crate>
cargo check --locked -p talos-tools --no-default-features
cargo check --locked -p talos-tools --no-default-features --features <feature-set>
scripts/validate_project_governance.sh .
```

Acceptance requires:

- `talos-runtime` can be consumed from a clean external Rust project;
- the documented minimal runtime does not silently install a full coding toolset;
- the official coding preset reproduces shared Talos coding composition without bypassing security;
- sandbox unavailability follows the selected typed fallback policy;
- default `talos-tools` features do not enable shell/network/destructive capability;
- CLI and SDK shared semantics are covered by tests rather than maintained by documentation alone;
- `talos-agent` is resolvable as a dependency but clearly outside the supported SDK contract;
- `talos-conversation` documentation states its experimental, product-oriented boundary;
- owner documents contain actual validation and completion commit evidence before any status becomes
  `Complete`.

## Rejected Alternatives

- **Publish only `talos-runtime`.** Rejected because Cargo still needs a valid implementation
  dependency closure and independently reusable capability crates are an explicit product goal.
- **Redesign the runtime first to avoid publishing implementation crates.** Rejected for the current
  route because it is a larger speculative change than completing the existing architecture.
- **Treat `talos-agent` as a second supported SDK.** Rejected because it weakens the facade boundary.
- **Automatically fall back to unsandboxed execution.** Rejected because omission must fail closed.
- **Enable the complete coding toolset by default in `talos-tools`.** Rejected due dependency weight
  and unnecessary default attack surface.
- **Make the coding preset implicit in `RuntimeBuilder::new()`.** Rejected because the minimal SDK
  must remain predictable and composition-first.
- **Force the CLI to depend only on public SDK methods.** Rejected because it would expand the SDK
  with product-only concerns.
- **Maintain independent CLI and SDK assembly.** Rejected because it creates security and behavior
  drift.
- **Create a new shared-composition crate immediately.** Rejected as premature.
- **Promote `talos-conversation` to a supported UI SDK now.** Rejected because no validated second
  external frontend currently requires the commitment.

## Migration And Compatibility

- Additive builder methods, preset types, and policy enums follow the pre-1.0 SDK change policy.
- Any replacement of an existing sandbox boolean or implicit fallback behavior requires at least one
  minor-version migration note and, where practical, a deprecated compatibility method for one
  minor cycle.
- Changing `talos-tools` default features may alter transitive dependency and capability behavior.
  Release notes must list the old and new defaults and show explicit feature selections for users
  who need the former full set.
- `talos-agent` publication must not retroactively claim its existing API as stable.
- Existing `talos-conversation` users retain normal pre-1.0 semver treatment, but documentation must
  avoid unsupported general-purpose UI guarantees.

## Reversal Triggers

Revisit this decision when one of the following becomes true:

- route A produces unacceptable default compile time, binary size, platform reach, or dependency
  risk that cannot be addressed with feature gates;
- two real external consumers require direct `talos-agent` access that cannot be promoted cleanly
  through `talos-runtime`;
- CLI and SDK cannot share composition without a dependency cycle or unstable public leakage,
  providing evidence for a dedicated composition crate;
- a real non-Talos desktop, web, or IDE frontend requires a supported conversation/UI contract;
- sandbox fallback semantics prove insufficient for a supported host category;
- post-1.0 independent crate versioning changes the dependency-closure or support model.

## Related

- [ADR-024: Embeddable Runtime API Boundary](024-embeddable-runtime-api-boundary.md)
- [ADR-029: Extensibility Atomic Component Model](029-extensibility-atomic-component-model.md)
- [ADR-039: Runtime Event Semantic Single-Flow Boundary](039-runtime-event-semantic-single-flow.md)
- [ADR-042: Embedded Durable Runtime Session Boundary](042-embedded-durable-runtime-session-boundary.md)
- [ARCH-031: Crate Publication Boundary And Distribution Architecture](../backlog/active/ARCH-031-crate-publication-boundary.md)
- [RUNTIME-001: Embeddable Agent Runtime API](../backlog/active/RUNTIME-001-embeddable-agent-runtime-api.md)
- [Talos Crate Distribution Architecture](../proposals/talos-crate-distribution-architecture.md)
- [Talos Crate Publication Matrix](../reference/CRATE-PUBLICATION-MATRIX.md)
- [talos-runtime SDK Support Contract](../reference/RUNTIME-SDK-CONTRACT.md)
