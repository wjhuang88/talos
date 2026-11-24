# PROVIDER-003-A: Authentication capability ADR and threat model

**Status**: In Progress / Claimed (effective on #1014 merge)
**Source Issue**: #1013
**Parent Epic**: PROVIDER-003 / #132
**Selected Iteration**: I336

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | Define acquisition, refresh, final-request signing and replay contracts in an ADR and threat model. |
| Claimed At | 2026-11-23 |
| Source Issue | #1013 |
| Governance Claim PR | #1014 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Claim #1014 used independent review. For the ADR only, the sole maintainer authorized the PR #1020 exception in AGENT-COLLABORATION §3. Independent Agent security re-review passed on d50941297d1cf7491a813c7f1cb9cd744929d4dd (PR comment 7122440728). |
| Implementation PR | Not started |
| Last Updated | 2026-11-23 |
| Handoff / Release Condition | Merge the reviewed architecture deliverable and record its commit before closing #1013. |

## Acceptance

1. Map static keys, Copilot exchange, Vertex ADC, Azure supplied bearer tokens and regional Bedrock signing to main source and controlled test evidence.
2. Decide credential acquisition, expiry, cancellation, concurrent refresh and stale-write handling separately from authorization of the final destination, headers and body.
3. Specify replay eligibility after authentication failure, limits for command-based acquisition, refresh-token storage, diagnostic redaction and compatibility with existing configurations.
4. Trace threats and negative tests to lifecycle, request or acquisition owners B–G. Record open decisions and review the proposed ADR independently before accepting it.

This slice delivers architecture only. Each later implementation requires its own claim. Parent #132 stays open.
