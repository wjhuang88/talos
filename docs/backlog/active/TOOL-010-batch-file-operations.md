# TOOL-010: Batch File Operations

**Status**: Refinement
**Priority**: P3
**Source**: User request 2026-06-26
**Iteration**: None yet

## Problem

The built-in `read`, `write`, and `edit` tools accept a single file path per invocation.
When an agent needs to inspect or modify multiple files (common during refactors, multi-file
reviews, or project-wide changes), it must issue N sequential tool calls. Each round-trip
adds latency, consumes a turn, and increases prompt/context churn.

## Scope

Extend file read/write/edit tools to accept an array of files so the agent can batch
multiple file operations in a single tool call.

### Required behavior

1. **Batch read**: `read` tool accepts either a single `path` (backward compatible) or a
   `paths` array. Each file is read independently; results are returned as a structured
   list keyed by path. Files that fail (not found, permission denied) are reported
   individually without aborting the entire batch.

2. **Batch write**: `write` tool accepts either a single `{path, content}` or a `files`
   array of `{path, content}` objects. Each write goes through the existing permission
   pipeline independently. Partial failure is reported per-file.

3. **Batch edit**: `edit` tool accepts either a single edit or an `edits` array of
   `{path, old_string, new_string}` objects across multiple files.

4. **Permission semantics**: each file in the batch is authorized independently through
   the existing `PermissionEngine`. A deny on one file does not block the others unless
   a configurable `fail_fast` option is set. Default: continue on denial (collect all
   results, report per-file outcome).

5. **Backward compatibility**: existing single-file call syntax must continue to work
   unchanged. The array form is an additive extension.

### Non-goals

- No glob-based recursive directory writes (use `glob` + batch `read` instead).
- No transactional/atomic multi-file writes (each file is independent).
- No new dependencies.
- No change to the permission model itself — only batch routing through the existing path.

## Acceptance

- Given a batch read call with 3 valid paths,
  When the tool executes,
  Then all 3 file contents are returned in a single `ToolResult` keyed by path.

- Given a batch read call where 1 path is denied by permission rules,
  When the tool executes,
  Then the 2 allowed files return content and the denied file reports a permission error,
  all in the same result.

- Given a single-file read call (legacy syntax),
  When the tool executes,
  Then the result shape matches the existing single-file format.

- Given a batch write with 2 files,
  When the tool executes through the permission pipeline,
  Then each write is independently authorized and applied.

## Dependencies

- TOOL-002 (tool calling architecture) — schema validation must handle array inputs.
- PERM-002 (operation-scoped permissions) — batch permission evaluation path.

## Decision links and constraints

- ADR-010 (self-contained capabilities)
- AGENTS.md: all write-capable tools gated by permissions

## State/status owners

- Backlog: `docs/backlog/active/TOOL-010-batch-file-operations.md`
- Board: add to Next/Later when prioritized

## User-facing documentation

- `README.md` Built-In Capabilities section: document batch syntax
- Tool schema description visible to the model must explain both single and array forms

## Required Reads

- `docs/backlog/active/TOOL-002-tool-calling-remediation.md`
- `docs/backlog/active/TOOL-003-posix-tool-set.md`
- `crates/talos-tools/src/file_tools/`
- `crates/talos-permission/src/lib.rs`
