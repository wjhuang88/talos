# TOOL-023: Cross-Platform Shell Execution and Reliable Timeout (Epic)

**Status**: Refinement (2026-07-24)
**Priority**: P1 (bug fix child) / P2 (platform + config children)
**Source**: User request 2026-07-24 ("Windows 下 bash 工具替换为 Windows 命令行工具；命令行工具加可配置超时，默认 300s")
**Type**: Epic

## Outcome

The shell/command execution tools (`bash`, `exec`) run reliably on all supported
platforms and cannot hang indefinitely:

1. A command that keeps producing output but never exits is killed at the timeout
   deadline (today the `bash` tool's timeout is silently defeated by such commands).
2. On Windows, the `bash` tool invokes a Windows-native interpreter (PowerShell)
   instead of the currently hardcoded `sh -c`, which does not exist on a stock
   Windows host.
3. The execution timeout is configurable per tool call and via a global config
   default, with a single documented default value.

## Origin And Redefinition

The request was phrased as "add a timeout". Investigation showed a per-call
`timeout_secs` parameter **already exists** on both `bash` and `exec` (clamped to
`[1, 600]`), with a 120s default. The user's actual pain — "shell calls sometimes
hang for an unbounded time" — traces to a **defect**, not a missing feature:

`crates/talos-tools/src/bash_tool.rs` places `tokio::time::sleep(timeout)` **inside**
a `tokio::select!` **loop** that also polls `stdout`/`stderr` line readers. Every
time a line arrives, the `select!` returns, the loop re-iterates, and the sleep
timer is dropped and restarted from zero. A subprocess that emits a line more often
than the timeout interval therefore **never times out**. `exec` does not have this
bug: it reads pipes in detached `tokio::spawn` tasks and runs a single
`select! { child.wait(), sleep(timeout) }`, so its timeout is a hard, single-shot
deadline. See child `TOOL-023-A` for the fix.

Because the true fix is a bug remediation with different acceptance than "add a
feature", the request is split into three children with the fix prioritized first.

## Children

| ID | Title | Priority | Depends on |
|---|---|---|---|
| `TOOL-023-A` | Fix bash timeout defeated by continuous output | P1 | none |
| `TOOL-023-B` | Configurable execution timeout with 300s default | P2 | TOOL-023-A |
| `TOOL-023-C` | Windows-native shell (PowerShell) for the bash tool | P2 | TOOL-023-A |

`TOOL-023-A` must land first: raising or configuring the default timeout (B) is
meaningless while the bash timer can be reset indefinitely, and the Windows shell
work (C) shares the same execution path that A repairs.

## Boundary And Exclusions

- Windows resource hardening (CPU/memory rlimits via Job Objects) is **excluded**.
  Per ADR-007, Windows child processes receive environment-variable sanitization
  only; no `RLIMIT_*` equivalents are added. PowerShell subprocesses keep that same
  boundary (env scrub only).
- `exec` shell replacement is **excluded**: `exec` is argv-only (`Command::new(argv[0])`)
  with no shell involvement and is already platform-agnostic. Only its timeout
  default/config participates (TOOL-023-B).
- Changing the maximum clamp (currently 600s) is **deferred** and not decided in
  this Epic; the default becomes 300s while the max clamp is revisited only if a
  concrete long-task need appears (recorded as a residual, not implemented here).
- No new interactive shell, no shell-selection config (`pwsh` vs `cmd`): Windows
  uses PowerShell, decided by the requester.

## Major Risks

- **A**: A naive fix (wrapping the whole loop in `tokio::time::timeout`) could drop
  buffered output produced before the deadline. The fix must preserve the
  drain-after-kill behavior the current code already attempts.
- **C**: PowerShell argument quoting differs from `sh -c`; command strings the model
  emits assuming POSIX shell semantics may behave differently. The tool name change
  (`bash` → `shell`/`powershell` on Windows) must flow into prompt/tool definitions
  so the model adapts its command syntax.

## Completion Condition

All three children reach Complete with `Completion Commit:` evidence, `bash` and
`exec` cannot hang on continuous-output commands, Windows uses PowerShell, and the
timeout default is 300s configurable per-call and globally. User-facing docs
(README tool section, `config.reference.toml`) reflect the new default and config
key.

## Required Reads

- `crates/talos-tools/src/bash_tool.rs` (defect site: `run_command`, the `select!` loop)
- `crates/talos-tools/src/exec_tool.rs` (correct timeout reference implementation)
- `crates/talos-config/src/types.rs` (`Config` struct — new `[tools]` table attaches here)
- `docs/decisions/007-process-hardening-unsafe.md` (Windows = env scrub only)
- `docs/decisions/012-exec-policy-dsl-boundary.md` (shell command classification)
- `docs/backlog/active/TOOL-005-bash-streaming-output.md` (bash tool evolution)
- `docs/backlog/active/TOOL-016-direct-exec-tool.md` (exec tool origin)
