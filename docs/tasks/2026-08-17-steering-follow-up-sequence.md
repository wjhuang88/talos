# Steering Follow-up Sequence

Source intake: [GitHub Issue #267](https://github.com/wjhuang88/talos/issues/267).

This sequence records three steering behaviors. I206/TUI-048 is Complete / Closed after PR #411
merged as `9d7c87cb` and absorbs Issue #408's provider-switch symptom. I207/TUI-049 proposes
Complete / Closed after implementation merge `ca3b2fa7` and native-terminal acceptance; I208 has
claim PR #486 open, which is ineffective until merge, and no implementation authorization is implied.

## Ordered Stories

1. `TUI-048 / I206` — admit accepted steering into the Session and activate one next turn when
   `Esc` cancels the active turn.
2. `TUI-049 / I207` — make steering wrapping honor the shared left/right padding contract.
3. `TUI-050 / I208` — define and implement insertion after a model response or tool call boundary.

## Shared Guardrails

- `TUI-044 / I169` and ADR-056 remain complete/accepted historical baselines.
- Each story needs its own owner, effective Collaboration Claim, implementation PR, exact-head
  validation and independent review before activation or completion.
- No story may change durable steering custody, Session identity, permission policy or execution
  authority without explicit change control.
- Current state: I206/TUI-048 and I207/TUI-049 are `Complete / Closed`; I208 claim PR #486 is open
  and ineffective until merge.

## Resume

I206 and I207 retain their completed implementation evidence. I208 implementation starts only after
claim PR #486 merges and the first two contracts are accepted or their interaction is explicitly
resolved.
