# TUI-062: Provider Switch Recovery After Cancelled Queued Work

**Issue**: [#408](https://github.com/wjhuang88/talos/issues/408)
**Status**: Intake / Unclaimed
**Selected Iteration**: None

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

## Completion Evidence

- Completion Commit: Pending.
