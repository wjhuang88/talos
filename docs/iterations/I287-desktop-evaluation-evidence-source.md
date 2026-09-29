# I287 — Desktop Evaluation Evidence Source

> Document status: Planned / Unclaimed
> Planned objective: Provide a production-authoritative, session-bound Evaluation evidence source so Desktop distinguishes missing, stale and current evidence and gates Delivery without fabricated PASS results.

## Scope

- Define the authoritative producer, subject identity, workspace revision and provenance for Evaluation evidence.
- Connect the existing Desktop projection to current/stale/missing evidence without a Desktop-owned business store.
- Preserve fail-closed `EvaluationUnavailable` behavior until authoritative evidence exists.
- Add deterministic revision/staleness/Delivery tests and a native H6 acceptance recipe.

## Out Of Scope

- No new durable Mission schema or migration without a separate accepted contract.
- Fixtures, harnesses, or manually entered results cannot be production evidence.
- No new evaluator authority, global event bus, scheduling system, or unrelated Desktop redesign.

## Readiness And Acceptance

- Requires an accepted production Evaluation-source contract and owner before implementation.
- Evidence is bound to a session/task subject and workspace revision with provenance.
- Desktop distinguishes missing, stale and current evidence; Delivery requires current authoritative evidence.
- Restart/resume and changed-workspace cases cannot replay or upgrade stale evidence.
- Locked tests cover source lifecycle, revision mismatch, unavailable evidence and Delivery gating.
- The maintainer can reproduce and record native H6 evidence in #29 using the exact candidate build.

## Governance

No claim, owner, implementation branch, or PR is active. Activation requires an effective target-branch Collaboration Claim and an accepted contract/ADR for the production evidence source.
