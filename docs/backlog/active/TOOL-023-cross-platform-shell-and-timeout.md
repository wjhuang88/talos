# TOOL-023: Cross-Platform Shell Execution And Reliable Timeout

**Status**: Partial — TOOL-023-A/C Complete through I170; TOOL-023-B remains Ready and separately unimplemented (2026-08-01)
**Priority**: P1 bug fix / P2 platform and configuration follow-ups
**Source**: User request 2026-07-24
**Type**: Epic

## Outcome

Talos shell execution must be portable and bounded:

1. continuous output cannot reset or evade the configured timeout;
2. Windows uses a native PowerShell tool identity/process contract;
3. Unix retains `bash` / `sh -c` and existing hardening;
4. timeout default/configuration changes remain a separate child rather than being hidden inside the portability repair.

## Children

| ID | Title | Current State | Depends On |
|---|---|---|---|
| `TOOL-023-A` | Absolute shell timeout under continuous output | Complete — PR #126 / `592254d73a98166df48da0139a02df67e9cd2cd6` | None |
| `TOOL-023-B` | Configurable execution timeout with 300s default | Ready / not selected by I170 | TOOL-023-A |
| `TOOL-023-C` | Windows-native PowerShell shell | Complete — PR #126 / `592254d73a98166df48da0139a02df67e9cd2cd6` | TOOL-023-A |

## I170 Boundary

I170 selected and completed only TOOL-023-A, TOOL-023-C and the portability fixtures required to validate them. It did not change the 120-second default, add global timeout configuration or complete TOOL-023-B.

Completed I170 behavior:

- one pinned absolute timeout independent of stdout/stderr activity;
- Windows `powershell.exe -NoLogo -NoProfile -NonInteractive -Command` with tool name `powershell`;
- Unix `bash` / `sh -c` behavior and ADR-007 pre-exec hardening preserved;
- child-local dangerous environment removal without parent mutation;
- one authoritative shell contribution under the current composition architecture;
- platform-aware prompts, permission resources, MCP/product inventories and Agent output handling;
- explicit Windows inert-token allowlist for reusable cwd templates, with computed PowerShell expressions falling back to exact resources;
- stable `/` workspace-relative paths and conservative Windows long-list metadata;
- CRLF, Unix-only symlink/hardening and portable temporary-directory fixture corrections;
- full Windows Rust CI in addition to the installer fixture.

## Exclusions

- No `exec` process or timeout behavior change.
- No timeout default/config key change.
- No POSIX-to-PowerShell translation, `cmd.exe` fallback, PowerShell parser or shell selection setting.
- No Windows Job Object, Unix process-group or descendant process-tree supervision claim.
- No I169 steering work.

## Decisions And Evidence

- ADR-007 preserves Unix hardening.
- ADR-012 preserves conservative exact permission fallback for complex commands.
- ADR-053 preserves the authoritative contribution/outer composition boundary.
- ADR-057 is Accepted for the Windows process, platform identity, environment, permission-template and direct-child timeout boundary.
- I170 completion/evidence lives in `docs/iterations/I170-windows-workspace-validation-unblocker.md`.
- Accepted security review: `docs/reference/I170-WINDOWS-SHELL-SECURITY-REVIEW-2026-08-01.md`.
- Exact implementation Head: `8cfe8edb2dbda581244f583fb809591391a54298`.
- Exact-head CI: `30705366763`.
- Walkthrough artifact: `8820174164`.
- Completion merge: `592254d73a98166df48da0139a02df67e9cd2cd6`.
- Historical recovery PR #121 remains archival only.

## Completion Condition

The Epic remains Partial because TOOL-023-B is not implemented or explicitly deferred by its owner. I170 completed A/C without claiming the configuration child.

## Residual Destination

- TOOL-023-B owns timeout default/configuration.
- TOOL-024 or a separately approved process-runtime owner must address descendant process-tree supervision.
- A future PowerShell parser or PowerShell 7 selection requires a new decision.
