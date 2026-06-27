# ARCH-026: Agent Configuration Decomposition

**Status**: Complete
**Priority**: P2
**Created**: 2026-06-27
**Parent**: Two-month architecture optimization M5/M6
**Selected iteration**: I071

## Problem

`crates/talos-agent/src/lib.rs` still owns constructor and runtime configuration methods in the
same file as the turn loop. It also repeats prompt-builder update boilerplate across setters:
take the builder, call a `with_*` method, assign it back, and sometimes invalidate the stable
prefix cache.

This is both size debt and duplicate-logic debt. Future prompt/cache changes are easy to apply
inconsistently if every setter repeats the mutation pattern.

## Scope

- Move `Agent` constructors and runtime configuration setters into a focused internal module.
- Extract a shared prompt-builder mutation helper for setter methods.
- Preserve public `Agent` API, prompt output, cache invalidation semantics, memory provider
  behavior, and security constructor behavior.

## Out of Scope

- Turn-loop behavior changes.
- Tool execution behavior changes.
- Provider request or prompt schema changes.
- Permission, sandbox, or hook semantics.

## Acceptance Criteria

- [x] `talos-agent/src/lib.rs` loses constructor/configuration responsibility.
- [x] Repeated prompt-builder setter boilerplate is centralized.
- [x] `cargo test -p talos-agent --quiet` passes.
- [x] Workspace quality gates pass.
- [x] Governance validation passes.

## Duplicate-Logic Disposition

The repeated `std::mem::take(&mut self.prompt_builder).with_*` pattern is in scope and must be
replaced by a local helper in this slice.

## Execution Notes

- Added `crates/talos-agent/src/configuration.rs` for constructors and configuration setters.
- Centralized prompt-builder mutation plus optional stable-prefix invalidation in
  `Agent::update_prompt_builder`.
- `crates/talos-agent/src/lib.rs` dropped from 914 to 655 lines.
- Turn-loop, tool execution, provider request, permission, sandbox, and hook behavior were not
  changed.
- Validation passed: `cargo test -p talos-agent --quiet`, `cargo fmt --all -- --check`,
  `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`,
  `cargo test --workspace --quiet`, `scripts/validate_project_governance.sh .`, and
  `git diff --check`.
