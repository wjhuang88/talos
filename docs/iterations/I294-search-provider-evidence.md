# Iteration I294: Native Provider Admission And Regional Evidence

> Document status: Active
> Parent: SEARCH-001-D / #644 / SEARCH-001 / #624
> Objective: qualify native zero-key routes with provenance-backed evidence before E/F authorization.

| Field | Value |
|---|---|
| Story | SEARCH-001-D |
| Source Issue | #644 |
| Depends on | Accepted ADR-085; SEARCH-001-A / #625 complete; B adapter boundary available |
| Claim State | Claimed |
| Research PR | #668 and #673 (merged; partial evidence) |
| Completion | Pending |

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
| Implementation PR | #668 and #673 (merged; evidence only) |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Governance-only claim+activation PR; exact-head CI, governance validators, remote Issue reconciliation and merge-time CAS required. |
| Last Updated | 2026-10-08 |
| Handoff / Release Condition | Claim effective after #665 merge `7019f63d3df3d6eb58473a53a499b9b7e8b98925`; evidence research only, with E/F provider implementation gated by the resulting matrix. |

## Scope

Record accepted, rejected and unresolved candidates with terms/automation review, parser maintainability, endpoint and failure-domain independence, privacy/query exposure, deterministic fixtures, protocol observations and reproducible regional evidence including genuine mainland-China evidence.

## Non-goals

No production behavior change, provider addition, GeoIP routing, CAPTCHA/access-control bypass, default relay, or paid-provider auto-selection.

## Acceptance

The candidate matrix has dated evidence links and honest limitations; two independent native zero-key routes are explicitly admitted or remain blocked; E/F authorization is gated by the result.

## Evidence Register

The working candidate matrix, regional observation schema and admission gate are maintained in
[SEARCH-001-D provider evidence register](../reference/SEARCH-001-D-PROVIDER-EVIDENCE-2026-10-06.md).
It is a research artifact only; no candidate is admitted by the register itself.

## Partial Research Evidence

- #668: evidence register, merged at `08b5bd2188892e3d721d7b5e845fdb35b522af19`.
- #673: dated source review, merged at `2708db4f56362d626d10b4e5c53fae880e7d3b08`.
- DDG/Bing terms follow-up: see the register's 2026-10-08 review. No route admitted.

Terms applicability, privacy, authorized fixtures, independent failure domains and genuine regional
observations remain incomplete. E/F remain gated; this is partial evidence, not completion.
