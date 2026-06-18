# I032: Tools Module Cleanup

**Status**: Planned
**Target Window**: Week 3 of next month plan
**Depends On**: I031 complete preferred

## Outcome

Finish the remaining ARCH-010 tools cleanup by decomposing `crates/talos-tools/src/file_tools.rs`.
This keeps the tools crate from growing another large mixed-responsibility file before additional
tool capabilities such as SCHED-001 are implemented.

## Selected Stories

- [ ] #ARCH-010-C: Split `talos-tools/src/file_tools.rs` into read, write/edit, and shared file modules
- [ ] #ARCH-010-D: Preserve all public tool imports through `talos-tools` re-exports
- [ ] #ARCH-010-E: Update architecture docs and file-size inventory

## Acceptance Criteria

- [ ] No `talos-tools` source file in the touched area exceeds 600 lines.
- [ ] Existing file tools preserve behavior and import paths.
- [ ] `ReadTool`, `WriteTool`, `EditTool`, `DeleteTool`, and `LsTool` remain importable from
      the same public paths.
- [ ] `cargo test -p talos-tools` passes.
- [ ] `cargo clippy -p talos-tools -- -D warnings` passes.
- [ ] `cargo test --workspace` passes before close.

## Risks

- Read/write/edit behavior is permission-sensitive; split by moving code, not by changing logic.
- Shared helpers such as path resolution and binary detection must remain single-purpose and avoid
  reintroducing a central catch-all module.
- SCHED-001 should start only after this cleanup or through its existing I028 plan.

## Verification Log

(to be filled as stories land)
