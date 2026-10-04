# SEARCH-001-D: Native Provider Admission And Regional Evidence

**Status**: Proposed / Unclaimed; selected in I294
**Type**: Research / Evidence Gate
**Parent Epic**: [SEARCH-001](SEARCH-001-zero-config-global-search.md) / [Issue #624](https://github.com/wjhuang88/talos/issues/624)

| Field | Value |
|---|---|
| Story ID | SEARCH-001-D |
| Source Issue | [#644](https://github.com/wjhuang88/talos/issues/644) |
| Depends on | Accepted ADR-085; SEARCH-001-A / #625 merged as `4567bf85` |
| Selected Iteration | [I294](../../iterations/I294-search-provider-evidence.md) |
| Implementation PR | None |
| Last Updated | 2026-10-03 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | None; provider evidence gate is dependency-ready but unclaimed |
| Claimed At | Not applicable |
| Source Issue | #644 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Handoff / Release Condition | Select a bounded research iteration and establish an effective claim before execution |

## Goal

Qualify candidate native zero-key routes before E/F. Record terms and automation policy,
maintainability, parser fixtures, endpoint and failure-domain independence, privacy/query exposure,
DNS/TLS/HTTP/parser observations, challenge interpretation and provenance-backed regional evidence.

## Constraints

- Evidence and research only; do not change production search behavior or add a provider.
- Do not fabricate geography, bypass access controls/CAPTCHAs or assume CI proves regional reachability.
- No GeoIP routing, default relay or paid-provider auto-selection.
- A route is not admitted without independent infrastructure, maintainable parser ownership,
  deterministic fixtures, terms review and real regional evidence.

## Acceptance

- Candidate matrix records admitted, rejected and unresolved routes with dated evidence links.
- Mainland-China and other regional evidence has a reproducible acquisition path and honest limits.
- The result explicitly gates or blocks E/F; no assumption becomes a default route.

## User-Facing Documentation

Publish only evidence-backed availability and limitations. Do not promise universal or mainland-
China availability from an unverified environment.

## Rollback

Keep all unqualified candidates out of the default distribution and retain the accepted
rust-websearch compatibility path while evidence is incomplete.
