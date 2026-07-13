# Iteration I123: Installation And Trial Confidence

> Document status: Planned — blocked on I122 Complete
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

Not started. Do not activate until I122 is Complete.
