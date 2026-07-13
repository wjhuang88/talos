# Iteration I123: Installation And Trial Confidence

> Document status: Active (2026-07-13) — Gate 0 passed; F130 in progress
> Published plan date: 2026-07-13
> Planned objective: Make installation failure modes and a clean-HOME local trial repeatable by a
> second operator without real credentials or release actions.
> Baseline rule: preserve this target after publication; changed targets use a new iteration ID.
> MVP deliverable: installer fixtures plus a replayed clean-HOME trial packet pass on supported CI.

## Published Baseline

### Selected Stories

| Story | Parent | Outcome |
|---|---|---|
| F130 | I123 | POSIX/PowerShell installer fixture matrix |
| F131 | I123 | Clean-HOME real-binary trial smoke |
| F132 | I123 | Second-operator recovery/troubleshooting replay |
| F133 | I123 | Honest trial-readiness report and residual owners |

### Scope

- Test asset naming, archive selection, checksum mismatch, offline/unreachable source, extraction,
  executable placement, and cleanup behavior without publishing artifacts.
- Extend clean-HOME smoke for setup/config masking, mock provider turn, session list/resume/export,
  permission preflight Ask/Deny, diagnostics, and graceful interruption.
- Document exact supported platforms and distinguish local, CI, static, and untested evidence.

### Non-Goals

- No tag, publish, deployment, GitHub Release mutation, production credentials/network provider,
  v1.0/REL-002 claim, telemetry, auto-update, or destructive installer cleanup.

### Acceptance

- Supported installer script paths pass fixture tests; checksum and offline failures are explicit and
  leave no false success state.
- Trial smoke starts from a disposable HOME and requires no real secret or external provider.
- A second operator replays the packet and records result/variance.
- Final report keeps REL-002 NO-GO unless a separate audit changes it and requests no release action.

### Validation And Docs

- Installer fixture matrix, `scripts/talos_smoke.sh`, standard validation ladder, install pages,
  troubleshooting guide, iteration/index/Board, and dated trial report.

### Risks And Fallback

- Missing platform runner: require static parse plus platform CI and label evidence honestly.
- Network instability: use local fixtures; external asset checks are separate non-blocking evidence.

## Execution Record

Activated 2026-07-13 after I122 Complete.

### F130 — POSIX/PowerShell installer fixture matrix (Complete 2026-07-13)

- Extended `scripts/test_installer_fixtures.sh` from 4 to 9 POSIX cases: preserved
  install / latest / checksum-mismatch / offline; added unsupported-OS, unsupported-arch,
  install-dir override + executable placement, temp cleanup, corrupted-archive extraction.
  Result: **9/9 passed** (no network — fake `curl` + fake `uname` via PATH injection).
- Added `scripts/install_fixtures.ps1` (locally mocks `Invoke-RestMethod` / `Invoke-WebRequest`;
  no network) and `scripts/test_installer_fixtures_ps1.sh` (SKIP with honest label when `pwsh`
  is absent — never a false failure). PowerShell matrix: success + `talos.exe` placement,
  `latest` resolution, offline terminating error, ARM64 explicit unsupported message.
  Result: **4/4 passed** (`pwsh` 7.6.2 present in this environment).
- **Honest residual**: `install.ps1` performs no checksum verification (unlike `install.sh`).
  A checksum-mismatch case for PowerShell therefore cannot exist; this is documented in the
  fixture output, not faked. Adding checksum verification to the installer is a maintainer
  decision outside F130 scope (fixture tests only).
- Acceptance met: both installer script paths pass fixture tests; checksum (POSIX) and offline
  failures are explicit and leave no false success state.

### F131 — Clean-HOME real-binary trial smoke (Next)
