# ADR-087: Authoritative Context Budget And Request-Only Recovery

## Status

Accepted by maintainer on 2026-10-09 in this session. Implementation/API/privacy review
evidence remains required; acceptance does not establish an effective Collaboration Claim.

## Decision

The Agent's sealed request admission calculation is the sole authority for context budget.
Provider billing usage remains separate. Emit bounded numeric facts over the existing transient
non-exhaustive AgentEvent/session progress path; never encode them in assistant text, overload
billing usage or log request bodies. Display unknown before evidence and estimated budget after
evidence, including the requested output reservation.

Recover overflow in a disposable request projection, before dispatch, at both initial and
continuation boundaries. Recovery is deterministic and bounded, preserving system rules, user
intent, tool identity/pairing and recent evidence. Never mutate authoritative transcript history,
repeat tool execution, weaken permissions or increase the configured limit. Re-evaluate the exact
projection and reject irreducible requests. No unbounded model-driven recovery loop is introduced.

Recovery operates on a clone after BeforeProviderCall hooks and private-output projection,
without rerunning hooks or provider discovery. Protect System, Context, all user and multimodal
messages, and replay-enabled signed/redacted reasoning byte-for-byte. Remove only the oldest
complete tool exchanges atomically; preserve the latest complete exchange. Never truncate tool
arguments or leave orphaned calls/results. Existing pre-turn history compaction remains unchanged;
the history-preservation guarantee applies to this new admission recovery, not that legacy policy.

The numeric budget is an authoritative local admission estimate, not exact provider token usage.
Include reasoning in the estimator and ephemeral image reservation in the final admission facts.
Emit visible budget facts on rejection as well as successful admission; clear stale facts on
model/session changes. Requests still exceeding the limit after bounded recovery are not sent.

## Compatibility And Migration

A new AgentEvent variant is additive because that enum is already non-exhaustive. If a budget
field is added to StatusSnapshot, external struct literals must supply None or use Default;
that source-breaking presentation change must ship in a pre-1.0 minor release. Older providers
need not implement new methods or report usage. Existing billing usage retains its meaning.
I297 retry/timeout patch publication must exclude this incompatible presentation surface.

## Evidence And Reversal

Require tests for missing usage, continuation overflow, irreducible requests, Unicode, paired
tools, raw transcript preservation and no repeated execution; independently review API/privacy
boundaries. Revert if context recovery drops protected instructions, leaks hidden output,
loses durable evidence or admits an oversized request. No migration is needed to revert.

## Related

- [I298](../iterations/I298-context-budget-recovery.md)
- [MEM-005](../backlog/active/MEM-005-context-compaction-policy.md)
- [ADR-039](039-runtime-event-semantic-single-flow.md)
- [ADR-045](045-transient-model-private-tool-projection.md)
