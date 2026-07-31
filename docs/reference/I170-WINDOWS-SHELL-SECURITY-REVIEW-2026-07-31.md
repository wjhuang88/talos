# I170 Windows Shell Security Review — 2026-07-31

## Scope

Review the TOOL-023-A/C changes that replace the non-functional Windows `sh -c` path with
PowerShell, enforce an absolute timeout, and normalize tool output. This review does not approve
Job Objects, an interactive shell, permission bypasses, or POSIX-to-PowerShell translation.

## Threat And Invariant Review

| Surface | Risk | Required invariant | Evidence |
| --- | --- | --- | --- |
| Interpreter selection | Hidden host fallback changes command meaning | Windows selects only `powershell.exe`; Unix selects `sh` | cfg code plus platform name/execution tests |
| Permission routing | Rename bypasses stored/evaluated policy | Nature remains Execute, family remains Shell, unknown/control syntax remains exact Ask | permission-profile tests and ADR-012 review |
| Environment inheritance | Loader injection reaches child | Every ADR-007 dangerous name is removed on the child command | `platform_command_removes_dangerous_environment_variables` |
| Parent integrity | Environment scrub mutates concurrent parent | Use only `Command::env_remove`; no parent `set_var/remove_var` | code inspection |
| Resource claims | Windows falsely claims Unix rlimits | No Windows rlimit/Job Object claim; Unix pre-exec block unchanged | cfg inspection |
| Timeout | Output resets timeout or child hangs | One pinned deadline is created before the read loop | timeout and partial-output tests |
| Output handling | Closed pipe spins or output is silently lost | Track stdout/stderr closure, kill/wait, then drain and return marker | focused tests and code inspection |
| Command translation | Rewriting changes authority or intent | No translation; prompt requests platform-native syntax | prompt/ADR review |
| Path projection | Host separators produce unstable protocol | Relative display projection normalizes only separators | ls/glob/grep tests |

## Findings

- No new dependency or `unsafe` was introduced.
- The existing Unix ADR-007 `pre_exec` hardening is unchanged.
- PowerShell commands remain arbitrary Execute operations behind the existing permission pipeline.
- The conservative Windows long-list projection reports file type, read-only-derived permissions,
  size, and path; it does not invent uid/gid/nlink or executable bits.
- Known residual: killing the direct shell does not prove all descendants terminate. TOOL-024 owns
  background/process-tree lifecycle design.

## Required Evidence Before Completion

- `cargo test -p talos-tools --locked`
- locked workspace format/check/Clippy/test
- rebuilt Windows execution of output, stderr, non-zero exit, working directory, and timeout paths
- governance validation
- CI result or an explicit repository-level conclusion when no workflow is attached
