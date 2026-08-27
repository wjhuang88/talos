# TUI-062: Provider Switch Recovery After Cancelled Queued Work

**Issue**: [#408](https://github.com/wjhuang88/talos/issues/408)
**Status**: Superseded before implementation by TUI-048 / I206
**Selected Iteration**: I206 through TUI-048

## Problem

After Esc interrupts an active conversation, provider switching can report that queued work must
be finished or cancelled without exposing a reliable recovery action. Characterize the queued-work
lifecycle and provider-switch guard before selecting an implementation slice.

## Scope Boundary

This owner is intake only. It does not authorize changes to provider selection, queued-message
semantics, session persistence, cancellation behavior, or permission policy. Preserve the current
behavior until a separately selected iteration and effective Collaboration Claim exists.

## Acceptance For Intake

- Reproduce the Esc/provider-switch sequence with exact CLI/TUI state transitions.
- Identify the owner of queued work and the explicit release/cancel boundary.
- Record compatibility, persistence, and regression-test requirements before decomposition.

Remote status reconciliation was recorded on Issue #408 on 2026-08-26; this intake remains
unselected and unclaimed.

The intake remains intentionally outside the Issue #59 closeout and has no implementation claim.

## Disposition (2026-08-27)

Read-only source tracing established that provider switching is blocked because Engine-owned
steering remains authoritative after Esc cancellation. ADR-049 forbids a TUI-local clear or guard
bypass, while TUI-048/I206 already requires that accepted steering become exactly one runnable
same-Session turn after Esc. TUI-062 is therefore superseded before implementation and Issue #408
is added to TUI-048/I206 acceptance. No separate TUI-062 branch, claim or code change is allowed.

## Completion Evidence

- Completion Commit: Not applicable; superseded before implementation.
