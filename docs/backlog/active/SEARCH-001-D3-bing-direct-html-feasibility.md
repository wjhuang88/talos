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
| Last Updated | 2026-11-13 |
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

T1/#941 was **Complete / Closed** after implementation #944 `9cbc615cebb119ead9706a4d00159d7151639606` and owner-first closeout #946 `73ec650b6ef9804d0c9c42a6f1c5e7c9d9f0baa7`. Its synthetic test records an explicit sponsored ancestor `b_ad` containing `li.b_algo` incorrectly yielded as an organic candidate. [D3-T2 / #947](SEARCH-001-D3-T2-bing-sponsored-ancestor-rejection.md) and [I327](../../iterations/I327-search-bing-sponsored-ancestor-rejection.md) propose a **separate, narrowly scoped parser correction**, with no new query host, network/default/permission or generic sponsor-classification heuristics. This registration has no effective source implementation claim until a reviewed governance PR merges. It does not amend D3's **Intake / Proposed / Unclaimed** research status or Microsoft's outstanding use-rights, live regional, second-provider and E/F/G/I gates. Epic #624 remains Open.

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
