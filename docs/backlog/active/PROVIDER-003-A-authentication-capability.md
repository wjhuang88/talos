# PROVIDER-003-A: Authentication capability ADR and threat model

**Status**: Planned
**Source Issue**: #1013
**Parent Epic**: PROVIDER-003 / #132
**Selected Iteration**: I336

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Define acquisition, refresh, final-request signing and replay contracts in an ADR and threat model. |
| Claimed At | Not applicable |
| Source Issue | #1013 |
| Governance Claim PR | Pending |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-11-23 |
| Handoff / Release Condition | Merge the reviewed architecture deliverable and record its commit before closing #1013. |

## Acceptance

1. Map static keys, Copilot exchange, Vertex ADC, Azure supplied bearer tokens and regional Bedrock signing to main source and controlled test evidence.
2. Decide credential acquisition, expiry, cancellation, concurrent refresh and stale-write handling separately from authorization of the final destination, headers and body.
3. Specify replay eligibility after authentication failure, limits for command-based acquisition, refresh-token storage, diagnostic redaction and compatibility with existing configurations.
4. Trace threats and negative tests to lifecycle, request or acquisition owners B–G. Record open decisions and review the proposed ADR independently before accepting it.

This slice delivers architecture only. Each later implementation requires its own claim. Parent #132 stays open.
