# RUNTIME-008: Runtime Event Compatibility Closeout

> Document status: Refinement / Unclaimed

| Field | Value |
|---|---|
| Story ID | RUNTIME-008 |
| Type | Runtime API Compatibility / Migration Closeout |
| Priority | P2 |
| Status | Refinement / Unclaimed |
| Source | [GitHub Issue #613](https://github.com/wjhuang88/talos/issues/613) |
| Selected Iteration | None |
| Depends On | SESSION-011; ADR-039; ARCH-033; I115; I172; SESSION-009 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Close legacy SessionEvent and UiOutput compatibility after canonical ordered TurnEvent convergence |
| Claimed At | Not applicable |
| Source Issue | #613 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-09-29 |
| Handoff / Release Condition | Inventory current producers/consumers and select a deprecation/removal release plan before any public compatibility change |

## Scope

- Inventory remaining legacy unwrapped SessionEvent/UI compatibility surfaces.
- Record canonical replacements, public compatibility obligations, deprecation window and removal trigger.
- Keep any migration shim narrow and prevent new internal reliance on legacy event semantics.

## Exclusions

No redesign of canonical TurnEvent semantics, multi-client replay implementation, TLOG change, custody/settlement change or premature public API removal is authorized.

## Acceptance

- Remaining compatibility surfaces and consumers are enumerated.
- Deprecation and removal triggers satisfy the SDK compatibility contract.
- Canonical ordering, persistence and external fixture coverage remain protected.
