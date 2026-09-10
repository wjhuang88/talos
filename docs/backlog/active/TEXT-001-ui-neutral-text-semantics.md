# TEXT-001: UI-Neutral Text Semantics

| Field | Value |
|---|---|
| Story ID | TEXT-001 |
| Type | Shared text contract |
| Parent | CAP-001 / #466 |
| Status | Refinement / Unclaimed |
| Selected Iteration | I256 (preparation; not activated) |
| Source Issue | [GitHub Issue #511](https://github.com/wjhuang88/talos/issues/511) |
| Depends On | CAP-001-A; ADR-072 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | UI-neutral text classification, language identity and semantic result contract. |
| Claimed At | Not applicable |
| Authorization Evidence | No effective claim; intake owner only. Implementation is not authorized. |
| Governance Claim PR | Pending |
| Implementation PR | Not started |
| Authorization Mode | Not applicable |
| Last Updated | 2026-09-10 |
| Handoff / Release Condition | Requires a selected iteration and explicit overlap agreement with TUI/Desktop consumers. |

## Required Reads

- [CAP-001 parent](CAP-001-progressive-capability-provider-architecture.md) and [ADR-072](../../decisions/072-capability-provider-bundle-boundary.md).
- I246/CAP-001-P0 compatibility evidence; this contract must remain UI-neutral and does not authorize parser loading or distribution.

## Goal And Scope

Define shared text/code-block semantics and deterministic plain-text fallback for TUI, Desktop and
tools. Results must not expose Ratatui, GPUI, Arborium or Tree-sitter types.

CAP-001-C is not a prerequisite for this contract. It remains Refinement pending the
I253 acceptance audit and consumer ownership inventory; scheduling CAP-001-B first
does not create a technical dependency on its Plugin/Carrier branch.

## Non-Goals

No parser trimming, dynamic loading, Bundle installation, WASM provider, UI layout, Desktop binding,
or language-specific implementation.

## Acceptance

- Language IDs and aliases normalize deterministically.
- Semantic results are renderer-independent and streaming-safe.
- Unsupported, unavailable or malformed providers fall back explicitly to plain text.
- TUI and Desktop can consume the same contract without importing each other's crates.

## Validation And Documentation

Contract fixtures, fallback/property tests, architecture/API docs and an explicit shared-file
ownership inventory. No default Cargo feature expansion is permitted by this owner.

## I256 Selection Preparation (2026-09-10)

I253 audit is Complete, as are CAP-001-A and I246 compatibility preparation. The consumer
inventory and bounded shared-file agreement are recorded in
[I256](../../iterations/I256-ui-neutral-text-semantics.md): reuse existing `talos-text`, extract
renderer-neutral stream classification and validate semantic fallback, with narrow TUI adapters.
Current main has no competing Active/Review iteration or open PR. This is a proposed serial
ownership boundary, not effective authority before atomic claim merge and plan review.
Preserve public constructors/serialized forms, canonical-only TUI grammar admission, existing
stream rendering, Cargo defaults and dependency versions. LanguageProvider dispatch and symbol
migration remain LANG-001; parser loading, distribution and Desktop binding remain excluded.
No implementation code is authorized by this preparation.
