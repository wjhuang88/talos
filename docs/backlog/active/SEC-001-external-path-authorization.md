# SEC-001: External-Path File Authorization Gap

| Field | Value |
|-------|-------|
| Story ID | SEC-001 |
| Priority | P1 — security boundary |
| Status | Complete (2026-07-16) — external paths require Ask; workspace reads still Allow; Deny wins; symlink escape rejected |
| Depends On | PERM-004, PERM-005 |
| Relates To | ADR-038 (workspace trust), ADR-040 (command access evidence) |
| Origin | Architecture review v2 of the four-month reliability plan |

## Finding

File tools (`crates/talos-tools/src/file_tools.rs`) unconditionally restrict paths to the
workspace root. The permission engine can produce Allow/Ask/Deny, but authorization results
are not passed as constrained execution capability to file tools.

External-path reads may be judged Allow by the permission engine
(`crates/talos-permission/src/lib.rs`), only to be hard-rejected by the file tool.

## Correct Semantics (requires new iteration + security review)

- Workspace-internal: current behavior unchanged.
- Workspace-external: Ask with "once" or "always this exact file/directory" scope.
- Deny always wins.
- Non-interactive modes: safe-deny.
- Authorization carries normalized path, operation type, and scope — not `allow_external: bool`.
- Symlink/TOCTOU, path traversal, and delete-root protection continue to apply.
- bash/exec sandbox permissions must not be broadened by file-tool authorization.

## Why This Needs a Separate Iteration

This is a permission boundary change. Per AGENTS.md Hard Constraint #4, all write-capable
tools must go through the permission pipeline. External-path access is a new authorization
surface that requires security review and cannot be silently patched into existing iterations.
