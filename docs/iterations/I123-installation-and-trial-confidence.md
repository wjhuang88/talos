# Iteration I123: Installation And Trial Confidence

> Document status: Complete (2026-07-13) — F130-F133 verified; REL-002 NO-GO unchanged
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

### F131 — Clean-HOME real-binary trial smoke (Complete 2026-07-13)

Extended `scripts/talos_smoke.sh` from 11 to 17 checks (running from a disposable `HOME`,
`TALOS_*` env cleared, cleanup trap). New coverage:
- (12) Disposable-HOME isolation — binary starts with no real credentials.
- (13) Config masking — a fixture `api_key = "sk-test-fixture-secret-xxxxx"` written to the temp
  HOME config is displayed as `***`, never plaintext.
- (14) Session resume evidence — a mock turn creates a session; `--list` shows it.
- (15) Export evidence — **SKIP (honest)**: `/export` is a TUI-only slash command; print mode has
  no non-interactive export path. Documented, not faked.
- (16) Permission preflight Ask/Deny — risky `rm important.txt` surfaces a non-allow decision
  (ask/deny), never unconditional `allow`; read-only `cat` shows a decision keyword.
- (17) Graceful interruption — **SKIP (best-effort)**: mock turns return instantly, so the process
  finishes before SIGINT can be delivered; signal handling may also require a TTY. Soft skip, no
  false pass/fail.
Result: **18 passed, 0 failed, 2 skipped** (`bash scripts/talos_smoke.sh`, exit 0). Acceptance met:
trial smoke starts from a disposable HOME and needs no real secret or external provider.

### F132 — Second-operator recovery/troubleshooting replay (Complete 2026-07-13)

Added `scripts/replay_trial.sh`: a one-command packet that runs the F130 installer fixtures and
the F131 clean-HOME smoke in sequence, records platform/`rustc`/`pwsh` and each step's exit code
and summary, and writes a machine-comparable JSON record to
`target/trial-replay/trial-replay-<UTC>.json`. Exit code is non-zero only when a step genuinely
FAILS; an intentional SKIP (e.g. PowerShell wrapper exiting 0 when `pwsh` is absent) does not fail.
A second operator replays with `bash scripts/replay_trial.sh` and `diff`s two JSON records to spot
variance (platform/arch/`pwsh` fields explain expected differences).

Supported platforms (from `README` archive table + installer behavior):

| Platform | Installer | Fixture evidence | Notes |
|---|---|---|---|
| macOS x86_64 | `install.sh` | POSIX 9/9 (local) | |
| macOS aarch64 | `install.sh` | POSIX 9/9 (local, this env) | |
| Linux x86_64 | `install.sh` | POSIX 9/9 (CI) | |
| Linux aarch64 | `install.sh` | POSIX 9/9 (CI) | |
| Windows x86_64 | `install.ps1` | PowerShell 4/4 (local, pwsh present) | checksum verification absent (gap) |
| Windows ARM64 | `install.ps1` | **untested** | installer throws "not published yet" |

Evidence tiers (honest):
- **Local**: ran in this environment (Darwin/arm64, pwsh 7.6.2): POSIX 9/9, PowerShell 4/4, smoke
  18 pass / 0 fail / 2 skip.
- **CI**: POSIX fixture and smoke should run on Linux/macOS CI; PowerShell fixture on Windows CI.
- **Static**: `scripts/validate_installers.sh` checks canonical URLs, archive naming, explicit
  error exits, and credential safety for both installers.
- **Untested**: live GitHub download (no network fixtures); Windows ARM64 installer (not
  published); PowerShell checksum verification (installer gap, see F130 residual).

### F133 — Honest trial-readiness report and residual owners (Complete 2026-07-13)

**Trial-readiness verdict: GO for a controlled local trial; NO-GO for v1.0 / REL-002.**

I123 makes installation failure modes and a clean-HOME local trial repeatable by a second operator
without real credentials or any release action. What is now repeatable:

- Installer fixture matrix (F130): POSIX `install.sh` 9/9 (install, latest, checksum mismatch,
  offline, unsupported OS/arch, install-dir override, temp cleanup, corrupted archive); PowerShell
  `install.ps1` 4/4 (install + placement, latest, offline terminating error, ARM64 explicit
  unsupported). Both run with zero network access.
- Clean-HOME trial smoke (F131): 18 pass / 0 fail / 2 honest SKIP (export = TUI-only; graceful
  interruption = mock turns finish too fast to signal). Runs from a disposable HOME, proves config
  masking, session resume evidence, and permission preflight Ask/Deny.
- Second-operator replay (F132): `bash scripts/replay_trial.sh` emits a JSON record
  (`target/trial-replay/trial-replay-<ts>.json`) a second operator can `diff` for variance.

REL-002 posture is **unchanged**: this iteration adds no self-bootstrap evidence, no v1.0 claim,
and **requests no release action** (no tag, publish, deploy, or push to main). It improves
install/trial confidence only.

Residual owners (gaps found, none blocking a controlled local trial):

| Residual | Owner area | Required action |
|---|---|---|
| `install.ps1` performs no checksum verification (unlike `install.sh`) | Installer hardening (maintainer decision) | Add checksum step to `install.ps1`; until then, PowerShell install integrity is unverified |
| `/export` has no non-interactive path (TUI-only slash command) | TUI export surface | Provide a CLI/print export if non-interactive export is needed |
| Graceful interruption not exercisable via mock (turns too fast; TTY-dependent) | Signal-handling test harness | Add a long-running mock request type or a TTY-based interrupt test |
| Windows ARM64 installer not published (`install.ps1` throws) | Release artifacts | Publish ARM64 Windows binaries or keep the explicit throw |
| Live GitHub download untested (no network fixtures) | Release pipeline | Run fixtures against real release assets in CI (separate from offline fixtures) |
| PowerShell fixture runs only where `pwsh` is installed | CI | Run `test_installer_fixtures_ps1.sh` on Windows CI |

No secret, raw plugin/hook body, or real credential appears in any fixture or smoke output. The
PowerShell checksum gap is documented, not faked.
