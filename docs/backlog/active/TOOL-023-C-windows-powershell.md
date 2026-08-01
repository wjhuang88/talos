# TOOL-023-C: Windows-Native Shell (PowerShell)

**Status**: In Progress — implemented in Draft PR #126; exact-head cross-platform validation and security acceptance pending (2026-08-01)
**Priority**: P2
**Parent Epic**: TOOL-023
**Type**: Product / State Story
**Depends on**: TOOL-023-A
**Selected Iteration**: I170

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | GPT-5.6 Thinking / talos recovery session 2026-08-01 |
| Work Slice | TOOL-023-C within I170: one Windows PowerShell process/tool boundary, child environment scrub and portable presentation while preserving Unix behavior and current contribution ownership. |
| Claimed At | 2026-08-01 |
| Source Issue | #119 (I170 dependency recovery context) |
| Governance Claim PR | #122 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | PR #122 merged the I170 claim after exact-head governance, collaboration, remote-owner and CI validation; its recorded I170 slice explicitly includes Windows PowerShell and portability corrections. |
| Implementation PR | #126 |
| Last Updated | 2026-08-01 |
| Handoff / Release Condition | Release only through the I170 claim owner after PR #126 merges and exact Completion Commit plus security/maintainer acceptance are recorded. |

## Problem

The authoritative shell tool historically invoked `sh -c` and exposed `bash` on every platform. Stock Windows does not provide that process contract, so the registered write-capable shell was unusable and its tests encoded Unix-only assumptions.

## Goal / Value

Windows users receive one native, permission-gated `powershell` tool while Unix users retain the existing `bash` / `sh -c` behavior. Tool definitions, prompts, permissions, MCP, output handling and product inventories agree on the platform identity.

## Scope

- Windows invokes `powershell.exe -NoLogo -NoProfile -NonInteractive -Command <command>` and presents the tool as `powershell`.
- Unix/non-Windows keeps `bash`, `sh -c` and ADR-007 pre-exec hardening.
- Child command construction removes the canonical dangerous inherited environment names without mutating the Talos parent environment.
- One authoritative `talos-tools:shell` contribution remains; no second CLI/registry registration path is introduced.
- Permission resource prefixes/descriptions, prompts, Agent output compression, MCP listing and exact product inventories use the actual platform name.
- Workspace-relative file/search output uses `/`; Windows long listing uses one type character plus nine conservative permission characters.
- CRLF, Unix-only symlink/hardening fixtures and temporary-directory assumptions are corrected without deleting or weakening tests.

## Exclusions

- No `exec` behavior change.
- No POSIX-to-PowerShell translation, `cmd.exe` fallback, shell-selection config, PowerShell parser, Windows rlimit, Job Object or descendant process-tree claim.
- No parent environment mutation.
- No I169 steering implementation.

## Decision Links And Constraints

- ADR-007: Unix hardening and approved unsafe remain unchanged.
- ADR-012: complex/unknown shell commands remain exact permission resources.
- ADR-053: current Tool Contribution and outer composition ownership remain authoritative.
- ADR-057: platform process, identity, environment and timeout boundary.

## State / Status Owners

- Story status and acceptance: this file.
- Execution/evidence: `docs/iterations/I170-windows-workspace-validation-unblocker.md`.
- Process decision: `docs/decisions/057-windows-powershell-process-boundary.md`.
- Security review: `docs/reference/I170-WINDOWS-SHELL-SECURITY-REVIEW-2026-08-01.md`.
- Implementation: Draft PR #126.
- Historical evidence only: recovery Draft PR #121 at `e1da5dd893418a3f6e3737ec900aabe9967b1dda`.

## Acceptance For Behavior

- Windows presents exactly one `powershell` shell contribution and executes native PowerShell commands with stdout, stderr, exit code, current directory and bounded timeout behavior.
- Unix presents exactly one `bash` shell contribution and retains `sh -c` behavior.
- Dangerous environment variables are absent from the child and unchanged in the parent.
- Permissions and prompts use the presented platform tool name without broadening high-risk trust.
- Windows path and long-list projections are deterministic and do not invent Unix metadata.

## Acceptance For Technical Work

- [x] Platform process construction and tool identity implemented.
- [x] Child-local environment removal implemented without new unsafe.
- [x] Current contribution inventory remains singular and platform-aware.
- [x] Permission, prompt, Agent and MCP surfaces accept the platform name.
- [x] Portable path/metadata and fixture corrections implemented.
- [x] README EN/zh-CN platform behavior updated.
- [x] ADR-057 and current security review recorded as Proposed/Review.
- [x] A full Windows Rust CI job covers format/check/Clippy/focused tests/full workspace/governance/mock smoke.
- [ ] Exact final Head passes Windows and macOS/Unix CI.
- [ ] Rebuilt Windows walkthrough evidence and independent security/maintainer acceptance are recorded.

## Residual Destination

PowerShell grammar-aware reusable permissions, PowerShell 7 selection and descendant process-tree supervision require separate decisions and owners.
