# I286 — Desktop Provider Error Injection Contract

> Document status: Planned / Unclaimed
> Planned objective: Define and implement a safe, isolated provider failure path that exercises the real Desktop error/timeout presentation without changing credentials, disrupting networking, or treating a mock response as native evidence.

## Scope

- Define an explicit locally scoped failure-injection contract at the configured Desktop provider boundary.
- Preserve production fail-closed behavior; injection must never be enabled implicitly.
- Exercise provider error, timeout, cancellation and retry presentation through the real Desktop host path.
- Add deterministic automated coverage and a native acceptance recipe for H1.

## Out Of Scope

- No credential mutation, network disruption, provider workaround, release behavior, or permission-policy change.
- Existing `MockProvider` and localhost fixtures do not satisfy native H1 evidence.

## Readiness And Acceptance

- Requires an accepted security/API contract before implementation authority is granted.
- Default configured-provider execution remains unchanged.
- Error and timeout states reach the real Desktop path without panic, hang, credential change, or network side effect.
- Locked tests cover error, timeout, cancellation and retry cleanup.
- The maintainer can reproduce and record native H1 evidence in #29 using the exact candidate build.

## Governance

No claim, owner, implementation branch, or PR is active. Activation requires an effective target-branch Collaboration Claim and an accepted contract/ADR if the boundary changes public APIs or security behavior.
