# TOOL-019: Bash Exit-Code Classification

| Field | Value |
|---|---|
| Story ID | TOOL-019 |
| Priority | P0 |
| Status | Complete (SSP110) |
| Source | [GitHub Issue #23](https://github.com/wjhuang88/talos/issues/23) |
| Depends On | `TOOL-017`, `PERM-002`, `RUNTIME-002` |

## Problem

The bash tool currently treats every non-zero exit status as `is_error`. For tools such as `rg`,
`grep`, `diff`, `test`, and `cargo fmt --check`, exit code 1 can be an expected negative result
rather than an execution error. Mislabeling these results as tool errors pollutes model context and
can amplify stuck-turn behavior.

## Acceptance

- Classify expected non-zero statuses for known low-risk commands without hiding true crashes,
  timeouts, permission denials, or missing executables.
- Preserve exact exit status and stderr/stdout in the visible tool result.
- Keep default behavior conservative for unknown commands.
- Add tests for `rg`/`grep` no match, `diff` differences, `cargo fmt --check` differences, timeout,
  and command-not-found.
- Record the classification policy in the bash permission/policy documentation or fixture comments.

## Non-Goals

- No permission approval relaxation.
- No shell parser expansion beyond what classification needs.

## Required Reads

- `crates/talos-tools/src/bash_tool.rs`
- `crates/talos-tools/src/bash_permission_policy.toml`
- `docs/backlog/active/TOOL-017-exec-multi-parallel-pipe.md`
- `docs/backlog/active/PERM-002-operation-scoped-permissions.md`

