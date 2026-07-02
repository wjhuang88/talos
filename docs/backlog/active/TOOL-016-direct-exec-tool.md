# TOOL-016: Direct Exec Tool

| Field | Value |
|-------|-------|
| Story ID | TOOL-016 |
| Priority | P2 |
| Status | Complete — delivered in I077/T115; closed in T116 |
| Source | [GitHub Issue #16](https://github.com/wjhuang88/talos/issues/16) |
| Relates To | TOOL-005, TOOL-006, PERM-001 |

## Requirement

Add an `exec` tool for launching a single subprocess with argv-style arguments, avoiding shell
parsing for common command execution.

## Scope

- Add a structured input shape: command, args, optional cwd, optional env, optional timeout.
- Use `tokio::process::Command` directly, not shell `-c`.
- Return structured exit code, stdout, stderr, and duration.
- Route through the existing permission pipeline.

## Security Gate

This is a process-execution tool. Before implementation, define the permission policy for command
allowlists/defaults and environment handling. If that policy changes existing approval semantics,
record or update an ADR.

T114 policy update (2026-07-02): `docs/reference/EXEC-TOOL-PERMISSION-POLICY-2026-07-02.md`
accepts only a narrow T115 implementation. `exec` must be an argv-only single-process tool, default
to `Ask`, expose an `Execute` command facet and optional `Read` cwd facet, deny sensitive env names
before spawn, avoid echoing env values, clamp timeout, bound stdout/stderr, and never invoke
`sh -c`.

## Non-Goals

- No shell pipelines, glob expansion, redirection, or background jobs.
- No write-capable bypass around existing filesystem/process permissions.

## Acceptance Criteria

- [x] `exec` runs a single command with argv arguments.
- [x] Timeout terminates the subprocess.
- [x] stdout/stderr are bounded.
- [x] Permission checks run before execution.
- [x] Tests cover success, non-zero exit, timeout, permission denial, and argument safety.

T115 implementation update (2026-07-02): `ExecTool` landed in `talos-tools` and is registered in
CLI print, TUI, and MCP registries. It uses `tokio::process::Command` directly, exposes command/cwd
permission facets, denies sensitive env names before spawn, redacts env values in output metadata,
keeps shell metacharacters as literal argv data, bounds stdout/stderr, and kills timed-out children.

T116 closeout (2026-07-02): full workspace validation passed and Issue #16 may be closed after the
closeout commit is pushed.

## Required Reads

- [GitHub Issue #16](https://github.com/wjhuang88/talos/issues/16)
- `docs/backlog/active/TOOL-005-bash-streaming-output.md`
- `docs/backlog/active/PERM-001-guardian-exec-policy.md`
- `crates/talos-tools/src/bash_tool.rs`
- `crates/talos-permission/src/`
