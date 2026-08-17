# ARCH-031-C: Explicit Sandbox Fallback Policy And Official Coding Preset

> Document status: Complete (2026-08-15)

| Field | Value |
|---|---|
| Story ID | ARCH-031-C |
| Type | SDK / Security-Sensitive API Story |
| Parent Epic | ARCH-031 |
| Priority | P1 |
| Status | Complete / Closed — I161 |
| Depends on | ADR-024; ADR-052; I160 Complete; independent security review recorded in Issue #245 |
| Selected Iteration | I161 (Complete/Closed) |
| Value | Embedders can choose a fail-closed sandbox fallback and opt into Talos coding defaults without copying product internals |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | `Codex / GPT-5 mainline session` |
| Work Slice | `ARCH-031-C / I161` only: `SandboxFallbackPolicy`, explicit coding preset, typed fallback approval context if required, security matrix tests, runtime evidence, and SDK documentation; no I162 publication or release work. |
| Claimed At | 2026-08-15 |
| Source Issue | None |
| Governance Claim PR | #244 |
| Authorization Mode | Independent review |
| Authorization Evidence | Claim PR #244 merged as `b570ac27`; implementation PR #250 exact-head APPROVE bound to `74c5502d` and matrix-closure PR #251 exact-head APPROVE bound to `8b3ca5fc`, both with shared-account identity limits disclosed. |
| Implementation PR | #250; matrix-closure follow-up #251 |
| Last Updated | 2026-08-15 |
| Handoff / Release Condition | None — I161 is complete; I162 and publication remain separately governed. |

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

## Formal Security Review Record

Issue #245 is the formal pre-implementation security review record accepted on 2026-08-15.
The complete `ARCH-031-C` security chapters and nine-row matrix are normative; the issue summary
is not a lossy substitute. The review confirms these required invariants: permission `Deny` always
wins; `AllowUnsandboxed` only permits continuation when required isolation is unavailable and never
bypasses permission; headless `Ask` fails closed; fallback approval is typed, scoped, distinguishable
from ordinary approval, and cannot silently create a permanent broad grant; ordinary `AlwaysApprove`
does not approve fallback; coding composition cannot weaken permission or sandbox constraints; CLI
behavior changes only on explicit selection; old construction remains minimal by default; and
policy stays out of `talos-sandbox`. Path, network, and execute variants remain in scope.

The security reviewer and implementation roles are separate, with shared-account identity limits
disclosed. This is a design/matrix review, not implementation acceptance. The final implementation
head must receive a fresh independent exact-head security review against the complete matrix before
merge, and the acceptance checkbox below remains open until that evidence exists.

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

- [x] public API and migration note reviewed before merge.
- [x] `SandboxFallbackPolicy` default is `Deny`.
- [x] coding preset is explicit.
- [x] caller overrides win.
- [x] SDK docs accurately show supported types.

### Security

- [x] independent security review approves the matrix.
- [x] all matrix cases have focused tests.
- [x] ordinary `AlwaysApprove` cannot substitute for fallback approval.
- [x] headless `Ask` denies.
- [x] permission/path/network/execute decisions remain enforced.

### Composition

- [x] coding preset and CLI use the same shared composition implementation.
- [x] registry/capability equivalence test passes.
- [x] no new crate or global registry appears.

### Runtime evidence

- [x] embedded fixture proves Deny.
- [x] embedded fixture proves headless Ask denies.
- [x] embedded fixture proves scoped Ask approval.
- [x] embedded fixture proves AllowUnsandboxed still respects permission Deny.
- [x] coding preset can complete one read-only turn and one permission-gated tool scenario.

### Validation

Focused tests, full locked validation, docs, Story/Iteration/Board sync.

## Completion Evidence

- Completion Commit: `74c5502d8860316070182c0cf2366d5adf57ea6c` and `3ca2ec62b3e91d88c345f5bba15e986cb31f606c` (pre-existing implementation/test commits).
- Implementation PR #250 merged as `d2b4bdd12f69f1eaffeade7e05625369a7d4f8aa`; matrix-closure PR #251 merged as `da5a43a244ee17902fb001b2445b4ec54cbf206c`.
- Exact-head CI: `31873172667` and `31878744293`, both 5/5 SUCCESS; independent approvals bound to both implementation heads.

## Completion Checkpoint 2026-08-15

I161/ARCH-031-C is Complete/Closed. Non-blocking M1/M4/N4/N5/N6 remain residual follow-ups and do not weaken the delivered acceptance boundary.

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
