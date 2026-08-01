# TOOL-023-A: Fix Shell Timeout Defeated by Continuous Output

**Status**: In Progress — implemented in Draft PR #126; exact-head cross-platform validation pending (2026-08-01)
**Priority**: P1
**Parent Epic**: TOOL-023
**Type**: Technical Story (bug fix)
**Depends on**: none
**Selected Iteration**: I170

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | GPT-5.6 Thinking / talos recovery session 2026-08-01 |
| Work Slice | TOOL-023-A within I170: one absolute shell timeout, partial-output preservation and direct-child cleanup without changing timeout defaults or I169 semantics. |
| Claimed At | 2026-08-01 |
| Source Issue | #119 (I170 dependency recovery context) |
| Governance Claim PR | #122 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | PR #122 merged the I170 claim after exact-head governance, collaboration, remote-owner and CI validation; its recorded I170 slice explicitly includes the absolute shell timeout. |
| Implementation PR | #126 |
| Last Updated | 2026-08-01 |
| Handoff / Release Condition | Release only through the I170 claim owner after PR #126 merges and exact Completion Commit evidence is recorded. |

## Problem

The historical shell loop created `tokio::time::sleep(timeout_duration)` inside each `select!` iteration. Every stdout/stderr line dropped and recreated the timer, so a chatty child could evade the advertised timeout indefinitely.

## Goal / Value

The platform shell escape hatch enforces one absolute wall-clock deadline from spawn, independent of output frequency, while preserving partial output and direct-child cleanup.

## Scope

- Create and pin one deadline before stdout/stderr/wait arbitration.
- Preserve current timeout range/default, output header, exit code and `[timeout]` projection.
- Kill and wait for the direct child at expiry, then drain already-produced output.
- Keep Unix `sh -c` hardening unchanged and share the repaired loop with Windows PowerShell.
- Record descendant process-tree supervision as an explicit residual rather than overstating direct-child kill.

## Exclusions

- No timeout default/configuration change (TOOL-023-B).
- No process-group or Windows Job Object implementation.
- No I169 steering behavior.

## Decision Links And Constraints

- ADR-007 preserves Unix pre-exec hardening.
- ADR-057 defines the cross-platform process and direct-child timeout boundary.
- External process failures return bounded tool errors; they do not crash Talos.

## State / Status Owners

- Story status and acceptance: this file.
- Execution/evidence: `docs/iterations/I170-windows-workspace-validation-unblocker.md`.
- Process decision: `docs/decisions/057-windows-powershell-process-boundary.md`.
- Implementation: Draft PR #126.

## Acceptance For Behavior

- A child emitting output repeatedly still returns `[timeout]` near the configured absolute duration.
- Output produced before expiry remains in the result.
- A child exiting normally preserves complete output and actual exit code.
- Timeout kills and reaps the direct child before returning.
- Output activity, pipe closure or stderr activity cannot restart the deadline.

## Acceptance For Technical Work

- [x] One pinned deadline replaces per-iteration sleeps.
- [x] Fixed regression test emits continuous output and asserts a bounded timeout.
- [x] Partial-output timeout behavior is tested.
- [x] Direct-child/process-tree limitation is documented in ADR-057 and the security review.
- [ ] Exact final Head passes focused Unix and Windows tests.
- [ ] Exact final Head passes full locked workspace format/check/Clippy/tests on macOS and Windows.
- [ ] Governance, collaboration, release preflight and review gates pass.

## Residual Destination

Descendant process-tree supervision belongs to a separately reviewed process-runtime/TOOL-024 slice. I170 must not claim it through direct-child `kill()`.
