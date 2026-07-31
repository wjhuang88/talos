# TOOL-023-C: Windows-Native Shell (PowerShell) for the Bash Tool

**Status**: Review — implementation and Windows gates pass; remote review pending (2026-07-31)
**Priority**: P2
**Parent Epic**: TOOL-023
**Type**: Product/State Story
**Depends on**: TOOL-023-A (shares the repaired execution path)

## Problem

`crates/talos-tools/src/bash_tool.rs::run_command` hardcodes `Command::new("sh")`
with `-c` on all platforms. Stock Windows has no `sh`, so the `bash` tool fails to
spawn there. The tool must invoke a Windows-native interpreter.

## Goal / Value

On Windows, the shell tool runs commands through PowerShell and presents itself to
the model under a Windows-appropriate name, so the model emits PowerShell-compatible
commands and the tool works on Windows hosts.

## Scope

- Add a `#[cfg(windows)]` execution path selecting PowerShell
  (`powershell -NoProfile -Command <cmd>`, or `pwsh` if a decision to prefer it is
  recorded). Keep `sh -c` on `#[cfg(unix)]`.
- On Windows, expose the tool to the model under a Windows-appropriate name
  (`shell` or `powershell`, requester chose to rename) so prompt/tool definitions
  signal PowerShell syntax. Unix keeps `bash`. Ensure the name flows into the tool
  definition/prompt and permission descriptions consistently.
- Process hardening on Windows: environment-variable sanitization only (reuse
  `ProcessHardening::dangerous_env_var_names`), no `RLIMIT_*` / Job Object resource
  limits — consistent with ADR-007.
- New ADR: record the Windows shell substitution decision (interpreter choice,
  tool-name-per-platform, env-scrub-only hardening, quoting caveats), analogous to
  how ADR-007 scoped Unix hardening.

## Exclusions

- No `exec` change (argv-only, already cross-platform).
- No Windows resource limits (Job Objects) — explicitly out per ADR-007.
- No shell-selection config (`pwsh` vs `cmd`): PowerShell only.
- No attempt to translate model-emitted POSIX command strings into PowerShell; the
  tool-name change is the mechanism by which the model adapts syntax.

## Decision Links And Constraints

- ADR-007: Windows children get env sanitization only; no rlimits.
- ADR-012: shell command classification for the permission pipeline. Verify the
  classifier's assumptions (pipes/redirects/globs meaning) still hold under
  PowerShell, or record the difference as a residual/ADR note.
- ADR-057: selects `powershell.exe`, the Windows `powershell` tool identity, child-local env
  removal, exact permission fallback, and the no-translation/no-rlimit boundary.
- Permission pipeline: the tool remains `ToolNature::Execute` / `ToolFamily::Shell`;
  a per-platform tool-name change must not weaken permission routing.

## Uncertainty And Validation Path

Windows CI is not part of the current release workflow (macOS runner). Real
Windows/PowerShell execution is therefore a manual gate. Automated coverage on
non-Windows hosts is limited to compile-time `#[cfg(windows)]` correctness and
name/definition selection unit tests. Record the manual Windows walkthrough as the
Ready → Complete gate.

## State/Status Owners

This story file; parent `TOOL-023`; Board mirror; new ADR.

## User-Facing Documentation

- `README.md` / `README.zh-CN.md`: document that on Windows the shell tool runs
  PowerShell and is named `shell`/`powershell`.
- Public site capabilities page if it enumerates the `bash` tool.

## Required Reads

- `crates/talos-tools/src/bash_tool.rs` (`run_command`, `name()`, cfg blocks)
- `crates/talos-sandbox/src/hardening.rs` (env scrub source; Windows boundary)
- `docs/decisions/007-process-hardening-unsafe.md`
- `docs/decisions/012-exec-policy-dsl-boundary.md`

## Acceptance for behavior

- Given a Windows host
  When the model invokes the shell tool with a PowerShell command
  Then it runs via PowerShell, returns output and exit status, and the child receives
  environment sanitization.

- Given a Unix host
  When the shell tool is invoked
  Then behavior is unchanged (`sh -c`, tool name `bash`) — no regression.

## Acceptance for technical work

- [x] `#[cfg(windows)]` path selects PowerShell; `#[cfg(unix)]` unchanged; both compile.
- [x] Tool name is `bash` on Unix and the chosen Windows name on Windows, verified by
      a unit test on the tool definition/name selection.
- [x] Env sanitization applies on Windows; no rlimit/Job Object code added.
- [x] New ADR recorded and linked from this story and the Epic.
- [x] README (EN/zh-CN) and site capabilities updated.
- [x] Windows PowerShell execution is exercised by the native `talos-tools` test suite and the
      full Windows release preflight.
- [x] `cargo clippy --workspace --locked -- -D warnings` clean on the default target.

Implementation commits: `5dc7854d`, `77323b91`, `ed07a769`, `25002064`, `248d3217`. Completion
Commit remains pending while ADR-057 and the independent PR await review.
