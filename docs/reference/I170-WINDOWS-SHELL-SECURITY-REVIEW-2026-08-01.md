# I170 Windows Shell Security Review — 2026-08-01

## Review Status

Accepted for the I170 scope on 2026-08-01.

- Exact implementation Head: `8cfe8edb2dbda581244f583fb809591391a54298`.
- Exact-head CI: `30705366763` (`CI` run 718), successful.
- Windows walkthrough artifact: `8820174164`.
- Completion merge: PR #126 at `592254d73a98166df48da0139a02df67e9cd2cd6`.
- The prior computed-expression permission blocker is closed.
- The repository owner explicitly accepted the final review outcome and authorized readiness and merge.
- Accepted residual: timeout cleanup is guaranteed for the direct shell child, not the full descendant process tree.

This record does not claim a formal GitHub approval from a separate account. It records the reviewed exact-head evidence and the maintainer's explicit acceptance and merge authorization.

## Scope

Review PR #126 against I170 and ADR-057 for:

- Windows PowerShell process construction and tool identity;
- Unix compatibility and existing ADR-007 hardening;
- dangerous inherited environment removal;
- absolute timeout and output draining;
- contribution/permission/prompt/MCP identity consistency;
- conservative Windows reusable-template classification;
- portable path and long-list projection;
- cross-platform fixture corrections.

Recovery PR #121 and historical head `e1da5dd893418a3f6e3737ec900aabe9967b1dda` are provenance only. Acceptance is based on the final exact implementation Head and its own CI/artifact evidence.

## Threat Model

| Threat | Boundary | Accepted control |
|---|---|---|
| Model or user invokes arbitrary shell mutation | Permission pipeline | Execute/Shell facets, exact resources for complex/high-risk commands, no registration bypass |
| PowerShell computes a path outside cwd while receiving a reusable cwd grant | Permission resource classification | Explicit inert-token allowlist on Windows; grouping/call/array/variable/member/provider forms fail closed to exact resources |
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

The final Windows spawn-level regression exercises the production `BashTool::execute` path, proves representative dangerous values are absent from the child, proves the parent retains its values, serializes process-global test mutation and restores original state through an unwind-safe guard.

### Timeout

The deadline future is created once before the `select!` loop and polled by mutable reference. Output branches append bounded output and update pipe state; they cannot recreate the timer.

The one deadline remains active until both the direct child and its stdout/stderr pipes finish. At expiry, Talos kills and waits for the direct child when still running, preserves output already received, emits `[timeout]`, and returns without waiting for pipe EOF. This prevents descendants that inherited handles from extending the operation timeout while making no descendant-termination claim.

Final regressions cover continuous output, retained partial output, descendant-held pipe handles and direct-child cleanup.

### Permission and composition

`BashTool` retains `Execute` / `Shell`. The platform name is used in contribution inventory, permission resource prefixes/descriptions, prompts, MCP listing and output compression. No PowerShell-specific auto-allow parser is introduced; complex or unknown commands remain exact.

Windows reusable parameter templates use an explicit token allowlist rather than an expanding syntax denylist. Only ASCII alphanumeric characters plus `-`, `_`, `.`, `/`, and `=` are eligible before the existing parent/absolute-path checks run.

The following forms therefore receive exact resources:

- ordinary grouping and nested command expressions;
- the call operator and script blocks;
- arrays, splatting and comma lists;
- variables, environment expansion and home expansion;
- member and index expressions;
- quoting, wildcards and backslashes;
- drive/provider and UNC-like syntax;
- stop-parsing and other unreviewed punctuation forms.

The original blocker command receives an exact resource:

```powershell
cat (Join-Path (Get-Item ..).FullName secret.txt)
```

Normal reviewed inert tokens such as `src/lib.rs`, `./src/lib.rs`, `file-name_1.2.txt`, `-p talos-tools`, and `--package=talos-tools` retain reusable-template behavior.

### Portable output and fixtures

Path normalization changes only protocol-visible separators after a path has already been resolved relative to the authorized root. It does not change authorization or filesystem traversal.

Windows long-list metadata deliberately avoids Unix owner/link/executable claims. CRLF and temp-directory fixes normalize representation without broadening access or weakening expected values. Symlink and `pre_exec` tests remain active on Unix and are excluded only where the API/contract is Unix-specific.

The `fetch_url` repair remains test-only: ambient-proxy bypass and bounded loopback server behavior are confined to test constructors/fixtures, while the production client keeps normal proxy behavior.

## Findings Closure

| ID | Severity | Finding | Final Status |
|---|---|---|---|
| I170-S1 | High | A second Windows shell registration would bypass the authoritative contribution inventory. | Closed — final inventories prove one authoritative platform shell contribution and no duplicate registry path. |
| I170-S2 | High | A timeout created inside the output loop can be extended indefinitely. | Closed — one pinned deadline passed continuous-output and descendant-held-pipe tests on Windows and Unix/macOS. |
| I170-S3 | High | Parent-side environment mutation would race concurrent process execution. | Closed — canonical names are removed child-locally; production-path regression proves parent preservation. |
| I170-S4 | Medium | Direct-child kill does not guarantee descendant termination. | Accepted residual — documentation and behavior make no stronger process-tree claim. |
| I170-S5 | High | PowerShell grouping and computed-path expressions could receive a reusable cwd-scoped template under the prior token denylist. | Closed — explicit inert-token allowlist plus adjacent-expression exact-resource regressions passed on the final Head. |
| I170-S6 | Medium | Platform output normalization could conceal authorization path changes. | Closed — normalization occurs after authorized resolution and external paths remain rejected. |
| I170-S7 | Medium | Windows CI previously validated only installer fixtures. | Closed — final CI runs the full Windows Rust workspace, focused security tests, governance, walkthrough, CLI smoke and installer fixture. |

No unresolved High finding remains in the accepted I170 scope.

## Exact-Head Evidence

Run `30705366763` completed successfully on Head `8cfe8edb2dbda581244f583fb809591391a54298`, including:

- macOS release preflight and full locked workspace validation;
- Windows format, check, Clippy, focused permission/process/timeout tests and full workspace tests;
- project governance and collaboration validators;
- remote Issue/Owner reconciliation;
- rebuilt Windows CLI mock smoke;
- Windows installer fixture.

Artifact `8820174164` is bound to the same Head and records:

- matching `exact_head` and checked-out Git Head;
- actual PowerShell command execution;
- stdout, stderr, working directory and exit code `7`;
- timeout partial output and `[timeout]` marker;
- bounded elapsed time;
- no completed post-timeout output;
- direct-child cleanup;
- the direct-child-only residual statement.

## Acceptance Recommendation

Accepted for I170, TOOL-023-A, TOOL-023-C and ADR-057.

The final implementation does not leave an unreviewed permission expansion, parent-environment mutation, timeout guarantee overstatement, production-network behavior change, Unix regression or current-main integration conflict within the reviewed scope.

## Residual Ownership

The following remain outside I170 and require separate decisions/owners:

- full descendant process-tree supervision and Windows Job Object lifecycle;
- PowerShell grammar-aware reusable-template parsing;
- PowerShell 7 selection or bundled runtime requirements;
- broader background/supervised process-job lifecycle work.
