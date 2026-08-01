# 057: Windows PowerShell Process Boundary

## Status

Accepted on 2026-08-01 for TOOL-023-C and I170.

Acceptance evidence:

- exact implementation Head `8cfe8edb2dbda581244f583fb809591391a54298`;
- exact-head CI run `30705366763` (`CI` run 718), all macOS/Windows/governance jobs successful;
- Windows walkthrough artifact `8820174164`;
- merged PR #126 at `592254d73a98166df48da0139a02df67e9cd2cd6`;
- final review closed the computed-expression permission blocker;
- the repository owner explicitly authorized readiness and merge after reviewing the final evidence.

The accepted residual is direct-child-only timeout cleanup. This decision does not authorize or claim full descendant process-tree supervision.

## Context

The authoritative shell contribution historically constructed `BashTool`, invoked `sh -c` on every platform and exposed the name `bash`. Stock Windows does not provide that process contract. The recovered I170 evidence preserved a Windows-native PowerShell design, but recovery PR #121 is archival and does not establish current-main architecture or validation.

Current architecture moves built-in tool construction into `talos-tools` contributions with outer composition roots responsible for selection and permission wrapping. The Windows fix therefore changes the single contributed tool's platform identity and process construction rather than adding a second registry path.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
|---|---|---|---|
| Shell execution remains permission-gated Execute/Shell behavior | Hard | `AGENTS.md`; ADR-012 | No |
| Exactly one authoritative shell contribution exists per product profile | Hard | ADR-053; current contribution architecture | No |
| Unix `bash` name, `sh -c`, rlimits and approved `pre_exec` unsafe remain unchanged | Hard | ADR-007; compatibility | No |
| External process failures degrade to bounded returned errors | Hard | repository error policy | No |
| Windows has no POSIX rlimit contract | Platform fact | ADR-007 | No |
| Child environment cleanup must not mutate the parent process | Hard | concurrency and credential safety | No |
| One configured timeout is an absolute deadline, not an inactivity timer | Hard | TOOL-023-A | No |
| PowerShell is the Windows-native presented shell | Product decision | TOOL-023-C | Yes, through a superseding ADR |
| Arbitrary POSIX source can be translated safely | Assumption | historical cross-platform tests | Rejected |

## Decision

### Platform process and tool identity

- Unix/non-Windows presents `bash` and invokes `sh -c <command>`.
- Windows presents `powershell` and invokes:

```text
powershell.exe -NoLogo -NoProfile -NonInteractive -Command <command>
```

- Talos does not translate command text between POSIX shell and PowerShell. Prompts instruct the model to use the exact shell tool present in the active tool definitions.
- The existing `BashTool` Rust type remains an internal compatibility name; public tool identity comes from `AgentTool::name()`.

### Contribution and permission boundary

- `talos-tools::bash_tool_contribution` remains the one authoritative shell escape-hatch contribution.
- No Windows-specific duplicate contribution or CLI registry construction is added.
- Outer print/TUI/MCP composition continues to select and permission-wrap the contribution.
- Tool nature/family remain `Execute` / `Shell`.
- Permission resource prefixes and descriptions use the actual platform tool name. Unknown or complex commands remain exact resources.
- On Windows, reusable cwd-scoped parameter templates accept only inert ASCII alphanumeric path/option tokens plus `-`, `_`, `.`, `/`, and `=`.
- PowerShell grouping, call operators, script blocks, arrays, variables, member/index expressions, quoting, provider/drive syntax, home expansion, wildcard syntax, backslashes, splatting, comma lists and stop-parsing forms fail closed to exact resources until a reviewed PowerShell lexer/parser exists.
- Parent and absolute paths remain ineligible for cwd-scoped reusable grants after token validation.

### Child hardening

