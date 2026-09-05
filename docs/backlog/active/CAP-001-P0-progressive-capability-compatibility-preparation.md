# CAP-001-P0: Progressive Capability Compatibility Preparation

| Field | Value |
|---|---|
| Story ID | CAP-001-P0 |
| Type | Architecture Preparation / Compatibility Story |
| Priority | P1 |
| Status | Active / Claimed (proposed; ineffective until governance merge) |
| Parent Epic | CAP-001 |
| Source | [GitHub Issue #467](https://github.com/wjhuang88/talos/issues/467) |
| Selected Iteration | I246 |
| Depends On | CAP-001 architecture decision; current code-truth characterization; active Desktop/shared-file overlap inventory |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline session |
| Work Slice | Bounded code-alignment: characterization, minimal UI-neutral text/language seam, existing-consumer migration, Plugin package/runtime isolation and dependency guards; no full CAP-001 providers/resolver, Desktop production binding, persisted/public rename or release/publication. |
| Claimed At | 2026-09-05 |
| Source Issue | #467 |
| Governance Claim PR | Pending |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer explicitly authorized narrowed scope on 2026-09-05. |
| Implementation PR | Not started |
| Last Updated | 2026-09-05 |
| Handoff / Release Condition | This bounded claim is proposed and ineffective until governance merge; code/Cargo changes are limited to behavior-preserving CAP-001-P0 alignment and its tests. |

## Identity / Goal / Value

Prepare the current repository for progressive capability work without changing shipped behavior.
The eventual deliverable will place existing Arborium-backed language/text behavior and
package-versus-runtime Plugin terminology behind reviewed compatibility seams so later CAP,
BUNDLE, TEXT, LANG and Desktop work does not require one uncontrolled cross-workspace rewrite.

This intake records the proposed executable Story boundary only. It is not Ready, selected or
authorized for implementation.

## Proposed Scope Requiring Refinement

- Characterize all direct Arborium dependencies, enabled language features, parser call sites,
  TUI highlighting/fallback behavior, symbol behavior and relevant release-size evidence.
- Decide whether a focused `talos-text` boundary is necessary without duplicating
  `talos-conversation`, and define UI-neutral inputs/results that expose no Arborium or Tree-sitter
  native types.
- Preserve current TUI and symbol behavior through one built-in compatibility integration before
  any dynamic Provider work.
- Inventory package-oriented `PluginManifest`/`[plugin]` compatibility separately from executable
  Plugin loading; prepare, but do not execute, the Bundle terminology migration.
- Define enforceable dependency guards and a changed-file/merge-order contract for shared code and
  a separately governed Desktop lane.

## Exclusions

- No generic CapabilityResolver, dynamic Provider runtime or online resolution.
- No Bundle download, installation, marketplace or persisted `BundleManifest` migration.
- No WASM language provider, parser trimming/default-distribution change or Browser connector.
- No GPUI, Desktop window, Desktop production binding or Desktop-owned shared semantic model.
- No Session/Preset, permission, Work Graph/Evaluator, release, version or publication changes.
- No broad `talos-plugin` rename and no new native, FFI or `unsafe` dependency.

## Readiness Gates

- CAP-001's superseding architecture decision is Accepted or this Story is explicitly narrowed to
  decision-compatible characterization only.
- The actual boundary between `talos-conversation` and the proposed shared text/language seam is
  decided against current code and public API responsibilities.
- Active Desktop and shared-code claims are inventoried, with ownership for root `Cargo.toml`,
  `Cargo.lock`, shared projection types and CI workflows explicitly coordinated.
- One runnable/testable iteration preserves behavior, names user/API documentation, and carries a
  complete changed-file authority boundary.
- A Collaboration Claim for that iteration is effective on the target branch before any code,
  Cargo, dependency or persisted-format edit starts.

## Acceptance For A Future Selected Slice

- Highlighting, language normalization, fallback and symbol behavior are characterized before
  movement and remain equivalent afterward.
- TUI and symbol consumers use one UI-neutral seam with a deterministic plain-text or
  provider-unavailable fallback and no startup network dependency.
- Shared result types expose no Arborium/Tree-sitter, Ratatui, Crossterm or GPUI types.
- Package/distribution terminology and executable Plugin runtime responsibilities have a concrete
  compatibility/migration matrix; current persisted/public names are not silently renamed.
- Existing PLUGIN-001 safety, timeout, trap, output, provenance and permission-denial evidence
  remains green.
- Desktop receives a fixture/adapter handoff without importing GPUI into shared crates or creating
  a second capability/text/session authority.

## State / Status Owners

- Story scope, readiness and status: this file.
- Parent architecture and child order: `docs/backlog/active/CAP-001-progressive-capability-provider-architecture.md`.
- Detailed requirement source and acceptance discussion: GitHub Issue #467.
- Compact selection view: `docs/backlog/PRODUCT-BACKLOG.md`.
- Open-Issue reconciliation: `docs/reference/ISSUE-DOC-CODE-STATUS-2026-08-31.md`.

## User-Facing Documentation

A future implementation iteration must update architecture/API documentation and any affected TUI
or symbol capability descriptions. This intake makes no user-visible behavior claim.

## Required Reads

- GitHub Issues #466, #467, #29 and #308
- `docs/backlog/active/CAP-001-progressive-capability-provider-architecture.md`
- `docs/backlog/active/PLUGIN-001-wasm-runtime-plugins.md`
- `docs/backlog/active/TOOL-008-tree-sitter-on-demand.md`
- `docs/backlog/active/TOOL-012-tool-family-progressive-loading.md`
- `docs/backlog/active/TOOL-014-conditional-tool-backends.md`
- `docs/backlog/active/DIST-001-optional-runtime-asset-distribution.md`
- `docs/backlog/active/WEB-005-browser-session-continuity-research.md`
- `docs/backlog/active/DESKTOP-001-D0-renderer-host-boundary.md`
- `docs/backlog/active/DESKTOP-001-desktop-product-direction.md`
- `docs/decisions/027-plugin-runtime-boundary.md`
- `docs/decisions/029-extensibility-atomic-component-model.md`
- `docs/decisions/052-sdk-publication-and-composition-boundary.md`
- `docs/reference/ARCHITECTURE.md`
- `docs/sop/REQUIREMENT-INTAKE.md`
- `docs/sop/AGENT-COLLABORATION.md`

## Residual Destination

The accepted CAP-001 ADR and separately owned CAP/BUNDLE/TEXT/LANG/DIST/BROWSER children own target
architecture implementation. Desktop hosts, renderers, i18n and production binding remain under
Desktop owners. None of those residuals are authorized here.
