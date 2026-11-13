# SEARCH-001-D3: Bing Direct-HTML Multi-Endpoint Feasibility

**Status**: In Progress / Claimed — proposed D3.2 test-only claim via governance #942, **ineffective until that PR merges**. D3.1 documentary intake already merged (#938, `8e32fcadb5698e8a5cf35affd0a064ba8d02daf4`); live endpoint/provider admission remains blocked.

| Field | Value |
|---|---|
| Story ID | SEARCH-001-D3 |
| Type | Research Spike / evidence and feasibility gate |
| Parent Epic | [SEARCH-001 / #624](SEARCH-001-zero-config-global-search.md) |
| Source Issue | [#937](https://github.com/wjhuang88/talos/issues/937) |
| Predecessor | [SEARCH-001-D / #644](SEARCH-001-D-provider-evidence-gate.md), Complete / Closed **BLOCKED / ZERO-ADMISSION** |
| Sibling | [SEARCH-001-D2 / #848](SEARCH-001-D2-keyless-mcp-admission.md), independent unclaimed Exa/Parallel hosted MCP research |
| Selected Iteration | [I326](../../iterations/I326-search-bing-synthetic-characterization.md) — proposed, not effective until governance claim merge |
| Evidence and acceptance | [D3 Bing direct HTML source/contract packet, 2026-11-13](../../reference/SEARCH-001-D3-BING-DIRECT-HTML-2026-11-13.md) |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | D3.2 only: inventory BF-01..BF-13 against existing Bing parser/compatibility tests and add missing *test-only* synthetic HTML and local/loopback negative characterization (organic/ad, valid/empty/challenge, denied redirect, root-vs-query probe, cancellation/permission and source identity); publish evidence; no runtime, new endpoints, real Bing HTTP, permissions or provider admission. |
| Claimed At | 2026-11-13 |
| Source Issue | #937 |
| Governance Claim PR | #942 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | User repeatedly requested continuation of D3 development following accepted #938; this test-only, non-overlapping I326 is proposed via #942. Exact-head CI, two governance validators, independent read-only review and final CAS must pass before target-branch claim/activation. |
| Implementation PR | Not started — D3.2 requires effective claim first |
| Last Updated | 2026-11-13 |
| Handoff / Release Condition | D3.2 synthetic-only test code starts only after effective governed claim; new live automated Bing SERP acquisition, endpoint additions, default routing or provider admission remain blocked on separately resolved use/output rights, ADR-090 and independent security/API approval. |

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

## Acceptance (documentary only)

- [x] Existing implementation, historical D3 overlap and rights ambiguity identified,
  without claiming live or Chinese-network proof.
- [x] Multi-surface same-family design and offline test plan published with source dates.
- [x] Blocking gate, rollback and negative permissions/default-route implications explicit.
- [x] Documentary D3.1 [PR #938](https://github.com/wjhuang88/talos/pull/938) merged `8e32fcadb5698e8a5cf35affd0a064ba8d02daf4` after exact-head [CI43424338312](https://github.com/wjhuang88/talos/actions/runs/43424338312) success and independent read-only no-blocker review; no implementation Completion Commit claimed.
- [ ] Any later offline executable tests, authorized live observation or production change
  separately selected, claimed, reviewed and tested. This checkbox does not block
  publishing the intake; it blocks upgrading the technical decision.

## D3.1 publication checkpoint — 2026-11-13

Documentation-only intake #938 is merged as `8e32fcadb5698e8a5cf35affd0a064ba8d02daf4`, with exact-head CI43424338312 successful and no-blocker source/ADR/governance review. This is **not** D3 provider admission, executable test evidence, genuine Bing endpoint observation or mainland-China quality acceptance. Issue #937 remains Open. The D3.1 documentary decision stays BLOCKED for new automated distributed-default service/result use.

## D3.2 proposed bounded work slice — test-only, NOT effective until governance merge

[Iteration I326](../../iterations/I326-search-bing-synthetic-characterization.md) proposes **one independently testable offline characterization increment**, separate from D3.1 and from E/F/G/I. The scope is *only* Talos-authored synthetic responses and in-memory/loopback controlled transport tests against already accepted contracts; **no actual Bing query, contact with `www.bing.com`/`cn.bing.com`/`bing.cn`, or captured upstream markup**.

1. **Inventory first**: compare BF-01..BF-13 against existing tests in `search_backend/bing_parser/tests.rs`, `transport/compatibility` and prepared authority fixtures. Reuse existing selectors/count/status/encoding/limits coverage rather than duplicating it. Publish explicit covered/uncovered/blocked evidence.
2. **Fill only uncovered negative test gaps**: synthetic ad/navigation decoys that must not be mistaken for organic `li.b_algo` rows, 200 valid-vs-empty-vs-challenge classification, local response and redirect-host denial before permitted send, root-reachable-vs-query-unusable health *as characterization* (without production probes or health routing changes), cancellation and source identity protection.
3. **Strict boundary**: changes confined to `#[cfg(test)]` modules or test-only fixtures and evidence docs; do not change runtime parser, `SearchRouter`, URL allowlist, manifest, host selection, query order, permissions, remote provider, Cargo, dependencies or model-facing schema. A failing test demonstrating an existing limitation may remain documented/owned; do not silently redesign production behavior to make it pass.
4. **Validation**: repository-pinned `cargo fmt --all -- --check`, `cargo test -p talos-tools --features network --locked` with focused filters (or current correct Cargo commands), relevant check/Clippy/workspace tests, native applicable CI, governance validators and independent exact-head review. Record exact executed test counts and pre-existing unrelated failures transparently.
5. **Decision**: publish proof of covered local behavior, unresolved parser false-positive/health-model risks and any future remediation as a separate authorized scope; do not treat synthetic tests as proof of Bing terms, live SERP compatibility, server reachability or a second independent zero-key provider.

**Activation prerequisite:** D3.1 #938 is accepted, but the I326 claim is merely proposed until the governance-only PR containing a complete `Claim State: Claimed`, `Active` iteration and exact PR number merges into `main`. No code/test implementation before then. If the governance claim is rejected, D3 remains Ready/Unclaimed and the existing baseline is unchanged.

## State/status owners and rollback

This document owns D3 intake, #937 owns discussion, the
[SEARCH-001 Epic](SEARCH-001-zero-config-global-search.md) owns final product
acceptance, `docs/backlog/PRODUCT-BACKLOG.md` is derived inventory and the
Issue reconciliation matrix records remote status. Rollback of this documentary
proposal requires only removal/reversion of its new documents and references.
No runtime or user configuration migration is performed.