- Before spawning, the command removes every name returned by `ProcessHardening::dangerous_env_var_names()` with child-local `Command::env_remove` on every platform.
- Unix additionally retains the existing ADR-007 `pre_exec` cleanup and rlimits. The duplication is deliberate defense in depth and preserves the reviewed post-fork boundary.
- Windows adds no rlimit, Job Object or parent-environment mutation and introduces no new `unsafe`.

### Timeout and output

- A single Tokio sleep future is created and pinned before stdout/stderr/wait arbitration.
- Reading output cannot recreate or extend the deadline.
- The same deadline covers direct-child completion and stdout/stderr closure.
- Talos completes normally only after the direct child and both pipes finish; otherwise it kills and waits for the direct child when still running, preserves output already received, appends `[timeout]`, and returns without waiting for descendant-held pipe EOF.
- This is a direct-child guarantee only. It does not claim termination of every descendant process created by a shell command.

### Portable presentation and fixtures

- Workspace-relative protocol-visible file/search paths use `/` on every host.
- Windows long-list output starts with one file-type character and nine conservative permission characters. It does not invent Unix uid, gid, nlink or executable bits.
- Unix-only symlink and `pre_exec` fixtures are target-gated rather than deleted or weakened.
- CRLF comparisons and temporary-directory fixtures normalize only the platform representation while preserving semantic assertions.

## Security Review

Accepted review evidence is recorded in:

- `docs/reference/I170-WINDOWS-SHELL-SECURITY-REVIEW-2026-08-01.md`.

The original high-severity reusable-template finding was corrected with the explicit Windows inert-token allowlist and exact-resource regressions. No unresolved high-severity finding remains in the I170 scope.

## Rejected Alternatives

- **Use `cmd.exe`.** It provides a different scripting/error contract and does not match TOOL-023-C.
- **Search for Git-for-Windows `sh.exe`.** This preserves an undeclared optional dependency and presents POSIX semantics on a native Windows product surface.
- **Register both `bash` and `powershell` on Windows.** This creates ambiguous duplicate authority and inconsistent permission/prompt behavior.
- **Translate POSIX commands.** Translation is not semantics-preserving and can change permission meaning.
- **Maintain an expanding PowerShell syntax denylist for reusable templates.** PowerShell has too many expression and invocation forms for a denylist to establish a conservative cwd boundary; an explicit inert-token allowlist fails closed.
- **Mutate the parent environment before spawn.** Process-wide mutation races with concurrent tool execution and can leak or remove credentials from unrelated work.
- **Reset timeout whenever output arrives.** That changes an absolute operation budget into an unbounded inactivity timer.
- **Claim Job Object/process-tree supervision now.** That requires a separate process lifecycle and security design.

## Validation Evidence

- Unix focused tests prove `bash`, `sh -c`, existing hardening and timeout behavior remain intact.
- Windows focused tests prove the `powershell` identity, native commands, child env removal, stdout/stderr/exit status, working directory and continuous-output deadline.
- Windows permission regressions prove ordinary safe relative path/options may retain reviewed templates while `cat (Join-Path (Get-Item ..).FullName secret.txt)` and adjacent grouping, call, array, variable and member-expression forms receive exact resources.
- Current contribution inventories prove exactly one platform shell contribution and no duplicate registry path.
- Permission, MCP, prompt and Agent output surfaces use the actual platform identity.
- Full locked workspace format/check/Clippy/tests passed on Windows and macOS/Unix.
- Governance, collaboration-claim, diff and release-preflight gates passed on the exact Head.
- The rebuilt Windows CLI and direct PowerShell walkthrough were recorded in artifact `8820174164`.

## Residual And Reversal Triggers

- Descendant process-tree supervision remains a separate TOOL-024/process-runtime concern.
- Revisit if PowerShell 7 becomes bundled or required, Windows Job Object hardening is accepted, the permission pipeline gains a reviewed PowerShell parser, or supported Windows images no longer provide Windows PowerShell.
- A future shell-selection setting requires a new decision because it changes presented tool identity, permission resources and reproducibility.
