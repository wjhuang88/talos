# LANG-001: Language Provider Contract And Existing Consumer Migration

| Field | Value |
|---|---|
| Story ID | LANG-001 |
| Type | Language capability contract |
| Parent | CAP-001 / #466 |
| Status | Refinement / Unclaimed |
| Selected Iteration | None |
| Source Issue | [GitHub Issue #510](https://github.com/wjhuang88/talos/issues/510) |
| Depends On | TEXT-001; CAP-001-B/C |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | LanguageProvider contract and migration of existing TUI/symbol consumers to one seam. |
| Claimed At | Not applicable |
| Authorization Evidence | No effective claim; intake owner only. Implementation is not authorized. |
| Governance Claim PR | Not applicable |
| Implementation PR | Not started |
| Authorization Mode | Not applicable |
| Last Updated | 2026-09-09 |
| Handoff / Release Condition | Requires TEXT-001 and a behavior-characterization iteration before implementation. |

## Goal And Scope

Define a shared LanguageProvider surface for highlighting, syntax queries, symbols and outline
consumers, then migrate existing consumers without exposing parser-native types.

## Non-Goals

No WASM language Provider, parser removal, default distribution change, online resolution or Desktop
production binding.

## Acceptance

- Existing highlighting and symbol behavior is characterized and remains equivalent.
- One Provider contract serves TUI and tools with deterministic unavailable fallbacks.
- Arborium ownership is explicit and does not create a second parser registry.
- No startup network dependency or new public breaking rename is introduced.

## Validation And Documentation

Focused language fixtures, TUI/symbol regression tests, dependency-tree checks, API/architecture
documentation, and independent review of shared-file overlap.
