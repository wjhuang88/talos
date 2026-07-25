# ARCH-031-C: Explicit Sandbox Fallback Policy And Official Coding Preset

| Field | Value |
|---|---|
| Story ID | ARCH-031-C |
| Type | SDK / Security-Sensitive API Story |
| Parent Epic | ARCH-031 |
| Priority | P1 |
| Status | Refinement — blocked on ARCH-031-B and security review |
| Depends on | ADR-024; ADR-052; I160 Complete; permission/sandbox security review scheduled |
| Selected Iteration | I161 (Planned/Blocked) |
| Value | Embedders can choose a fail-closed sandbox fallback and opt into Talos coding defaults without copying product internals |

## Problem

The SDK needs explicit behavior when sandbox isolation is unavailable and an official way to request
Talos coding composition. ADR-052 defines the desired boundary, but the APIs are not implemented.

Implementing these APIs before shared composition would copy product logic and risk permission drift.

## Goal

Add:

```rust
pub enum SandboxFallbackPolicy {
    Deny,
    Ask,
    AllowUnsandboxed,
}
```

and an explicit coding preset surface such as:

```rust
RuntimePreset::coding()
```

through `talos-runtime`, using the shared composition from I160.

## API Constraints

The exact Rust signatures must be recorded in I161 before implementation. They must satisfy:

- default fallback is `Deny`;
- caller omission is fail-closed;
- preset is explicit, inspectable, and overridable;
- caller configuration takes precedence over preset defaults;
- additive API preferred;
- breaking changes require a pre-1.0 migration note and deprecation where practical.

## Sandbox And Permission Semantics

Sandbox fallback and permission are orthogonal.

`AllowUnsandboxed`:

- allows continuing only when required isolation is unavailable;
- grants no tool permission;
- grants no path permission;
- grants no execute permission;
- grants no network permission;
- does not bypass `Deny`;
- does not change tool nature.

`Ask`:

- carries a typed/identifiable sandbox-fallback reason/context;
- is not the same approval meaning as a normal tool permission request;
- is scoped at least to the current invocation/runtime;
- cannot silently create a permanent broad allow rule;
- fails closed as `Deny` when no approval handler exists;
- is not automatically widened by an ordinary `AlwaysApprove` tool rule.

`talos-sandbox` remains policy-neutral and exposes typed availability/errors only.

## Preset Semantics

`RuntimePreset::coding()`:

- selects the approved coding capability composition;
- uses the same shared internal composition as CLI;
- does not hide write/execute/network tools from permission evaluation;
- does not authorize sandbox fallback;
- does not override explicit caller tool selection, permission rules, sandbox requirement, or fallback;
- has no additional authorization capability;
- is documented as a convenience composition, not a security profile.

## Scope

- add SDK types and builder methods;
- add typed approval context if required by the chosen API;
- thread fallback policy through runtime/agent execution without placing policy in `talos-sandbox`;
- implement the coding preset through I160 composition;
- preserve existing callers through additive/default behavior or migration;
- add focused permission/sandbox tests;
- update SDK contract from Planned to Supported only after implementation and evidence.

## Explicit Exclusions

- no permission-default relaxation;
- no automatic unsandboxed execution;
- no new sandbox backend;
- no `talos-sandbox` product policy;
- no UI-specific approval flow redesign beyond required typed context;
- no new composition crate;
- no real publish;
- no tag/release;
- no unrelated RuntimeBuilder cleanup.

## Security Test Matrix

| Permission result | Sandbox available | Fallback | Handler | Expected |
|---|---|---|---|---|
| Deny | any | any | any | denied |
| Allow | available | any | any | sandboxed execution |
| Allow | unavailable | Deny | any | denied |
| Allow | unavailable | Ask | none | denied |
| Allow | unavailable | Ask | approves scoped fallback | unsandboxed execution for approved scope only |
| Allow | unavailable | Ask | rejects | denied |
| Allow | unavailable | AllowUnsandboxed | any | unsandboxed execution, permission still evaluated |
| Ask normal tool permission | unavailable | AllowUnsandboxed | none | denied because permission remains unresolved |
| AlwaysApprove normal tool rule | unavailable | Ask | no fallback approval | denied |

Add path/network/execute variants and adversarial cases.

## Invariants

- permission `Deny` always wins;
- no approval handler means fail closed;
- fallback approval is distinguishable in logs/hooks/UI projections without leaking secrets;
- a preset cannot weaken security;
- CLI behavior changes only if explicitly selected and documented;
- old runtime construction remains minimal by default;
- no policy logic moves into `talos-sandbox`.

## Acceptance

### API

- [ ] public API and migration note reviewed before merge.
- [ ] `SandboxFallbackPolicy` default is `Deny`.
- [ ] coding preset is explicit.
- [ ] caller overrides win.
- [ ] SDK docs accurately show supported types.

### Security

- [ ] independent security review approves the matrix.
- [ ] all matrix cases have focused tests.
- [ ] ordinary `AlwaysApprove` cannot substitute for fallback approval.
- [ ] headless `Ask` denies.
- [ ] permission/path/network/execute decisions remain enforced.

### Composition

- [ ] coding preset and CLI use the same shared composition implementation.
- [ ] registry/capability equivalence test passes.
- [ ] no new crate or global registry appears.

### Runtime evidence

- [ ] embedded fixture proves Deny.
- [ ] embedded fixture proves headless Ask denies.
- [ ] embedded fixture proves scoped Ask approval.
- [ ] embedded fixture proves AllowUnsandboxed still respects permission Deny.
- [ ] coding preset can complete one read-only turn and one permission-gated tool scenario.

### Validation

Focused tests, full locked validation, docs, Story/Iteration/Board sync.

## Stop And Escalate Conditions

Stop if:

- approval context cannot distinguish fallback from normal tool permission;
- implementation requires weakening an existing Deny/default;
- policy must be added to `talos-sandbox`;
- caller precedence is ambiguous;
- a public API break lacks migration review;
- shared composition from I160 is incomplete.

## Required Reads

- `AGENTS.md`
- program plan
- ADR-024
- ADR-052
- accepted ADR-053
- ARCH-031-B
- `docs/reference/RUNTIME-SDK-CONTRACT.md`
- permission and sandbox ADRs/tests
- `talos-runtime`, `talos-agent`, `talos-permission`, and `talos-sandbox` implementation paths

## Residual Destination

- new sandbox backend: separate Story/ADR;
- richer approval UX: separate product Story;
- real crate publication: ARCH-031-D.
