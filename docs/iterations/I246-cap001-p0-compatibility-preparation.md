# Iteration I246: CAP-001-P0 Compatibility Preparation

> Document status: Active / Claimed
> Planned date: 2026-09-04
> Planned objective: prepare the current codebase for progressive capability work through a characterized, UI-neutral text/language and Plugin compatibility boundary without changing shipped behavior.
> Baseline rule: preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a runnable compatibility characterization and seam handoff that lets later capability and Desktop slices integrate without duplicate parser or Plugin authorities.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline session |
| Work Slice | I246 / CAP-001-P0 bounded code-alignment: characterize current behavior, introduce the smallest UI-neutral text/language seam, migrate existing TUI/symbol consumers, isolate Plugin package/runtime compatibility, and add dependency guards. Excludes full CAP-001 providers/resolver, Desktop production binding, persisted/public renames and release/publication. |
| Claimed At | 2026-09-05 |
| Source Issue | #467 |
| Governance Claim PR | #491 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer explicitly authorized the narrowed scope on 2026-09-05. |
| Implementation PR | #492 (stable candidate; exact head evolves only through batched local convergence) |
| Last Updated | 2026-09-05 |
| Handoff / Release Condition | Claim effective on merge commit `f913d28b`; implementation may change bounded shared code and Cargo edges only within the published CAP-001-P0 scope, preserving behavior and exclusions. |

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| CAP-001-P0 / #467 | CAP-001 / #466 | Planned / Unclaimed | CAP-001 architecture decision or an explicit decision-compatible narrowing; current code-truth inventory | Characterize current behavior, define compatibility seams and hand off non-overlapping child boundaries. |

### Scope

- Inventory Arborium/Tree-sitter dependencies and call sites, current TUI highlighting and symbol behavior, Plugin manifest/runtime terminology, binary-size evidence and Desktop/shared-file overlap.
- Decide the smallest UI-neutral text/language seam against talos-conversation ownership, with opaque result types and deterministic built-in fallback.
- Preserve current TUI, symbol and Plugin behavior through characterization and bounded migration.
- Define dependency guards and a changed-file/merge-order handoff for later Desktop and capability children.

### Non-Goals

- No CapabilityResolver, dynamic Provider, Bundle installer, Browser connector, WASM language provider, GPUI, talos-desktop, Desktop runtime binding, new native/FFI/unsafe dependency, persisted/public Plugin rename, release, version, publication, permission or Session change.

### Acceptance

- Given the current workspace, when the inventory and characterization run, then all direct Arborium call sites, language aliases, fallback behavior and symbol behavior are recorded with focused regression evidence.
- Given a proposed seam, when its dependency graph is checked, then shared results expose no Arborium/Tree-sitter, Ratatui, Crossterm or GPUI types and no second consumer-owned parser authority is introduced.
- Given the Plugin compatibility matrix, when future children are selected, then package format and executable runtime responsibilities remain distinct without silently renaming persisted fields.
- Given the Desktop lane, when changed-file ownership is reviewed, then Desktop may consume fixtures/adapters later without importing TUI or creating a second capability/text authority.

### Planned Validation

- scripts/validate_project_governance.sh .
- bash scripts/validate_collaboration_claims.sh .
- Focused TUI highlighting and symbol compatibility tests.
- Dependency/import and changed-file boundary checks.
- git diff --check and locked validation applicable to the implementation diff.

### Documentation To Update

- CAP-001 and CAP-001-P0 owner documents.
- docs/reference/ARCHITECTURE.md or a focused compatibility reference.
- TUI/symbol/API documentation affected by any seam.
- docs/BOARD.md, Product Backlog, iteration index and the long-task ledger after owner updates.

### Risks And Rollback

- Risk: seam placement duplicates talos-conversation or changes fallback behavior.
- Rollback: preserve existing consumer paths, record incompatibility, and route a new decision/child owner before changing public or persisted contracts.

## Activation Checkpoint — 2026-09-05

Claim PR #491 merged as `f913d28b027530bf43a70db21e1ef2c8eb89f5bc` after exact-head CI `33958282011` and independent review approval. Implementation must start from this merge or later main.

## Current Nonterminal Inventory And Disposition — 2026-09-05

