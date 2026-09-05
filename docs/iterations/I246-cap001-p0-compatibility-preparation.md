# Iteration I246: CAP-001-P0 Compatibility Preparation

> Document status: Active / Claimed (proposed; ineffective until governance merge)
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
| Work Slice | I246 / CAP-001-P0 narrowed characterization only: current-code inventory, compatibility matrix, dependency boundary and downstream handoff. No code, Cargo, dependency, persistence or runtime behavior changes. |
| Claimed At | 2026-09-05 |
| Source Issue | #467 |
| Governance Claim PR | Pending |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer explicitly authorized the narrowed scope on 2026-09-05. |
| Implementation PR | Not started |
| Last Updated | 2026-09-05 |
| Handoff / Release Condition | This narrowed claim remains ineffective until its governance PR merges; implementation is limited to documentation/characterization artifacts and must not change code/Cargo/behavior. |

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

## Execution Order

This iteration is intentionally scheduled after I207 and I208 in the long task. It must not start until the steering slices have reached terminal dispositions and its own architecture/overlap gates are satisfied.

## Completion Evidence

- Completion Commit: {already-existing implementation SHA}
- Status-only documentation commits must not cite themselves.

## Variance And Residuals

- CAP-001-A/B/C and BUNDLE/TEXT/LANG/DIST/BROWSER children remain separately governed.
- Real Desktop capability binding remains under Desktop owners after this preparation.
