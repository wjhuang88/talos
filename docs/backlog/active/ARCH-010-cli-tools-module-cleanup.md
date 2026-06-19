# ARCH-010: CLI and Tools Module Cleanup

**Status**: Partial (Slice 1 complete 2026-06-19; Slice 2 planned)
**Priority**: P3
**Source**: Architecture decay audit 2026-06-18 (post-ARCH-005)
**Depends on**: ARCH-005 partial complete

## Problem

Two files remain larger than expected after the ARCH-005 decomposition pass:

1. `crates/talos-cli/src/main.rs` was at 1250 lines — 8 modules were extracted (registry.rs,
   provider_setup.rs, session_setup.rs, tui_bridge.rs, etc.) but the main mode-runner functions
   (`run_tui_mode`, `run_print_mode`, `run_inline_mode`, `run_interactive_mode`, ~130 lines each)
   and `build_hook_registry` still sit in main.rs.

2. `crates/talos-tools/src/file_tools.rs` remains at 1308 lines — this is a new file created during
   ARCH-005 that absorbed ReadTool, WriteTool, EditTool, DeleteTool, and LsTool plus shared
   helpers. It is now the largest single-responsibility file in the tools crate.

## Scope

### Slice 1: talos-cli/src/main.rs (P3)

Extract mode runner functions:

- `mode_runners.rs` — `run_tui_mode`, `run_print_mode`, `run_inline_mode`,
  `run_interactive_mode`, `run_rpc_mode`, `run_mcp_server`.
- Keep `main()`, `Cli` struct, `Mode` enum, and `build_hook_registry` in `main.rs`.
- Target: `main.rs` ≤400 lines.

Status: complete (2026-06-19). `main.rs` now owns argument parsing, mode dispatch, and hook
registry construction; `mode_runners.rs` owns mode execution.

### Slice 2: talos-tools/src/file_tools.rs (P3, lower priority)

Split into:

- `read_tool.rs` — ReadTool (complex: offset/limit, pagination, line numbering).
- `write_edit_tools.rs` — WriteTool, EditTool (write-side).
- `file_tools.rs` — DeleteTool, LsTool, shared helpers (`is_skip_dir`, `resolve_workspace_path`,
  `is_binary_file`).

Target: no single file >600 lines.

## Acceptance Criteria

- [x] `talos-cli/src/main.rs` ≤400 lines (Slice 1).
- [ ] `talos-tools/src/file_tools.rs` ≤600 lines (Slice 2).
- [ ] No behavior changes in either slice.
- [x] Slice 1 targeted checks pass: `cargo check -p talos-cli`, `cargo test -p talos-cli`,
      `cargo clippy -p talos-cli -- -D warnings`.
- [ ] Full post-Slice-2 checks: `cargo check --workspace`, `cargo test --workspace`, and
      `cargo clippy --workspace -- -D warnings`.
- [ ] All public tool types remain importable at the same paths via `pub use`.

## Verification Notes

Baseline sizes (2026-06-18 audit):
- `talos-cli/src/main.rs`: 1250 lines
- `talos-tools/src/file_tools.rs`: 1308 lines

Slice 1 completion evidence (2026-06-19):

- `talos-cli/src/main.rs`: 241 lines.
- `talos-cli/src/mode_runners.rs`: new module containing mode execution functions.
- `talos-cli/src/tests.rs`: existing main-module tests moved out of the dispatch surface.
- Verification:
  - `cargo check -p talos-cli` passed.
  - `cargo test -p talos-cli` passed: 25 unit tests + hooks/MCP/RPC e2e tests.
  - `cargo clippy -p talos-cli -- -D warnings` passed.

Slice 2 remains planned and depends on confirming no downstream imports rely on direct paths within
`file_tools.rs`.
