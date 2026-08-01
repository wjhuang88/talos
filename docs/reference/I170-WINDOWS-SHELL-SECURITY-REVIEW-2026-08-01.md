# I170 Windows Shell Security Review — 2026-08-01

## Scope

Review Draft PR #126 against I170 and proposed ADR-057 for:

- Windows PowerShell process construction and tool identity;
- Unix compatibility and existing ADR-007 hardening;
- dangerous inherited environment removal;
- absolute timeout and output draining;
- contribution/permission/prompt/MCP identity consistency;
- portable path and long-list projection;
- cross-platform fixture corrections.

Recovery PR #121 and historical head `e1da5dd893418a3f6e3737ec900aabe9967b1dda` are evidence sources only. All findings and validation below must be re-established on the final exact head of PR #126.

## Threat Model

| Threat | Boundary | Required control |
|---|---|---|
| Model or user invokes arbitrary shell mutation | Permission pipeline | Execute/Shell facets, exact resources for complex/high-risk commands, no registration bypass |
| Windows receives POSIX shell assumptions | Prompt/tool definition | Present only `powershell`; no command translation |
| Child inherits injection/loader/runtime variables | Process spawn | Child-local removal of the canonical dangerous-name set |
| Parent environment is changed concurrently | Talos process | Never call parent `set_var`/`remove_var`; use `Command::env_remove` |
| Continuous output evades timeout | Async process loop | One pinned absolute deadline independent of stdout/stderr activity |
| Timeout leaves direct child running | Process cleanup | Kill and wait for the direct child before returning |
| Timeout claim is overstated to descendants | Documentation/API | Explicitly limit guarantee to direct child; no Job Object/process-tree claim |
| Duplicate shell contributions create inconsistent policy | Composition | One `talos-tools:shell` contribution; platform identity comes from the tool |
| Windows path separators leak into protocol/test behavior | Presentation | Workspace-relative `/` projection |
| Windows metadata invents Unix semantics | `ls --long` | Conservative type/readonly projection only |
| Platform test repair hides defects | Tests/governance | Target-gate only genuinely Unix-specific fixtures; no broad ignore/delete/weakened assertions |

## Design Review

### Process construction

Windows uses `powershell.exe` with `-NoLogo -NoProfile -NonInteractive -Command`. `-NoProfile` prevents user profile scripts from silently altering execution, while `-NonInteractive` avoids prompts that can hang the tool. The command remains a native PowerShell string; Talos performs no lossy POSIX translation.

Unix retains `sh -c` and the existing ADR-007 `pre_exec` rlimits. No new unsafe site is introduced.

### Environment

`ProcessHardening::dangerous_env_var_names()` is applied through `Command::env_remove` before spawn. This configuration is scoped to the child command and cannot race by changing the parent environment. Unix retains the reviewed post-fork cleanup as defense in depth.

Required adversarial tests:

- command builder records every dangerous name as removed;
- a child cannot observe representative dangerous variables;
- the parent still observes its original value after child completion;
- concurrent shell creation does not require global environment mutation.

### Timeout

The deadline future is created once before the `select!` loop and polled by mutable reference. Output branches only append bounded lines and update pipe-open state; they cannot recreate the timer.

The one deadline remains active until both the direct child and its stdout/stderr pipes finish. At expiry, Talos kills and waits for the direct child when still running, preserves output already received, emits `[timeout]`, and returns without waiting for pipe EOF. This prevents descendants that inherited the handles from extending the operation timeout while making no descendant-termination claim.

Required adversarial tests:

- a command emitting output repeatedly still times out near the configured absolute duration;
- partial output is retained;
- closed stdout/stderr do not spin or reset the deadline;
- configured values are clamped to the existing 1–600 second range;
- direct child is reaped before return.

### Permission and composition

`BashTool` retains `Execute` / `Shell`. The platform name is used in contribution inventory, permission resource prefixes/descriptions, prompts, MCP listing and output compression. No PowerShell-specific auto-allow parser is introduced; complex/unknown commands remain exact.

Required checks:

- exactly one shell contribution exists in each profile;
- Windows inventory is `powershell` plus `exec`, Unix inventory is `bash` plus `exec`;
- no CLI registry constructs an additional shell tool;
- denied shell calls remain denied through MCP and product modes;
- high-risk commands do not acquire reusable permission templates merely because of the platform rename; drive/provider paths and `$`/`~` expansion fall back to exact resources.

### Portable output and fixtures

Path normalization changes only protocol-visible separators after a path has already been resolved relative to the authorized root. It does not change authorization or filesystem traversal.

Windows long-list metadata deliberately avoids Unix owner/link/executable claims. CRLF and temp-directory fixes normalize representation without broadening access or weakening expected values. Symlink and `pre_exec` tests are retained on Unix and excluded only where the underlying API/contract is Unix-specific.

## Current Findings

| ID | Severity | Finding | Status |
|---|---|---|---|
| I170-S1 | High | A second Windows shell registration would bypass the authoritative contribution inventory. | Automated evidence complete: one authoritative contribution and platform-sorted product inventories passed on Windows and macOS. |
| I170-S2 | High | A timeout created inside the output loop can be extended indefinitely. | Automated evidence complete: continuous output and descendant-held-pipe regressions passed under one deadline on Windows and Unix/macOS. |
| I170-S3 | High | Parent-side environment mutation would race concurrent process execution. | Implementation and command-builder evidence complete: all canonical names are removed child-locally with no parent mutation; independent reviewer confirmation remains required. |
| I170-S4 | Medium | Direct-child kill does not guarantee descendant termination. | Accepted residual only if PR/docs make no stronger claim; maintainer/security acceptance pending. |
| I170-S5 | Medium | PowerShell command classification reuses conservative shell heuristics rather than a PowerShell parser. | Automated permission evidence complete: unknown/control, drive/provider and `$`/`~` expansion remain exact resources. |
| I170-S6 | Medium | Platform output normalization could conceal authorization path changes. | Automated evidence complete: normalization occurs after authorized resolution; document fixtures stay inside explicit workspaces while external paths remain rejected. |
| I170-S7 | Medium | Windows CI previously validated only installer fixtures. | Automated evidence complete on `1ca536159c34437719e4f776db2e02e4afc8510d` / run `30686493121`: full Windows Rust workspace, governance and rebuilt CLI smoke passed. |

## Required Evidence Before Merge

- exact final Head SHA recorded in I170 and PR #126;
- macOS release preflight passes format/check/Clippy/full tests/governance;
- Windows job passes format/check/Clippy/full locked workspace tests;
- focused Windows PowerShell process, env and absolute-timeout tests pass;
- focused Unix shell/hardening regressions pass;
- MCP and permission tests prove actual platform identity and denial routing;
- `git diff --check`, collaboration claim and remote Issue/Owner reconciliation pass;
- rebuilt Windows direct PowerShell walkthrough records command, stdout/stderr, exit code, working directory and timeout behavior;
- rebuilt Unix CLI mock smoke passes;
- no unresolved review thread or unreviewed high-severity finding remains.

## Merge Recommendation

**Automated gate passed; independent approval pending.** Implementation Head `1ca536159c34437719e4f776db2e02e4afc8510d` passed CI run `30686493121`. PR #126 remains Draft until the Review synchronization Head repeats the gates and an independent process/security plus maintainer review accepts I170-S3/I170-S4 and ADR-057. Historical recovery PR #121 is provenance only.
