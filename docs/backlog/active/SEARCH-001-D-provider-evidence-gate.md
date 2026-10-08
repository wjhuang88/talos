# SEARCH-001-D: Native Provider Admission And Regional Evidence

**Status**: Active / Claimed; selected in I294
**Type**: Research / Evidence Gate
**Parent Epic**: [SEARCH-001](SEARCH-001-zero-config-global-search.md) / [Issue #624](https://github.com/wjhuang88/talos/issues/624)

| Field | Value |
|---|---|
| Story ID | SEARCH-001-D |
| Source Issue | [#644](https://github.com/wjhuang88/talos/issues/644) |
| Depends on | Accepted ADR-085; SEARCH-001-A / #625 merged as `4567bf85` |
| Selected Iteration | [I294](../../iterations/I294-search-provider-evidence.md) |
| Implementation PR | #668 and #673 (merged; evidence only) |
| Last Updated | 2026-10-08 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | GPT-6 Sol / talos开发 session |
| Work Slice | SEARCH-001-D native provider admission and regional evidence only |
| Claimed At | 2026-10-06 |
| Source Issue | #644 |
| Governance Claim PR | #665 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Governance-only claim+activation PR; exact-head CI, governance validators, remote Issue reconciliation and merge-time CAS required. |
| Handoff / Release Condition | E/F remain blocked until the candidate matrix and regional evidence gate are accepted. |

## Goal

Qualify candidate native zero-key routes before E/F. Record terms and automation policy,
maintainability, parser fixtures, endpoint and failure-domain independence, privacy/query exposure,
DNS/TLS/HTTP/parser observations, challenge interpretation and provenance-backed regional evidence.

Working evidence register: [SEARCH-001-D provider evidence register](../../reference/SEARCH-001-D-PROVIDER-EVIDENCE-2026-10-06.md).

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

## Partial Research Evidence

- #668: evidence register, merged at `08b5bd2188892e3d721d7b5e845fdb35b522af19`.
- #673: dated source review, merged at `2708db4f56362d626d10b4e5c53fae880e7d3b08`.
- DDG/Bing terms follow-up: see the register's 2026-10-08 review. No route admitted.

Terms applicability, privacy, authorized fixtures, independent failure domains and genuine regional
observations remain incomplete. E/F remain gated; this is partial evidence, not completion.
