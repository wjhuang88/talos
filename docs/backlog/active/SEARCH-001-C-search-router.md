# SEARCH-001-C: Talos-Owned SearchRouter

**Status**: Proposed / Unclaimed
**Type**: Implementation / Runtime Routing
**Parent Epic**: [SEARCH-001](SEARCH-001-zero-config-global-search.md) / [Issue #624](https://github.com/wjhuang88/talos/issues/624)

| Field | Value |
|---|---|
| Story ID | SEARCH-001-C |
| Source Issue | [#643](https://github.com/wjhuang88/talos/issues/643) |
| Depends on | SEARCH-001-B / #642; accepted ADR-085; NET-001 / #199 boundary coordination |
| Selected Iteration | None; requires a separately selected bounded iteration |
| Implementation PR | None |
| Last Updated | 2026-10-03 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | None; Talos-owned routing behavior is dependency-ready but unclaimed |
| Claimed At | Not applicable |
| Source Issue | #643 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Handoff / Release Condition | B delivered, then select a bounded iteration and establish an effective claim before implementation |

## Goal

Make Talos own search routing semantics: first-valid-success, bounded hedging, cancellation and
deadline propagation, backend identity, typed failure classes and bounded search-specific health.

## Constraints

- Preserve the model-facing `web_search` contract, Network permission and compatible output.
- Use observed network/backend outcomes; never use GeoIP, country hardcodes or startup probes.
- Do not duplicate NET-001 generic retries, backoff or circuit breakers.
- Paid providers remain explicit opt-in; Wikipedia remains a compatibility knowledge fallback.
- Retain rust-websearch as a rollback adapter until later evidence authorizes disposition.

## Acceptance

- Fixtures cover first-valid-success, fast-error/slow-success, all-fail, invalid result, timeout,
  cancellation and bounded hedge cases.
- Typed errors and backend identity survive routing decisions.
- Health state excludes query text and remains bounded/expirable.
- Locked tests, governance validators, focused review and rollback evidence pass.

## Rollback

Switch the built-in Search Provider back to the B compatibility composition without changing the
model-facing tool contract.
