# ARCH-010: CLI and Tools Module Cleanup

**Status**: Complete (2026-06-19)
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
- `delete_tool.rs` — DeleteTool.
- `ls_tool.rs` — LsTool and directory listing formatting.
- `file_tools.rs` — shared helpers (`is_skip_dir`, `resolve_workspace_path`, `is_binary_file`)
  and stable public re-exports.
- `file_tools/tests/` — oversized test groups split out of the module wrapper.

Target: no single file >600 lines.

Status: complete (2026-06-19). `file_tools.rs` is now a thin shared module and public re-export
surface; the concrete read/write/edit/delete/ls implementations live in focused child modules.

## Acceptance Criteria

- [x] `talos-cli/src/main.rs` ≤400 lines (Slice 1).
- [x] `talos-tools/src/file_tools.rs` ≤600 lines (Slice 2).
- [x] No behavior changes in either slice.
- [x] Slice 1 targeted checks pass: `cargo check -p talos-cli`, `cargo test -p talos-cli`,
      `cargo clippy -p talos-cli -- -D warnings`.
- [x] Full post-Slice-2 checks: `cargo check --workspace`, `cargo test --workspace`, and
      `cargo clippy --workspace -- -D warnings`.
- [x] All public tool types remain importable at the same paths via `pub use`.

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

Slice 2 completion evidence (2026-06-19):

- `talos-tools/src/file_tools.rs`: 108 lines.
- `talos-tools/src/file_tools/read_tool.rs`: 145 lines.
- `talos-tools/src/file_tools/write_edit_tools.rs`: 144 lines.
- `talos-tools/src/file_tools/ls_tool.rs`: 253 lines.
- `talos-tools/src/file_tools/delete_tool.rs`: 86 lines.
- `talos-tools/src/file_tools/tests.rs`: 531 lines; delete tests split to
  `file_tools/tests/delete.rs` (86 lines).
- Verification:
  - `cargo test -p talos-tools` passed: 81 unit tests + 3 integration hardening tests.
  - `cargo clippy -p talos-tools -- -D warnings` passed.
  - `cargo test --workspace` passed.
  - `cargo clippy --workspace -- -D warnings` passed.
