# TUI-051: Resumed Session Interactivity Under Provider Delay

| Field | Value |
|---|---|
| Story ID | TUI-051 |
| Type | TUI / Runtime Reliability Story |
| Priority | P0 |
| Status | Intake / Unclaimed |
| Source Issue | #272 |
| Selected Iteration | None |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #272 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-17 |
| Handoff / Release Condition | Close PROVIDER-005/#270/#271 first, then refine and select a separate runnable iteration with its own effective claim. |

## Identity / Goal / Value

A resumed large Session must remain input-responsive during provider delay, expose bounded retry
progress, cancel promptly through the Session-owned route, and restore terminal modes on supported
termination paths.

## Intake Scope

- Preserve the measured Issue #272 incident evidence without assuming the exact cancellation loss
  point.
- Refine provider retry status, resumed-turn cancellation, large-history projection invalidation
  and terminal restoration into one runnable acceptance contract after #271 closes.
- Coordinate with NET-001, I166, I200 and I206 without importing their implementation authority.

## Exclusions

- No implementation, activation, branch, timeout-policy change or transcript mutation.
- No PROVIDER-005 UTF-8 decoding change and no reuse of #271 emergency authorization.
- No I200 scroll semantic change or I206 steering activation.

## Acceptance For Intake

- [ ] #271 reaches a terminal disposition before TUI-051 is selected for implementation.
- [ ] The exact cancellation loss point is reproducible across TUI, bridge, actor and provider
      boundaries.
- [ ] A runnable/testable child iteration, user-facing documentation target and effective claim are
      recorded before implementation.
