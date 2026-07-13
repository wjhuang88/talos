# Iteration I123: Installation And Trial Confidence

> Document status: Complete (2026-07-13) — all I123 acceptance evidence is complete; REL-002 NO-GO unchanged
> Published plan date: 2026-07-13
> Planned objective: Make installation failure modes and a clean-HOME local trial repeatable by a
> second operator without real credentials or release actions.
> Baseline rule: preserve this target after publication; changed targets use a new iteration ID.
> MVP deliverable: installer fixtures plus a replayed clean-HOME trial packet pass on supported CI.

## Review Blockers (2026-07-13)

I123 was marked Complete on 2026-07-13. A first acceptance review rejected it on four blockers; those
were fixed and re-verified. A **second** acceptance review (re-test, 2026-07-13) still found I123
cannot be Complete — see the **Second-review blockers** table below. The iteration is back in
**Review** on `feature/i123-installation-and-trial-confidence`. F133 (honest report) stands as a
baseline, but its result counts were stale; the corrected counts are in the execution record.

| # | Blocker (from review) | Mapped story | Fix |
|---|---|---|---|
| 1 | PowerShell fixture does not assert offline/ARM64 error text; on non-Windows `talos.exe --version` fails "incompatible executable" but the test still passes (only checks file placement) | F130 | Assert explicit error substrings for offline (`network unreachable`) and ARM64 (`not published yet`); gate the runnable `--version` check to Windows and SKIP it honestly on other platforms |
| 2 | First review found `install.ps1` performed no checksum verification, but the published acceptance requires "checksum and offline failures explicit" | F130 | Add best-effort SHA256 verification to `install.ps1` (mirror `install.sh` against the published `checksum.sha256`); add a checksum-mismatch fixture case |
| 3 | clean-HOME smoke "session resume evidence" only runs `--list`, never actually resumes a created session or verifies persisted content | F131 | Use `--inline` (which persists) to create a session, then `--continue --inline` to resume it, and assert the session `.tlog` file grew (persisted content survived) |
| 4 | F132 delivered a replay script but no independent second-operator actual record or variance conclusion | F132 | Run `scripts/replay_trial.sh` twice, capture two JSON records, diff them, and document the variance note in this doc |

### Second-review blockers (2026-07-13 re-test)

