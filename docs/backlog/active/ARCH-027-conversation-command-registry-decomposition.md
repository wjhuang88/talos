# ARCH-027: Conversation Command Registry Decomposition

**Status**: Complete
**Priority**: P2
**Created**: 2026-06-27
**Parent**: Two-month architecture optimization M7
**Selected iteration**: I072

## Problem

`crates/talos-conversation/src/engine.rs` mixed conversation state/event reduction with static
slash-command metadata, registry lookup, availability filtering, and completion logic. That made
the engine root larger than necessary and kept command metadata changes next to runtime mutation
logic.

The command registry is already a shared contract used by help, completion, and the TUI menu. It
should live in a focused module so command metadata remains centralized without expanding the
conversation event reducer.

## Scope

- Move command origin, definition, registry, availability predicate, registry singleton, and
  command-registry accessor into a focused module.
- Move the diagnostic `/mock-request` passthrough constant with the command registry boundary.
- Preserve public exports from `talos-conversation`.
- Preserve slash command help, completion, TUI menu, and passthrough behavior.

## Out of Scope

- Slash command dispatch behavior changes.
- New commands, aliases, or availability rules.
- TUI menu rendering changes.
- CLI command behavior changes.

## Acceptance Criteria

- [x] `talos-conversation/src/engine.rs` loses command registry responsibility.
- [x] Command metadata and completion logic are centralized in a focused module.
- [x] Public command registry API remains exported from `talos-conversation`.
- [x] `cargo test -p talos-conversation --quiet` passes.
- [x] Workspace quality gates pass.
- [x] Governance validation passes.

## Duplicate-Logic Disposition

Command metadata, available-name collection, and completion logic remain centralized in the new
`command_registry.rs` module. No repeated slash-command registry logic was introduced.

## Execution Notes

- Added `crates/talos-conversation/src/command_registry.rs`.
- Updated `crates/talos-conversation/src/lib.rs` to re-export command registry API from the new
  module while keeping `ConversationEngine` exported from `engine`.
- `crates/talos-conversation/src/engine.rs` dropped from 960 to 739 lines.
- `crates/talos-conversation/src/command_registry.rs` is 225 lines.
- Validation passed: `cargo test -p talos-conversation --quiet`, `cargo fmt --all -- --check`,
  `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`,
  `cargo test --workspace --quiet`, `scripts/validate_project_governance.sh .`, and
  `git diff --check`.
