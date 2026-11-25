# SEARCH-001-D3: Bing Direct-HTML Multi-Endpoint Feasibility

**Status**: Intake / Proposed / Unclaimed — no source, default-route, or provider admission

| Field | Value |
|---|---|
| Story ID | SEARCH-001-D3 |
| Type | Research Spike / evidence and feasibility gate |
| Parent Epic | [SEARCH-001 / #624](SEARCH-001-zero-config-global-search.md) |
| Source Issue | [#937](https://github.com/wjhuang88/talos/issues/937) |
| Predecessor | [SEARCH-001-D / #644](SEARCH-001-D-provider-evidence-gate.md), Complete / Closed **BLOCKED / ZERO-ADMISSION** |
| Sibling | [SEARCH-001-D2 / #848](SEARCH-001-D2-keyless-mcp-admission.md), independent unclaimed Exa/Parallel hosted MCP research |
| Selected Iteration | None |
| Evidence and acceptance | [D3 Bing direct HTML source/contract packet, 2026-11-13](../../reference/SEARCH-001-D3-BING-DIRECT-HTML-2026-11-13.md) |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | None — research-only intake, not an implementation claim |
| Claimed At | Not applicable |
| Source Issue | #937 |
| Governance Claim PR | Not applicable — documentation intake only |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | None |
| Last Updated | 2026-11-15 |
| Handoff / Release Condition | No live automated Bing SERP acquisition, new endpoint, test code, default-routing or provider admission until appropriate rights and an effective, independently reviewed owner/iteration claim. |

## Identity, goal and user value

The user observed that Bing search pages are directly requestable and asked whether
`bing.com` and `bing.cn` could improve Talos zero-configuration search. Evaluate
whether Talos can exploit **direct server-rendered HTML**, existing bounded Bing parsing
and a *single Bing provider family* with multiple request surfaces to improve
network-specific availability, without adopting Browser/CDP, creating an extra
provider identity or implicitly authorizing new query recipients.

The observed fact that a page may respond to an ordinary browser does **not**
establish authorization to automate third-party distributed default searches.

## Current code truth and scope boundary

- [SEARCH-001-C / #643](SEARCH-001-C-search-router.md) already owns the protected
  router and prepared Network authority; [SEARCH-001-L / #901](SEARCH-001-L-legacy-dependency-retirement.md)
  already removed `rust-websearch` and reqwest 0.12. Both remain closed.
- The protected compatibility transport currently freezes `GET https://cn.bing.com/search`
  with `q` and `mkt=zh-CN`, in the sequence DDG Lite → DDG Instant Answer →
  DDG HTML → Bing CN. `bing_parser.rs` parses the existing bounded Bing HTML form.
- `https://www.bing.com/search` is an **unverified additional endpoint candidate**;
  `bing.cn` is a **hypothetical alias/redirect**, not a tested approved query
  destination. Each new host, redirect and request form would require separately
  reviewed prepared endpoint authority under ADR-090.
- [Browser SERP / #891](https://github.com/wjhuang88/talos/issues/891) is a
  different, blocked approach: `BrowserPageRecord.selected_links` cannot be
  safely converted into organic results. D3 uses neither it nor unresolved CDP #452.

## D3.1 deliverable — source and offline design, no behavior change

1. Pin currently authoritative Microsoft Bing consumer terms and API retirement,
   explain SSR technical plausibility and unresolved distributed-product/result-use
   rights. Record **PASS / BLOCKED / UNKNOWN**, not unsupported approval.
2. Inventory existing fixed endpoint, parser, protected route, permission, proxy,
   query disclosure, redirect and candidate result-admission ownership.
3. Define an offline-only test plan, including organic-vs-ad, challenge, empty,
   403/429, locale/encoding/DOM drift, user consent, result-source and non-query probes.
4. Define a future conditional design and measured gate: one Bing family, independently
   tracked endpoint *variants*, observed query success rather than country/GeoIP,
   and no additional default requests without authorization.
5. Specify how a real controlled mainland-China plus independent comparison-network
   study would be conducted **if** rights and effective claim permit it.
6. Publish a bounded decision and independently owned next step or BLOCKED disposition.

## Exclusions and blocking dependencies

- No real automated Bing SERP request, traffic probe, scraping, reverse engineering
  of challenge behavior or extraction of real response bodies at this intake stage.
- No new SearchBackend identity or second-provider claim. `www.bing.com`,
  `cn.bing.com`, and any verified `bing.cn` alias are one Bing-family candidate.
- No Auto priority change, production code, synthetic test code, dependency/config,
  public `web_search` schema, request-source expansion, permission/budget/redirect
  relaxation, startup traffic, query logging, GeoIP routing or provider admission.
- No alteration to accepted D/#644 ZERO-ADMISSION, D2/#848, browser #891,
  complete H/K/L or blocked E/F/G/I. Parent #624 stays Open.

Microsoft's **2026-09-30** effective Services Agreement describes Bing/MSN
materials as personal/noncommercial unless otherwise authorized or legally
permitted. This is not a legal opinion. Default distributed Talos use and returned
result/snippet/model-context rights remain **BLOCKED / UNPROVEN** for a new route;
an HTTP 200 or robots allow decision alone is insufficient. A separately effective
claim and explicit rights decision are mandatory before live acquisition.

## Decision links and required reads

- [D3 source/protocol/fixture matrix](../../reference/SEARCH-001-D3-BING-DIRECT-HTML-2026-11-13.md).
- [ADR-085](../../decisions/085-talos-owned-search-routing-boundary.md),
  ADR-090, ADR-072, NET-001/#199; [D final disposition](../../reference/SEARCH-001-D-FINAL-DISPOSITION-2026-11-01.md).
- `AGENTS.md`; `docs/sop/REQUIREMENT-INTAKE.md`;
  `docs/sop/AGENT-COLLABORATION.md`; `docs/sop/START-ITERATION.md`;
  `docs/sop/CHANGE-CONTROL.md`.
- `crates/talos-tools/src/search_backend/bing_parser.rs`,
  `crates/talos-tools/src/search_backend/legacy_owned.rs`,
  `crates/talos-tools/src/search_backend/transport/compatibility/chain.rs`,
  `docs/reference/SEARCH-001-C-COMPATIBILITY-REQUESTS.md`.
- [Microsoft terms, effective 2026-09-30](https://www.microsoft.com/en/servicesagreement);
  [Microsoft Bing SSR](https://blogs.bing.com/search-quality-insights/2022/8/Fast-Front-End-Performance-for-Microsoft-Bing/);
  [Bing API retirement](https://learn.microsoft.com/lifecycle/announcements/bing-search-api-retirement).

## Separate test-only follow-up — SEARCH-001-D3-T1 / #941

D3.1 documentary research [PR #938](https://github.com/wjhuang88/talos/pull/938) merged at `8e32fcadb5698e8a5cf35affd0a064ba8d02daf4` after exact-head CI43424338312 and independent no-blocker review. [SEARCH-001-D3-T1](SEARCH-001-D3-T1-bing-synthetic-negative-conformance.md) / [#941](https://github.com/wjhuang88/talos/issues/941) / [I326](../../iterations/I326-search-bing-synthetic-negative-conformance.md) is a **separately governed test-only slice**: claim #943 merged `2f43e7d7`; implementation [#944](https://github.com/wjhuang88/talos/pull/944) merged Completion Commit `9cbc615cebb119ead9706a4d00159d7151639606` with five synthetic negatives, exact-head CI43428872719 8/8 and independent review. Its owner-first closeout follows; sponsored `li.b_algo` false-positive is characterized **not fixed**. D3 research owner remains **Intake / Proposed / Unclaimed**: new live Bing service/use rights, redirects, real mainland-China/other-region qualification and independent-provider admission remain blocked/unproven. No E/F/G/I activation follows.

## Separate parser-local correction candidate — SEARCH-001-D3-T2 / #947

T1/#941 was **Complete / Closed** after implementation #944 `9cbc615cebb119ead9706a4d00159d7151639606` and owner-first #946 `73ec650b6ef9804d0c9c42a6f1c5e7c9d9f0baa7`. Its synthetic test exposed a specific explicit `b_ad` ancestor containing `li.b_algo` falsely treated as an organic result. [D3-T2 / #947](SEARCH-001-D3-T2-bing-sponsored-ancestor-rejection.md) and [I327](../../iterations/I327-search-bing-sponsored-ancestor-rejection.md) then received effective claim #948 (`5704bb66aca22919347705a5bff40a84d641c3c9`) and a narrowly scoped **accepted parser correction** in [PR #949](https://github.com/wjhuang88/talos/pull/949), Completion Commit `5b2fa4358b83698ca4c0e517ce24a6dbba021d99`. Exact-head CI43465355386 second attempt 8/8 SUCCESS, independent source review and CAS; owner-first closeout [PR #950](https://github.com/wjhuang88/talos/pull/950) merged `6c5505abdc8d964ec39c01d221e9e3297a3458cf` after final docs CI43731465216 SUCCESS, independent review and CAS; [#947](https://github.com/wjhuang88/talos/issues/947) **Closed / Completed**. The correction recognizes **only explicit `b_ad` ancestor tokens**, not all real Bing sponsored formats. This does **not** change D3's **Intake / Proposed / Unclaimed** research status, query hosts, Microsoft usage/result rights, live mainland-China evidence, independent provider count or E/F/G/I admission. Epic #624 remains Open.

## Rights-gate reconciliation — 2026-11-15 (research only)

Following completed child D3-T2/#947, the parser's explicit `b_ad` ancestor
false-positive is fixed, but **this does not clear Bing's service-use gate**.
The [2026-11-15 vendor-source reconciliation](../../reference/SEARCH-001-D3-BING-DIRECT-HTML-2026-11-13.md#7-rights-and-next-action-recheck--2026-11-15)
confirms the Bing/MSN Materials clause in the Microsoft Services Agreement
(published 2026-07-30, effective 2026-09-30) still limits unlicensed
commercial/product-building reuse; the old Bing Search APIs were retired
2025-08-11. Microsoft-hosted Foundry/Web Search, Grounding with Bing and
Web IQ have distinct commercial service/terms and output contracts; none is
evidence of a free, raw HTML request grant for Talos. **BLOCKED / UNPROVEN**
for new automated default HTML use and result/display/model-context rights;
**UNKNOWN** for live www/bare alias redirects and regional performance.
These are product-use evidence gates, not an assertion that all individual
manual browsing is forbidden or a legal opinion.

**Next executable decision:** obtain supplier/rightsholder or qualified legal
determination covering (a) user-initiated automatic queries from a distributed,
zero-key OSS client to each proposed host; (b) user-facing titles/snippets/URLs,
attribution, cache and LLM-context use; (c) permitted frequency/redirect
handling and geographic applicability. Record permission in writing with
scope/version, otherwise keep BLOCKED. Only after affirmative rights, create
a *separate claimed work slice* for a bounded, consented live query/redirect
study from genuine mainland-China and independent comparator networks,
with prepared per-hop authority, Network permission, privacy redaction and
existing budget. No license decision, host test or new source has occurred here.

This closes no part of D3/#937 or Epic/#624, changes no code, tests, endpoint,
default route or grant, does not reopen closed #947 and preserves #644's
historical BLOCKED / ZERO-ADMISSION disposition.

## Acceptance (documentary only)

- [x] Existing implementation, historical D3 overlap and rights ambiguity identified,
  without claiming live or Chinese-network proof.
- [x] Multi-surface same-family design and offline test plan published with source dates.
- [x] Blocking gate, rollback and negative permissions/default-route implications explicit.
- [x] Documentary PR #938 reviewed and exact-head CI43424338312 accepted (merge `8e32fcadb5698e8a5cf35affd0a064ba8d02daf4`); no implementation
  `Completion Commit` claimed for research publication.
- [ ] Any later offline executable tests, authorized live observation or production change
  separately selected, claimed, reviewed and tested. This checkbox does not block
  publishing the intake; it blocks upgrading the technical decision.

## State/status owners and rollback

This document owns D3 intake, #937 owns discussion, the
[SEARCH-001 Epic](SEARCH-001-zero-config-global-search.md) owns final product
acceptance, `docs/backlog/PRODUCT-BACKLOG.md` is derived inventory and the
Issue reconciliation matrix records remote status. Rollback of this documentary
proposal requires only removal/reversion of its new documents and references.
No runtime or user configuration migration is performed.

## Maintainer implementation direction — 2026-11-16 (D3-T3 / #957)

[SEARCH-001-D3-T3 / #957](SEARCH-001-D3-T3-bing-dual-host-zero-key-route.md)
records the **desired zero-key Bing HTML product design**: within the same Bing family,
attempt `www.bing.com/search` (non-CN) first and use current `cn.bing.com/search`
only for correctly classified, permitted *unavailability*. Bare `bing.cn` stays
unadmitted. Existing higher-level DDG Lite → Instant Answer → DDG HTML → Bing order
is **not** changed by this endpoint-variant preference.

Its proposed [I329](../../iterations/I329-search-bing-dual-host-offline.md)
claim covers **offline-only** failover decision logic and tests, not a new
Network endpoint, live request, distribution, route switch, Browser/CDP integration,
rights grant or product-default activation. In particular no 429/challenge/403
fallback is allowed as an access-control evasion. New `www` HTTP traffic still needs
written applicable access **and output-use rights**, independently accepted
ADR-090 authority/budget, actual permitted mainland-China/comparator evidence,
separate effective source/activation claim, exact-head CI and security/API review.
D3/#937 stays Intake / Unclaimed; Epic #624 stays Open, D/#644 ZERO-ADMISSION and
E/F/G/I are unchanged.

## Maintainer technical GO and separately owned I333 handoff — 2026-11-20

The maintainer explicitly chooses **technical feasibility** as the GO criterion for the existing Bing HTML approach, per [#937](https://github.com/wjhuang88/talos/issues/937#issuecomment-7029524685) and [#957](https://github.com/wjhuang88/talos/issues/957#issuecomment-7029525278). The existing accepted `bing_parser.rs`, protected search transport, T1/T2 fixtures and I329/T3a fail-closed variant policy suffice to proceed with **bounded engineering** rather than repeat documentary feasibility research. Preferred same-family route remains `www.bing.com/search` first and `cn.bing.com/search` only on permitted typed source-local unavailability. The DDG-first production-family order remains intact.

New independent [D3-T4 / #991](SEARCH-001-D3-T4-bing-prepared-dual-host-integration.md) and [I333](../../iterations/I333-search-bing-prepared-dual-host-integration.md) propose **offline/mocked prepared-request and permission-aware integration** under governance [PR #992](https://github.com/wjhuang88/talos/pull/992). Claim and activation are ineffective until #992 merges; no T4 source change or newly activated Bing request originates from this documentary decision. Already-closed I329/T3a and historical #644 ZERO-ADMISSION remain unchanged.

**Fact separation:** technical GO is not a Microsoft access/reuse license, evidence of measured live `www` operation, or mainland-China/comparator reachability. #957 T3b live/default activation and external rights/real-region evidence remain separately unverified; no E/F/G/I admission or Epic #624 closure follows. Do not bypass access controls, 403/429/CAPTCHA/5xx restrictions, per-host prepared Network authority, or invocation budgets.

## I333 D3-T4 completion — 2026-11-22 (offline-only)

The separately owned [D3-T4](SEARCH-001-D3-T4-bing-prepared-dual-host-integration.md) / [#991](https://github.com/wjhuang88/talos/issues/991) / [I333](../../iterations/I333-search-bing-prepared-dual-host-integration.md) integrated the accepted I329 Bing WWW→CN variant policy into a **private, non-production request plan with offline mocked execution**, source [PR #994](https://github.com/wjhuang88/talos/pull/994) merged as **Completion Commit `c3b75e1e118dff37f29d7f028c99ac2336ba7b59`**, exact-head [CI #4483](https://github.com/wjhuang88/talos/actions/runs/44868769900) attempt 4 **8/8 SUCCESS** and [independent Agent APPROVE](https://github.com/wjhuang88/talos/pull/994#issuecomment-7075229507). D3-T4 closes only upon owner-first governance publication. D3/#937 itself remains **Open**; real Bing network deployment, supplier automated search/result reuse rights, region observations and T3b/#957 live/default remain **unverified / BLOCKED**; no added provider independence and historical #644 ZERO-ADMISSION remain unchanged.
