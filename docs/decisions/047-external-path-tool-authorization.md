# ADR-047: External-Path Tool Authorization

**Status**: Accepted (2026-07-17)

## Context

Talos previously made two contradictory decisions for a file request outside the active workspace:
the permission engine returned `Ask`, but the file tool unconditionally rejected the same path.
Consequently, an explicit user approval could not authorize the requested operation.

The maintainer requires a real choice without turning workspace trust or an approval into a broad
filesystem bypass.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
| --- | --- | --- | --- |
| Write-capable tools use the permission pipeline | Hard | AGENTS.md #4 | No |
| Deny rules override approval/trust | Hard | ADR-038 and permission contract | No |
| Headless unresolved Ask fails closed | Hard | runtime safety contract | No |
| Existing `AgentTool::execute` callers remain compatible | Hard | AGENTS.md #6 | No |
| An approved external path can execute | Product requirement | Maintainer correction, 2026-07-17 | Only by explicit authorization |

## Decision

Add an optional, structured execution-authorization channel to `AgentTool`.

- `AgentTool::execute_authorized` is additive and defaults to `execute`, so existing tools and
  embedders retain their behavior.
- A permission-aware composition root creates `ToolExecutionAuthorization` only after an `Allow`
  decision or an explicit `ApproveOnce`/`AlwaysApprove` response.
- The authorization binds the exact tool name, operation nature, resource kind, normalized path,
  and approval lifetime. A boolean `allow_external` flag is rejected.
- File tools continue to allow workspace-contained paths normally. An external path requires a
  matching authorization; direct `execute` remains fail-closed.
- Existing paths and the nearest existing ancestor of new write targets are canonicalized. The
  file tool normalizes again immediately before execution, so a changed symlink target cannot
  reuse a stale authorization.
- CLI/TUI/runtime composition roots pass the same typed capability. No file tool prompts directly.
- Deny rules, delete-root protection, traversal checks, bash/exec rules, sandbox behavior, tool
  approval event ordering, and persistence formats are unchanged.

No new dependency or crate edge is introduced: the neutral authorization types live in
`talos-core`; `talos-permission` produces them; `talos-tools` consumes them; CLI and
`talos-runtime` coordinate the boundary.

## Compatibility

This is an additive pre-1.0 public trait method with a default implementation and additive public
types. Existing implementers and callers need no source change. Hosts that want external file
access must use a permission-aware composition root; invoking a raw file tool directly remains
workspace-confined.

## Reversal Trigger

Revisit if Talos adopts an OS capability-handle API that can make path authority unforgeable across
process boundaries, or if a platform demonstrates that canonical-path revalidation cannot safely
represent the requested filesystem operation. Until then, uncertainty fails closed.
