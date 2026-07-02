# TUI-016: Slash Command Panel Smart Auto-Execute

| Field | Value |
|-------|-------|
| Story ID | TUI-016 |
| Priority | P3 |
| Status | Review — implemented in I078/T120 |
| Source | [GitHub Issue #7](https://github.com/wjhuang88/talos/issues/7) |
| Relates To | TUI-010, CMD-001, CMD-002 |

## Requirement

Optimize slash command panel interaction: parameterless commands execute directly on Enter in the
panel; parameter commands fill the input area for completion. Add visual hints distinguishing the
two modes.

## Scope

- Extend `CommandDefinition` with execution mode classification (DirectExecution / RequireInput)
  based on existing `arg_hint` metadata.
- Modify slash panel Enter behavior: DirectExecution commands trigger immediately; RequireInput
  commands fill the composer.
- Add UI hints in the panel showing parameter requirements.
- Maintain backward compatibility with manual command input.

I078/T120 activation (2026-07-02): selected as the first Month 3 packet after I077 closeout. The
implementation must preserve manual command typing and only change slash panel Enter behavior.

## Non-Goals

- No change to command execution semantics.
- No new commands — only interaction optimization for existing commands.

## Acceptance Criteria

- [x] Parameterless commands (/help, /status, /quit, /mcp, /new) execute on Enter in panel.
- [x] Parameter commands (/skills, /resume, /export) fill input area on Enter.
- [x] Panel visually distinguishes direct-execute vs input-required commands.
- [x] Manual typing path unchanged.
- [x] Unit tests for execution mode classification and Enter branching.

## Implementation Notes

- `CommandDefinition::execution_mode()` derives the picker behavior from `arg_hint`.
- Slash panel `Enter` sends DirectExecution commands through the existing message dispatch path.
- Slash panel `Enter` fills the composer for RequireInput commands; `Tab` always completes without
  execution.

## Verification

- `cargo test -p talos-tui slash_menu`
- `cargo test -p talos-conversation complete_slash_command`
- `cargo test -p talos-tui`
- `cargo test -p talos-conversation`
- `cargo clippy -p talos-tui -p talos-conversation -- -D warnings`
- `cargo check --workspace`
- `scripts/validate_project_governance.sh .`

## Required Reads

- `crates/talos-conversation/src/command_registry.rs`
- `crates/talos-tui/src/state.rs` (slash menu handling)
- `docs/backlog/active/TUI-010-slash-command-menu.md`
- `docs/backlog/active/CMD-001-interactive-command-runtime-contract.md`
