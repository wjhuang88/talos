# Steering Follow-up Sequence

Source intake: [GitHub Issue #267](https://github.com/wjhuang88/talos/issues/267).

This sequence records three newly reported steering behaviors. It is a planning container only;
the three stories remain independently claimable and no implementation authorization is implied.

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
- The current state of all three stories is `Planned / Unclaimed`; no implementation branch exists.

## Resume

Before selecting I206, refresh the exact-main inventory and create the pending intake Issue. Select
I207 only after the shared padding contract is identified. Select I208 after the first two contracts
are accepted or their interaction is explicitly resolved.
