# 057: Windows PowerShell Process Boundary

## Status

Proposed for TOOL-023-C/I170 on 2026-07-31; pending independent security and maintainer review.

## Context

`BashTool` invoked `sh -c` on every platform even though stock Windows does not provide `sh`.
This made the registered write-capable shell tool unusable on Windows and left 19 workspace tests
failing before I169/PR #68 could satisfy its mandatory G13 gate. TOOL-023-C already selected a
Windows-native PowerShell path and required a decision for interpreter identity, hardening, and
permission behavior.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
| --- | --- | --- | --- |
| Shell execution remains permission-gated Execute behavior | Hard | `AGENTS.md` #4; ADR-012 | No |
| External process failures degrade to a returned error | Hard | `AGENTS.md` #9 | No |
| Unix child hardening and approved `unsafe` remain unchanged | Hard | ADR-007 | No |
| Windows has no POSIX rlimit contract | Hard platform fact | ADR-007 | No |
| PowerShell is the Windows-native shell identity | Soft product decision | TOOL-023-C | Yes, via superseding ADR |
| Arbitrary POSIX commands can be translated safely | Assumption | Initial cross-platform tests | Rejected |

## Decision

- Unix/non-Windows keeps the public tool name `bash`, `sh -c`, and ADR-007 pre-exec hardening.
- Windows exposes the tool as `powershell` and invokes
  `powershell.exe -NoLogo -NoProfile -NonInteractive -Command <command>`.
- The child command removes every name returned by
  `ProcessHardening::dangerous_env_var_names()` through `Command::env_remove`. This is child-local,
  safe Rust configuration; it does not mutate the Talos parent environment and adds no `unsafe`.
- Windows makes no rlimit or Job Object claim. Timeout enforcement kills the direct PowerShell
  child at one absolute deadline and returns bounded output plus `[timeout]`.
- Permission nature/family remain Execute/Shell. Resource keys and descriptions use the actual
  presented platform tool name. ADR-012 still treats control syntax and unknown PowerShell cmdlets
  as complex exact resources; the change does not grant reusable trust to PowerShell grammar.
- The prompt names both possible shell identities and tells the model to use the exact presented
  tool definition. Talos does not translate POSIX source text to PowerShell.
- Workspace-relative file/search display paths use `/` on all hosts so protocol-visible output and
  tests do not depend on the host separator.

## Security Review

The I170 security review is recorded in
[`I170-WINDOWS-SHELL-SECURITY-REVIEW-2026-07-31.md`](../reference/I170-WINDOWS-SHELL-SECURITY-REVIEW-2026-07-31.md).
Merge requires its automated and manual evidence; passing unit tests alone is insufficient.

## Rejected Alternatives

- **Use cmd.exe.** It has weaker scripting/error behavior and contradicts the accepted TOOL-023-C
  baseline.
- **Locate Git-for-Windows `sh.exe`.** That preserves an undeclared host dependency and continues
  presenting POSIX semantics on a Windows-native product surface.
- **Translate commands.** Shell translation is not semantics-preserving and could alter permission
  meaning.
- **Apply parent-side environment mutation.** It risks concurrent process-wide mutation; child
  `env_remove` provides the required isolation.
- **Add Job Objects now.** That is a separate process-hardening design and is not required to close
  the current spawn/timeout defect.

## Residual And Reversal Triggers

- Direct-child kill may not terminate every descendant created by a shell command. Record process
  tree supervision under TOOL-024; do not claim it here.
- Revisit if PowerShell 7 becomes a bundled/required runtime, Windows Job Object hardening is
  approved, or permission policy gains a reviewed PowerShell parser.
