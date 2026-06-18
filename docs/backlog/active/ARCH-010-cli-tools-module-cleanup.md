# ARCH-010: CLI and Tools Module Cleanup

**Status**: Planned
**Priority**: P3
**Source**: Architecture decay audit 2026-06-18 (post-ARCH-005)
**Depends on**: ARCH-005 partial complete

## Problem

Two files remain larger than expected after the ARCH-005 decomposition pass:

1. `crates/talos-cli/src/main.rs` at 1250 lines — 8 modules were extracted (registry.rs,
   provider_setup.rs, session_setup.rs, tui_bridge.rs, etc.) but the main mode-runner functions
   (`run_tui_mode`, `run_print_mode`, `run_inline_mode`, `run_interactive_mode`, ~130 lines each)
   and `build_hook_registry` still sit in main.rs.

2. `crates/talos-tools/src/file_tools.rs` at 1308 lines — this is a new file created during
   ARCH-005 that absorbed ReadTool, WriteTool, EditTool, DeleteTool, and LsTool plus shared
   helpers. It is now the largest single-responsibility file in the tools crate.

## Scope

### Slice 1: talos-cli/src/main.rs (P3)

Extract mode runner functions:

- `mode_runners.rs` — `run_tui_mode`, `run_print_mode`, `run_inline_mode`,
  `run_interactive_mode`, `run_rpc_mode`, `run_mcp_server`.
- Keep `main()`, `Cli` struct, `Mode` enum, and `build_hook_registry` in `main.rs`.
- Target: `main.rs` ≤400 lines.

### Slice 2: talos-tools/src/file_tools.rs (P3, lower priority)

Split into:

- `read_tool.rs` — ReadTool (complex: offset/limit, pagination, line numbering).
- `write_edit_tools.rs` — WriteTool, EditTool (write-side).
- `file_tools.rs` — DeleteTool, LsTool, shared helpers (`is_skip_dir`, `resolve_workspace_path`,
  `is_binary_file`).

Target: no single file >600 lines.

## Acceptance Criteria

- [ ] `talos-cli/src/main.rs` ≤400 lines (Slice 1).
- [ ] `talos-tools/src/file_tools.rs` ≤600 lines (Slice 2).
- [ ] No behavior changes in either slice.
- [ ] `cargo check --workspace`, `cargo test --workspace`, and
      `cargo clippy --workspace -- -D warnings` pass after each slice.
- [ ] All public tool types remain importable at the same paths via `pub use`.

## Verification Notes

Baseline sizes (2026-06-18 audit):
- `talos-cli/src/main.rs`: 1250 lines
- `talos-tools/src/file_tools.rs`: 1308 lines

Slice 1 can proceed independently. Slice 2 depends on confirming no downstream imports rely
on direct paths within `file_tools.rs`.
