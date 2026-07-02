# TOOL-016: Direct Exec Tool

| Field | Value |
|-------|-------|
| Story ID | TOOL-016 |
| Priority | P2 |
| Status | In Progress — I077/T114 permission policy in Review |
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

- [ ] `exec` runs a single command with argv arguments.
- [ ] Timeout terminates the subprocess.
- [ ] stdout/stderr are bounded.
- [ ] Permission checks run before execution.
- [ ] Tests cover success, non-zero exit, timeout, permission denial, and argument safety.

## Required Reads

- [GitHub Issue #16](https://github.com/wjhuang88/talos/issues/16)
- `docs/backlog/active/TOOL-005-bash-streaming-output.md`
- `docs/backlog/active/PERM-001-guardian-exec-policy.md`
- `crates/talos-tools/src/bash_tool.rs`
- `crates/talos-permission/src/`
