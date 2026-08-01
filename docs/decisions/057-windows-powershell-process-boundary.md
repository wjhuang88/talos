# 057: Windows PowerShell Process Boundary

## Status

Proposed / Review for TOOL-023-C and I170 on 2026-08-01. Implementation is in Draft PR #126; acceptance requires exact-head Windows and Unix/macOS CI plus independent process/security review.

## Context

The authoritative shell contribution currently constructs `BashTool`, which historically invoked `sh -c` on every platform and exposed the name `bash`. Stock Windows does not provide that process contract. The recovered I170 evidence preserved a Windows-native PowerShell design, but the old recovery PR #121 is archival and cannot establish current-main architecture or validation.

Current main also moved built-in tool construction into `talos-tools` contributions with outer composition roots responsible for selection and permission wrapping. The Windows fix must therefore change the single contributed tool's platform identity and process construction, not add a second registry path.

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
- Permission resource prefixes and descriptions use the actual platform tool name. Unknown or complex commands remain exact resources; this decision does not grant reusable trust to PowerShell grammar.

### Child hardening

- Before spawning, the command removes every name returned by `ProcessHardening::dangerous_env_var_names()` with child-local `Command::env_remove` on every platform.
- Unix additionally retains the existing ADR-007 `pre_exec` cleanup and rlimits. The duplication is deliberate defense in depth and preserves the reviewed post-fork boundary.
- Windows adds no rlimit, Job Object or parent-environment mutation and introduces no new `unsafe`.

### Timeout and output

- A single Tokio sleep future is created and pinned before stdout/stderr/wait arbitration.
- Reading output cannot recreate or extend the deadline.
- At the deadline Talos kills and waits for the direct child, drains bounded remaining pipe output, appends `[timeout]`, and returns a tool error.
- This is a direct-child guarantee only. It does not claim termination of every descendant process created by a shell command.

### Portable presentation and fixtures

- Workspace-relative protocol-visible file/search paths use `/` on every host.
- Windows long-list output starts with one file-type character and nine conservative permission characters. It does not invent Unix uid, gid, nlink or executable bits.
- Unix-only symlink and `pre_exec` fixtures are target-gated rather than deleted or weakened.
- CRLF comparisons and temporary-directory fixtures normalize only the platform representation while preserving semantic assertions.

## Security Review

Current review evidence is recorded in:

- `docs/reference/I170-WINDOWS-SHELL-SECURITY-REVIEW-2026-08-01.md`

Merge requires the review's automated and manual gates. Historical PR #121 validation is provenance only and does not satisfy current exact-head requirements.

## Rejected Alternatives

- **Use `cmd.exe`.** It provides a different scripting/error contract and does not match TOOL-023-C.
- **Search for Git-for-Windows `sh.exe`.** This preserves an undeclared optional dependency and presents POSIX semantics on a native Windows product surface.
- **Register both `bash` and `powershell` on Windows.** This creates ambiguous duplicate authority and inconsistent permission/prompt behavior.
- **Translate POSIX commands.** Translation is not semantics-preserving and can change permission meaning.
- **Mutate the parent environment before spawn.** Process-wide mutation races with concurrent tool execution and can leak or remove credentials from unrelated work.
- **Reset timeout whenever output arrives.** That changes an absolute operation budget into an unbounded inactivity timer.
- **Claim Job Object/process-tree supervision now.** That requires a separate process lifecycle and security design.

## Validation Gate

Before acceptance:

- Unix focused tests prove `bash`, `sh -c`, existing hardening and timeout behavior remain intact.
- Windows focused tests prove the `powershell` identity, native commands, child env removal, stdout/stderr/exit status, working directory and continuous-output deadline.
- Current contribution inventories prove exactly one platform shell contribution and no duplicate registry path.
- Permission, MCP, prompt and Agent output surfaces use the actual platform identity.
- Full locked workspace format/check/Clippy/tests pass on Windows and macOS/Unix.
- Governance, collaboration-claim, diff and release-preflight gates pass on the exact Head.
- A rebuilt Windows CLI and direct PowerShell tool walkthrough are recorded.
- Independent security/maintainer review accepts the direct-child limitation and residuals.

## Residual And Reversal Triggers

- Descendant process-tree supervision remains a separate TOOL-024/process-runtime concern.
- Revisit if PowerShell 7 becomes bundled or required, Windows Job Object hardening is accepted, the permission pipeline gains a reviewed PowerShell parser, or supported Windows images no longer provide Windows PowerShell.
- A future shell-selection setting requires a new decision because it changes presented tool identity, permission resources and reproducibility.
