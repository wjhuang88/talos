# I032: Tools Module Cleanup

**Status**: Complete (2026-06-19)
**Target Window**: Week 3 of next month plan
**Depends On**: I031 complete preferred

## Outcome

Finish the remaining ARCH-010 tools cleanup by decomposing `crates/talos-tools/src/file_tools.rs`.
This keeps the tools crate from growing another large mixed-responsibility file before additional
tool capabilities such as SCHED-001 are implemented.

## Selected Stories

- [x] #ARCH-010-C: Split `talos-tools/src/file_tools.rs` into read, write/edit, and shared file modules
- [x] #ARCH-010-D: Preserve all public tool imports through `talos-tools` re-exports
- [x] #ARCH-010-E: Update architecture docs and file-size inventory

## Acceptance Criteria

- [x] No `talos-tools` source file in the touched area exceeds 600 lines.
- [x] Existing file tools preserve behavior and import paths.
- [x] `ReadTool`, `WriteTool`, `EditTool`, `DeleteTool`, and `LsTool` remain importable from
      the same public paths.
- [x] `cargo test -p talos-tools` passes.
- [x] `cargo clippy -p talos-tools -- -D warnings` passes.
- [x] `cargo test --workspace` passes before close.

## Risks

- Read/write/edit behavior is permission-sensitive; split by moving code, not by changing logic.
- Shared helpers such as path resolution and binary detection must remain single-purpose and avoid
  reintroducing a central catch-all module.
- SCHED-001 should start only after this cleanup or through its existing I028 plan.

## Verification Log

2026-06-19:

- Split file tool implementations into focused child modules:
  - `file_tools/read_tool.rs`
  - `file_tools/write_edit_tools.rs`
  - `file_tools/delete_tool.rs`
  - `file_tools/ls_tool.rs`
  - `file_tools/tests/delete.rs`
- Kept `file_tools.rs` as the shared helper and stable re-export surface.
- File-size inventory after split:
  - `file_tools.rs`: 108 lines
  - `read_tool.rs`: 145 lines
  - `write_edit_tools.rs`: 144 lines
  - `delete_tool.rs`: 86 lines
  - `ls_tool.rs`: 253 lines
  - `tests.rs`: 531 lines
  - `tests/delete.rs`: 86 lines
- Verification:
  - `cargo test -p talos-tools` passed.
  - `cargo clippy -p talos-tools -- -D warnings` passed.
  - `cargo test --workspace` passed.
  - `cargo clippy --workspace -- -D warnings` passed.
