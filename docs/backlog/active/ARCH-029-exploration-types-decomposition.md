# ARCH-029: Exploration Types Decomposition

**Status**: Complete
**Priority**: P2
**Created**: 2026-06-28
**Parent**: Two-month architecture optimization M9
**Selected iteration**: I074

## Problem

`crates/talos-exploration/src/lib.rs` mixed public domain entity definitions with SQLite schema
migration, FTS search, citation validation, and store operations. The domain types are imported by
ingestion and CLI-facing workflows, while the store implementation changes for different reasons.

Keeping entity definitions in the store root makes review harder and increases the chance that
schema/store edits accidentally churn public type definitions.

## Scope

- Move exploration domain entities and `EdgeType` display formatting into a focused module.
- Preserve existing `talos_exploration::*` public imports through re-export.
- Preserve SQLite schema, CRUD behavior, FTS search, citation validation, and ingestion behavior.

## Out of Scope

- SQLite schema changes.
- FTS query or snippet behavior changes.
- Citation validation changes.
- Ingestion/chunking behavior changes.
- Test-suite partitioning.

## Acceptance Criteria

- [x] `talos-exploration/src/lib.rs` loses domain entity definition responsibility.
- [x] Public exploration types remain re-exported from crate root.
- [x] `cargo test -p talos-exploration --quiet` passes.
- [x] Workspace quality gates pass.
- [x] Governance validation passes.

## Duplicate-Logic Disposition

Domain entity definitions remain centralized in `types.rs`. No duplicated type aliases, wrapper
types, or conversion helpers were introduced.

## Execution Notes

- Added `crates/talos-exploration/src/types.rs`.
- Updated `crates/talos-exploration/src/lib.rs` to `pub use types::*`.
- `crates/talos-exploration/src/lib.rs` dropped from 1070 to 958 lines.
- `crates/talos-exploration/src/types.rs` is 110 lines.
- Validation passed: `cargo test -p talos-exploration --quiet`, `cargo fmt --all -- --check`,
  `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`,
  `cargo test --workspace --quiet`, `scripts/validate_project_governance.sh .`, and
  `git diff --check`.
