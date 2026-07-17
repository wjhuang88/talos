# SEC-001: External-Path File Authorization Gap

| Field | Value |
|-------|-------|
| Story ID | SEC-001 |
| Priority | P1 — security boundary |
| Status | Complete (2026-07-17, I140) — exact external-path approval accepted after security and locked validation |
| Depends On | PERM-004, PERM-005 |
| Relates To | ADR-038 (workspace trust), ADR-040 (command access evidence) |
| Origin | Architecture review v2 of the four-month reliability plan |

## Finding

File tools (`crates/talos-tools/src/file_tools.rs`) unconditionally restrict paths to the
workspace root. The permission engine can produce Allow/Ask/Deny, but authorization results
are not passed as constrained execution capability to file tools.

External-path reads may be judged Allow by the permission engine
(`crates/talos-permission/src/lib.rs`), only to be hard-rejected by the file tool.

## Correct Semantics

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

## Delivery Status

**Status**: Complete (I140, 2026-07-17)

I140 implements the required choice through ADR-047. Permission-aware Runtime, CLI, and TUI
composition roots now pass a tool/nature/normalized-path authorization to file tools only after
Allow or explicit approval. Direct raw execution, Deny, missing headless approval, path/operation
reuse, and changed symlink targets fail closed. The security analysis is recorded in
`docs/reference/I140-SEC001-SECURITY-REVIEW-2026-07-17.md`; the full locked workspace ladder,
release preflight, governance validation, and diff check pass.
