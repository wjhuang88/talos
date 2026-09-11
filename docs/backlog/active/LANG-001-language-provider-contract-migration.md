# LANG-001: Language Provider Contract And Existing Consumer Migration

| Field | Value |
|---|---|
| Story ID | LANG-001 |
| Type | Language capability contract |
| Parent | CAP-001 / #466 |
| Status | Complete / Closed |
| Selected Iteration | I257 |
| Source Issue | [GitHub Issue #510](https://github.com/wjhuang88/talos/issues/510) |
| Depends On | TEXT-001; CAP-001-A |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex unattended single-developer mode |
| Work Slice | LanguageProvider contract and migration of existing TUI/symbol consumers to one seam. |
| Claimed At | 2026-09-11 |
| Authorization Evidence | Claim #533 effective on main; implementation #534 merged. |
| Governance Claim PR | #533 |
| Implementation PR | #534 |
| Authorization Mode | Independent review |
| Last Updated | 2026-09-11 |
| Handoff / Release Condition | Closed after merge commit `f6b77b51`; follow-up language slices remain separately governed. |

Completion Commit: `f6b77b5162b2dcfea1a9d849fee60b11f96b6d0e`

## Required Reads

- [CAP-001 parent](CAP-001-progressive-capability-provider-architecture.md), [ADR-072](../../decisions/072-capability-provider-bundle-boundary.md), and [TEXT-001](TEXT-001-ui-neutral-text-semantics.md).
- [TOOL-008](TOOL-008-tree-sitter-on-demand.md) and I246 evidence; feature trimming is not Provider loading.

## Goal And Scope

Define a shared LanguageProvider surface for highlighting, syntax queries, symbols and outline
consumers, then migrate existing consumers without exposing parser-native types.

This contract and existing-consumer migration do not require Plugin/Carrier loading.
CAP-001-C and verified installation instead gate the separate LANG-002 WASM slice.

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
