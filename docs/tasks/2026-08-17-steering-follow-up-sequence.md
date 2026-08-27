# Steering Follow-up Sequence

Source intake: [GitHub Issue #267](https://github.com/wjhuang88/talos/issues/267).

This sequence records three steering behaviors. I206/TUI-048 is in Review after claim #410 merged
as `c3a121f0` and absorbs Issue #408's provider-switch symptom; I207/I208 remain independently
claimable and no later-story implementation authorization is implied.

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
- Current state: I206/TUI-048 is `Review / Claimed`; I207/I208 remain `Planned / Unclaimed`.

## Resume

I206 implementation starts only from its claim merge or later main. Select I207 only after the
shared padding contract is identified. Select I208 after the first two contracts are accepted or
their interaction is explicitly resolved.