| Item | Current state | Disposition |
|---|---|---|
| I207 / TUI-049 | Complete / Closed | Predecessor complete; no overlap. |
| I208 / TUI-050 | Complete / Closed | Predecessor complete; no overlap. |
| I246 / CAP-001-P0 | Active / Claimed | Claim #491 is effective at `f913d28b`; implementation proceeds from that merge. |
| #466 / CAP-001 | Refinement / Unclaimed | Parent architecture remains separate; no full provider/resolver implementation here. |
| Future CAP/TEXT/LANG/BUNDLE/BROWSER children | Planned or unselected | Remain separately governed and unactivated. |

This inventory is a governance checkpoint, not implementation authorization beyond the bounded Work Slice.

## Execution Order

### Local Code-Truth Inventory — 2026-09-05

- `talos-tui/src/highlight.rs` owns `arborium::Highlighter` and converts Arborium spans directly to Crossterm-colored line segments; parse failures and the 500ms budget fall back to plain rendering.
- `talos-tools/src/symbol.rs` owns a separate Arborium parser path, including extension-to-language aliases and AST symbol/outline traversal.
- The two consumers therefore have duplicate parser authorities and expose renderer/parser coupling that the bounded seam must remove while preserving these fallback and alias behaviors.
- `talos-conversation` currently provides UI-independent conversation projection, not language parsing; seam placement must avoid duplicating that responsibility.

### Local Seam Contract Draft — 2026-09-05

The bounded seam will expose normalized language identifiers, immutable source text, highlight
spans, and provider-unavailable/plain-text outcomes. Consumer-facing types will contain no
Arborium, Tree-sitter, Ratatui, Crossterm, or GPUI values. Existing TUI colors remain a renderer
mapping; symbol consumers retain their current location and error semantics. This draft is an
implementation constraint, not a new CAP-001 architecture decision.

This iteration is intentionally scheduled after I207 and I208 in the long task. It must not start until the steering slices have reached terminal dispositions and its own architecture/overlap gates are satisfied.

## Completion Evidence

- Completion Commit: {already-existing implementation SHA}
- Status-only documentation commits must not cite themselves.

## Local Convergence Checkpoint — 2026-09-05

- Implemented neutral language, highlight, location, and symbol contracts in `talos-text`.
- TUI and symbol consumers convert/consume neutral values while retaining Arborium at their
  existing built-in adapter boundaries; Plugin package/runtime responsibilities are documented in
  `docs/reference/CAP-001-P0-compatibility-seams.md`.
- Evidence: `cargo test -p talos-text --locked` (3 passed),
  `cargo test -p talos-tui --locked --lib` (570 passed),
  `cargo test -p talos-tools --locked --features code-intelligence --lib` (51 passed),
  `cargo check --workspace --locked` and `cargo clippy --workspace --locked -- -D warnings` pass.
- Local implementation commits: `54db04be`, `0ff1642f`, `035c6c8b`, `1664e4b4`, `543ec076`,
  `ec38d73b`, `2b711e55`, `c36e24f7`, `7cfa1602`, `04035105`, `8eee6355`, `9d46a4d1`,
  `ce4c017d`, `26a25f2a`, `6e5e3f67`, `844a2859`, `73f41527`.

### Plugin compatibility inventory — 2026-09-05

`talos-plugin` currently persists `talos-plugin.toml` with a top-level `plugin` metadata object,
plus `skills`, `tools`, and `hooks`. The WASM carrier, confined handler paths, fuel/epoch timeout,
bounded output, no-host-import boundary, explicit local package selection, and default registry
exclusion are existing compatibility invariants. Manifest parsing does not grant permissions.
Future Bundle work must add an envelope around this legacy payload; it must not silently rename or
remove persisted `plugin` fields. Bundle identity/source/digest may be additive provenance and must
not imply permission. Package selection/install and executable runtime remain separate operations.
Evidence: existing `talos-plugin` manifest/WASM tests and CLI explicit-plugin tests pass in the
workspace run; no plugin behavior was changed by I246.

## Variance And Residuals

- CAP-001-A/B/C and BUNDLE/TEXT/LANG/DIST/BROWSER children remain separately governed.
- Real Desktop capability binding remains under Desktop owners after this preparation.
- Continuity guard: #467/I246 is a prerequisite seam for CAP-001/#466, not a substitute for the
  parent architecture. The parent and all later provider, bundle, language, distribution and
  browser outcomes remain open for separately governed child iterations.
