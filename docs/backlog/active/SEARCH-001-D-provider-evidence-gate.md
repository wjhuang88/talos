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
| Implementation PR | #668, #673, #676, #686, #687 and #693 (merged); #825 (Draft evidence refresh) |
| Last Updated | 2026-11-01 |

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
Latest bounded refresh candidate: [2026-11-01 provider refresh](../../reference/SEARCH-001-D-PROVIDER-REFRESH-2026-11-01.md).

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
- #676: terms follow-up, merge `916bc70bc43dcbae1679ea253eb4f2961c459d39`.
- 2026-10-09 independent-API source review: Mwmbl anonymous standard search is the next technical
  candidate; Purili applicable-use conditions remain unresolved; Stract hosted route is parked.
  Source inspection and published policy review only; no live query or regional measurement.

Terms applicability, privacy, authorized fixtures, independent failure domains and genuine regional
observations remain incomplete. E/F remain gated; this is partial evidence, not completion.

## Standard Route Qualification Checkpoint

#686 source stage merged at `fecc9583992ccf2c0678b58e902f4a3bdcf19d2b`.
[Mwmbl route packet](../../reference/SEARCH-001-D-MWMBL-ROUTE-QUALIFICATION.md) adds a pinned
source call-chain/provenance audit and bounded query-free public-schema/input-validation evidence.
No search query, result response, production behavior or native admission. Source/policy/deployment,
fixtures, failure-domain and real regional acceptance remain pending; this is partial D evidence.

## Offline Wire Evidence Stage — 2026-10-10

#687 merged at `9e376155ddc7b6c3644bf65f69b4c388956c1b5c`. I294 owns a disposable
integration-test parser and independently invented fixture for partial M-Q02..M-Q05 evidence.
No production adapter, live response, request, admission or whole-row acceptance. Experimental
policies still require production review; hosted execution/privacy/use, transport, failure-domain
and genuine regional qualification remain pending.

#693 subsequently merged at `e63e0ee3793e2e8fd914f676224f982c6cae5593`. Exact head
`e8683d99bb04e4a823e547358e9ba0dbb68625ff` passed full unskipped CI3097 / run
`38024072972`; independent Agent review `6093757461` approved the exact candidate and final
single-maintainer CAS `6094343503` was recorded. Its nine invented-fixture tests remain only
partial offline M-Q02..M-Q05 evidence: no hosted search success, native admission, independent
failure-domain proof or regional qualification was established. D remains Active / Claimed.

## Provider Evidence Refresh Candidate — 2026-11-01

Draft #825 refreshes current Mwmbl and Purili evidence without production changes. Until #825
passes its own review/CI/CAS and merges, this section describes a candidate update rather than
accepted target-branch evidence.

The refresh records current official Mwmbl API material describing anonymous no-key access and a
privacy policy that says API search queries are not logged or retained. Current upstream source at
`86376eb363fb458f660223cb1111db8ea8f8bf6a` still exposes an unauthenticated v2 path and standard
search still enables Wikipedia alongside Mwmbl's index. Published API terms also assign returned
search results CC BY-NC-SA 4.0, so result-use/licence applicability remains a default-distribution
gate; documented anonymous quota wording and the inspected handler also require deployment
reconciliation. M-Q01/M-Q06/M-Q08 remain unproved.

Purili's current developer documentation explicitly describes no-account/no-key/no-fixed-quota
Search/Context APIs for applications, AI agents and RAG, with Search backed by its crawler-built
index. Its privacy policy gives a bounded access-log exposure (IP/path/timestamp up to seven days).
The previously cited general-use terms were not independently recovered during this refresh, so
applicable distributed-client/result-use permission remains unresolved rather than inferred from
integration documentation. No live provider response or regional observation was acquired.

A single bounded valid-query attempt to each provider stopped at DNS resolution in the current
execution environment. Because both unrelated provider names failed at the same local boundary,
this is recorded only as an execution-environment limitation, not provider outage or regional
reachability evidence.

The shortest remaining D path is therefore: reconcile Purili use/result conditions; complete a
Purili offline fixture/parser packet; obtain permitted valid-query observations for both strongest
candidates from a network with working DNS/HTTPS; compare native provenance/failure domains; and
obtain paired controlled regional observations including genuine mainland-China evidence. Until
those gates close, E/F remain unauthorized and #644/#624 remain open.
