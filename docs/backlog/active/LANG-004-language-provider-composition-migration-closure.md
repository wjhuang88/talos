# LANG-004: Language Provider Composition And Migration Completion

> Document status: Refinement / Unclaimed

| Field | Value |
|---|---|
| Story ID | LANG-004 |
| Type | Language Provider Architecture / Migration Completion |
| Priority | P1 |
| Status | Refinement / Unclaimed |
| Source | [GitHub Issue #616](https://github.com/wjhuang88/talos/issues/616) |
| Selected Iteration | None |
| Depends On | CAP-001; LANG-001; LANG-002; LANG-003; CAP-001-D; TOOL-008; Issue #317 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Own multi-provider language composition, deterministic routing and language-by-language ownership transfer out of the transitional single-provider model |
| Claimed At | Not applicable |
| Source Issue | #616 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-09-29 |
| Handoff / Release Condition | Refine child decomposition and migration/fallback policy before provider-router or built-in grammar retirement implementation |

## Scope

- Define one authoritative multi-provider routing/composition model keyed by canonical LanguageId.
- Preserve unrelated built-in languages while multiple external providers coexist.
- Define deterministic duplicate-provider, lifecycle-revocation and fallback semantics.
- Converge language metadata ownership and provide an explicit per-language built-in retirement path.
- Keep TUI highlighting and symbol/code-intelligence consumers on the same routing decision.

## Exclusions

No wholesale language migration, Arborium/Tree-sitter replacement, startup network dependency, silent fallback broadening, or TOOL-008 completion claim is authorized.

## Acceptance

- Multiple providers coexist deterministically and load order cannot select semantics.
- Rust/Python external providers can coexist with unrelated built-in languages.
- At least one later claimed child proves real ownership transfer and static grammar retirement with compatibility/footprint evidence.
- Remaining language classes have explicit owners or an explicit built-in decision before LANG-004 can close.
