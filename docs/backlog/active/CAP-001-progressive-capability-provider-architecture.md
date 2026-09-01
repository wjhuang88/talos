# CAP-001: Progressive Capability Provider Architecture

| Field | Value |
|---|---|
| Story ID | CAP-001 |
| Type | Architecture / Domain Epic |
| Priority | P1 |
| Status | Refinement / Unclaimed |
| Source | [GitHub Issue #466](https://github.com/wjhuang88/talos/issues/466) |
| Selected Iteration | None — Epic parents are not selected directly |
| Depends On | ADR-027; ADR-029; PLUGIN-001; TOOL-008; TOOL-012; TOOL-014; DIST-001; WEB-005; current code-truth audit |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned — children require separate non-overlapping claims |
| Claimed At | Not applicable |
| Source Issue | #466 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | None — Epic parents are not implementation units |
| Last Updated | 2026-09-02 |
| Handoff / Release Condition | Refine and accept the Capability/Provider/Plugin/Bundle/Carrier decision, then select only a bounded child with its own runnable iteration and effective claim. |

## Identity / Goal / Value

Converge Talos capability discovery and delivery around one shared vocabulary and ownership model:
Talos core defines stable capabilities, Providers implement them, executable Plugins carry runtime
lifecycle, Bundles own installation/distribution, and Carriers describe execution or connection.
This prevents language, browser, Tool and future Desktop consumers from creating incompatible
registries, loading rules or package semantics.

This Epic is an architecture and decomposition owner. It does not authorize implementation.

## Scope

- Inventory current Plugin, Arborium, text/highlight, Tool disclosure, Browser and distribution
  ownership against repository code truth.
- Create a superseding ADR for the Capability/Provider/Plugin/Bundle/Carrier/Asset boundary while
  preserving ADR-027's WASM safety constraints and the still-valid parts of ADR-029.
- Define object-specific discovery, installation, activation, registration and disclosure
  lifecycles plus safe fallback and startup-network independence.
- Decompose bounded capability-contract, resolver, Bundle, text/language, distribution and Browser
  children with explicit dependencies and compatibility obligations.
- Keep current completed owners accurate: PLUGIN-001 remains the explicit local read-only WASM
  Tool adapter MVP; TOOL-012 remains Tool-family presentation; TOOL-014 remains conditional
  backend/schema disclosure.

## Exclusions

- No runtime registry, resolver, Bundle installer, Plugin loader expansion or Provider code.
- No new crate, Cargo dependency, persisted manifest migration or public API change.
- No WASM language provider, Browser connector, automatic download or marketplace.
- No Desktop/GPUI implementation and no transfer of Session, permission or Tool authority.
- No reopening completed PLUGIN-001, TOOL-012 or existing ADR implementation evidence.

## Child Map

| Child | Outcome | Current State |
|---|---|---|
| CAP-001-P0 / Issue #467 | Characterize current behavior and prepare compatibility seams plus cross-lane ownership before capability implementation. | Refinement / Unclaimed; no iteration or claim |
| CAP-001-A | Stable Capability and Provider descriptor contracts. | Identifier reserved by Issue #466; owner not created |
| CAP-001-B | Capability registry and resolver. | Identifier reserved by Issue #466; owner not created |
| CAP-001-C | Plugin capability declarations and Carrier adapters. | Identifier reserved by Issue #466; owner not created |
| BUNDLE/TEXT/LANG/DIST/BROWSER children | Domain-specific implementation slices after the architecture decision. | Proposed only; owner IDs require separate intake and collision checks |

## Acceptance

- Current repository facts and terminology conflicts are documented without presenting the target
  design as shipped behavior.
- A new ADR defines ownership, lifecycle, registration, fallback and compatibility boundaries and
  explicitly records its relationship to ADR-027 and ADR-029.
- Every implementation outcome is routed to a bounded child; this parent remains unclaimed and is
  never used as implementation authority.
- Default builds remain unchanged until a separately governed child provides locked validation and
  migration evidence.
- Desktop may proceed only under its own owners and must consume shared contracts rather than
  creating a second capability, text, Plugin or Session authority.

## State / Status Owners

- Epic architecture, child map and status: this file.
- Preparatory compatibility Story: `docs/backlog/active/CAP-001-P0-progressive-capability-compatibility-preparation.md`.
- Remote requirement source and discussion: GitHub Issue #466.
- Compact selection view: `docs/backlog/PRODUCT-BACKLOG.md`.
- Open-Issue reconciliation: `docs/reference/ISSUE-DOC-CODE-STATUS-2026-08-31.md`.

## User-Facing Documentation

Future behavior-facing children must update architecture and user documentation for the capability
or installation behavior they actually deliver. This intake must not advertise Provider resolution,
Bundle installation, dynamic language loading or Browser integration as available.

## Required Reads

- GitHub Issues #466 and #467
- `docs/backlog/active/PLUGIN-001-wasm-runtime-plugins.md`
- `docs/backlog/active/TOOL-008-tree-sitter-on-demand.md`
- `docs/backlog/active/TOOL-012-tool-family-progressive-loading.md`
- `docs/backlog/active/TOOL-014-conditional-tool-backends.md`
- `docs/backlog/active/DIST-001-optional-runtime-asset-distribution.md`
- `docs/backlog/active/WEB-005-browser-session-continuity-research.md`
- `docs/backlog/active/DESKTOP-001-desktop-product-direction.md`
- `docs/decisions/027-plugin-runtime-boundary.md`
- `docs/decisions/029-extensibility-atomic-component-model.md`
- `docs/reference/ARCHITECTURE.md`

## Residual Destination

Capability contracts, resolution, Bundle migration, shared text semantics, Language Providers,
distribution and Browser connectors remain separately governed child work after the architecture
decision. No residual implementation is authorized by this Epic intake.
