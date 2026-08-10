# ARCH-034-R04-AG12: SQLite Validator Policy Integrity

| Field | Value |
|---|---|
| Parent | ARCH-034-R04 |
| Finding | PR #184 review residuals 3-5 / validator policy linkage and diagnostics |
| Status | Refinement — explicit post-I183 residual |
| Priority | P2 |
| Selected Iteration | None |
| Preserved behavior | ADR-008 accepted consumers, locked graph semantics, bundled SQLite behavior and standard governance results |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | None |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable until separately selected and claimed |
| Authorization Evidence | PR #184 independent reviews `5235077449` and `5235367999` classified these findings as non-blocking follow-up. |
| Implementation PR | Not started |
| Last Updated | 2026-08-10 |
| Handoff / Release Condition | Refine the policy source of truth and host-failure contract before any validator change. |

## Confirmed Baseline

- The validator's exact `ACCEPTED_CONSUMERS` set is checked against locked dependency reality, but
  ADR-008 clause 2 is not parsed or mechanically compared with that set. A future accepted consumer
  change could update Cargo plus the constant while leaving the ADR prose stale.
- Standard project governance now invokes Python and `cargo metadata --locked`; the repository has
  not explicitly selected whether this host dependency should remain online-capable, use
  `--frozen`/offline behavior, or degrade when the toolchain/cache is unavailable.
- Explicit UTF-8 decoding is correct for Cargo JSON. A non-UTF-8 localized Cargo stderr is caught
  safely, but the decode failure can mask Cargo's actionable diagnostic.

## Scope And Acceptance

- Choose one authoritative, machine-checkable linkage between ADR-008's accepted consumer list and
  the validator constant without weakening exact set equality against locked metadata.
- Record and test the project-governance host dependency/network/cache policy before changing Cargo
  invocation flags; cold-cache failure must remain explicit.
- Preserve actionable Cargo stderr on decode anomalies while continuing to decode Cargo JSON as
  UTF-8 and fail closed on invalid metadata.
- Add focused fixtures for ADR drift and diagnostic decoding behavior.

## Exclusions And Residuals

No consumer, dependency, manifest, runtime SQLite, schema, migration, timeout, retry or containment
change. AG-11 remains the separate runtime containment evidence owner.

## Minimum Validation

Focused validator fixtures on Unix and Windows, both governance validators, locked release
preflight, exact-head CI and independent review.