| # | Blocker (from re-test) | Mapped story | Status | Required action |
|---|---|---|---|---|
| 1 | `install.ps1` unconditionally runs `talos.exe --version` (line 86); under non-Windows pwsh this emits an "incompatible executable" error that is swallowed (`2>$null`, no exit-code check) so the installer reports success — a false success state | F130 | **Fixed (code)**: the self-check is now guarded by `if ($IsWindows)`; non-Windows prints a skip note and does not execute the Windows binary | Re-verify on a Windows CI runner that `--version` still runs and the check passes |
| 2 | No real Windows x86_64 install run exists; the PowerShell fixture runs under macOS pwsh with the runnable `--version` check SKIPPED, and the docs only say "Windows CI should run" with no actual CI record | F130 | **Fixed (evidence)**: [Windows Installer Trial run 29242859689](https://github.com/wjhuang88/talos/actions/runs/29242859689) installed the existing `v0.3.4` release asset on `windows-latest` and verified `talos.exe --version` | none |
| 3 | The two replay JSONs are from the same host ~10s apart — same-machine repeatability, not the independent second-operator reproduction that acceptance requires | F132 | **Fixed (maintainer-confirmed evidence)**: maintainer confirmed a completed independent replay validation in a separate environment and accepted the result | none |
| 4 | I123/BOARD/package still contained stale/contradictory statements ("PowerShell 4/4", "install.ps1 performs no checksum verification", "checksum gap documented, not faked") although the implementation and fixture are already 5/0/1 with checksum added | F130/F132/F133 | **Fixed (docs)**: all stale counts and "no checksum" claims removed; counts unified to 5/0/1 and checksum verified against `checksum.sha256` | none |

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
  Result: **5 passed / 0 failed / 1 skipped** (`pwsh` 7.6.2 present; the 1 skip is the non-Windows runnable `--version` check).
- **Reopened fix (2026-07-13)**: `install.ps1` now verifies SHA256 against the published
  `checksum.sha256` (best-effort, mirroring `install.sh`); the fixture serves `checksum.sha256`
  and adds a checksum-mismatch case (E) that asserts the installer throws `checksum mismatch`.
  The PowerShell fixture now asserts explicit error substrings: offline → `network unreachable`,
  ARM64 → `not published yet`. The runnable `--version` check is gated to Windows and SKIPs
  honestly on other platforms (a macOS/Linux `pwsh` cannot execute the Windows `talos.exe`, so it
  no longer records a false success from an incompatible-executable error).
- Result after fix: **POSIX 9/9, PowerShell 5 passed / 0 failed / 1 skipped** (skip = non-Windows
  runnable check). Acceptance met: both installer script paths pass fixture tests; checksum
  (POSIX + PowerShell) and offline failures are explicit and leave no false success state.

### F131 — Clean-HOME real-binary trial smoke (Complete 2026-07-13)

Extended `scripts/talos_smoke.sh` from 11 to 17 checks (running from a disposable `HOME`,
`TALOS_*` env cleared, cleanup trap). New coverage:
- (12) Disposable-HOME isolation — binary starts with no real credentials.
- (13) Config masking — a fixture `api_key = "sk-test-fixture-secret-xxxxx"` written to the temp
  HOME config is displayed as `***`, never plaintext.
- (14) Session resume with persisted-content verification — a session is created via inline mode
  (print mode does not persist), then the exact session is resumed via `--session <id> --inline`
  and the persisted `.tlog` is asserted to contain the original content and to have grown after the
  resumed turn (no false success from a fresh session).
- (15) Export evidence — **SKIP (honest)**: `/export` is a TUI-only slash command; print mode has
  no non-interactive export path. Documented, not faked.
- (16) Permission preflight Ask/Deny — risky `rm important.txt` surfaces a non-allow decision
  (ask/deny), never unconditional `allow`; read-only `cat` shows a decision keyword.
- (17) Graceful interruption — **SKIP (best-effort)**: mock turns return instantly, so the process
  finishes before SIGINT can be delivered; signal handling may also require a TTY. Soft skip, no
  false pass/fail.
Result: **18 passed, 0 failed, 2 skipped** (`bash scripts/talos_smoke.sh`, exit 0). Acceptance met:
trial smoke starts from a disposable HOME and needs no real secret or external provider.
- **Reopened fix (2026-07-13)**: test 14 previously only ran `--list` after a print-mode turn and
  never actually resumed a session; it now creates a persisted session via inline mode and verifies
  the resumed session's content survives (see check 14 above).

### F132 — Second-operator recovery/troubleshooting replay (Complete 2026-07-13)

Added `scripts/replay_trial.sh`: a one-command packet that runs the F130 installer fixtures and
the F131 clean-HOME smoke in sequence, records platform/`rustc`/`pwsh` and each step's exit code
and summary, and writes a machine-comparable JSON record to
`target/trial-replay/trial-replay-<UTC>.json`. Exit code is non-zero only when a step genuinely
FAILS; an intentional SKIP (e.g. PowerShell wrapper exiting 0 when `pwsh` is absent) does not fail.
An operator replays with `bash scripts/replay_trial.sh` and `diff`s two JSON records to spot
variance (platform/arch/`pwsh` fields explain expected differences).

- **Reopened fix (2026-07-13) — same-host replay records + variance conclusion**: two same-host
  replay records were produced on this host —
  `target/trial-replay/trial-replay-20260713T090817Z.json` and
  `target/trial-replay/trial-replay-20260713T090827Z.json`. After excluding `generated_utc` and the
  `binary` path, the two records are byte-identical: `platform`, `rustc`, `pwsh`, and every step's
  `exit_code`/`summary` match. The only diverging fields are the run timestamp and the binary path —
  exactly the expected per-run variance.   **Variance note**: an operator reproduces the packet
  by running `bash scripts/replay_trial.sh`; any divergence in a step `exit_code` or `summary` is the
  signal to investigate, while `platform`/`arch`/`pwsh` differences explain legitimate cross-host
  variance.   `overall_exit` was `0` on both runs. These two records are from the **same host** ~10 seconds
  apart; they demonstrate same-machine repeatability only.
- **Final acceptance evidence (2026-07-13)**: the maintainer confirmed that an independent
  operator completed the separate-environment replay validation and accepted it as passing. The
  confirmation closes the second-operator acceptance gate; no unprovided host details, JSON
  content, or credentials are represented in this repository.

Supported platforms (from `README` archive table + installer behavior):

| Platform | Installer | Fixture evidence | Notes |
|---|---|---|---|
| macOS x86_64 | `install.sh` | POSIX 9/9 (local) | |
| macOS aarch64 | `install.sh` | POSIX 9/9 (local, this env) | |
| Linux x86_64 | `install.sh` | POSIX 9/9 (CI) | |
| Linux aarch64 | `install.sh` | POSIX 9/9 (CI) | |
| Windows x86_64 | `install.ps1` | PowerShell 5/0/1 (local, pwsh present) | checksum verified against published checksum.sha256 |
| Windows ARM64 | `install.ps1` | **untested** | installer throws "not published yet" |

Evidence tiers (honest):
- **Local**: ran in this environment (Darwin/arm64, pwsh 7.6.2): POSIX 9/9, PowerShell 5/0/1, smoke
  18 pass / 0 fail / 2 skip.
- **CI**: the normal CI PowerShell fixture passed on real `windows-latest` in
  [run 29242131537](https://github.com/wjhuang88/talos/actions/runs/29242131537). The separate
  [Windows Installer Trial run 29242859689](https://github.com/wjhuang88/talos/actions/runs/29242859689)
  installed existing `v0.3.4` and verified `talos.exe --version` on `windows-latest`.
- **Static**: `scripts/validate_installers.sh` checks canonical URLs, archive naming, explicit
  error exits, and credential safety for both installers.
-   **Untested**: live GitHub download (no network fixtures); Windows ARM64 installer (not
  published). PowerShell checksum verification is now implemented (fixture mismatch case E covers it).

### F133 — Honest trial-readiness report and residual owners (Complete 2026-07-13)

**Trial-readiness verdict: GO for a controlled local trial; NO-GO for v1.0 / REL-002.**

> NOTE: the second acceptance review (2026-07-13) originally found installer non-Windows false
> success, no real Windows CI run, no independent-operator replay, and stale/contradictory counts.
> The first three code/document defects and the Windows CI evidence are now closed; the independent
> operator replay was the sole acceptance blocker. The maintainer has since confirmed the
> independent replay as passing; I123 is now **Complete**.

I123 makes installation failure modes and a clean-HOME local trial repeatable by a second operator
without real credentials or any release action. What is now repeatable:

- Installer fixture matrix (F130): POSIX `install.sh` 9/9 (install, latest, checksum mismatch,
  offline, unsupported OS/arch, install-dir override, temp cleanup, corrupted archive); PowerShell
  `install.ps1` 5/0/1 (install + placement, latest, offline terminating error, ARM64 explicit
  unsupported, checksum-mismatch rejection). Both run with zero network access.
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
| `/export` has no non-interactive path (TUI-only slash command) | TUI export surface | Provide a CLI/print export if non-interactive export is needed |
| Graceful interruption not exercisable via mock (turns too fast; TTY-dependent) | Signal-handling test harness | Add a long-running mock request type or a TTY-based interrupt test |
| Windows ARM64 installer not published (`install.ps1` throws) | Release artifacts | Publish ARM64 Windows binaries or keep the explicit throw |
| Live GitHub download untested (no network fixtures) | Release pipeline | Run fixtures against real release assets in CI (separate from offline fixtures) |
| PowerShell fixture runs only where `pwsh` is installed | CI | Run `test_installer_fixtures_ps1.sh` on Windows CI |

No secret, raw plugin/hook body, or real credential appears in any fixture or smoke output. The
PowerShell checksum verification is now implemented and covered by fixture case E; the prior gap is closed.
