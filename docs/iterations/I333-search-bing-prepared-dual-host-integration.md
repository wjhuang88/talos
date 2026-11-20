# Iteration I333: Prepared Bing Dual-Host Technical Integration (Offline)

> Document status: Active / Claimed (PROPOSED; effective only after #992 is accepted and merged into main)
> Published plan date: 2026-11-20
> Planned objective: Integrate the accepted Bing WWW-first/CN-fallback pure decision into a safely prepared, mocked end-to-end request/permission execution seam, WITHOUT live/default traffic activation.
> Baseline rule: once committed, preserve this objective; change scope via a new iteration.
> MVP deliverable: Runnable native Rust deterministic mock tests proving both hosts were prepared before execution, only typed outages permit fallback, and the deployed CN-only/DDG-first route stays unchanged.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | I333 / SEARCH-001-D3-T4/#991 — inactive, offline-only prepared dual-host Bing integration + deterministic mock execution; not #957 live/default T3b |
| Claimed At | 2026-11-20 |
| Source Issue | #991 |
| Governance Claim PR | #992 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer #937/#957 technical GO; PR #992 atomic owner/iteration claim (ineffective before target merge), exact-head CI and governance validators, independent Agent review, expected-head CAS required; no distinct human approver claimed. |
| Implementation PR | Not started |
| Last Updated | 2026-11-20 |
| Handoff / Release Condition | Claim effective only after reviewed governance merge; source may then start, live/default activation remains separate |

## Published baseline

### Selected Story / dependency

| Story | Parent | Status at selection | Depends on | Outcome |
|---|---|---|---|---|
| [SEARCH-001-D3-T4 / #991](../backlog/active/SEARCH-001-D3-T4-bing-prepared-dual-host-integration.md) | SEARCH-001-D3/#937; SEARCH-001/#624 | New bounded technical Story, no effective claim | Accepted T3a/I329/#959, owned Bing parser and protected SearchTransport / ADR-090 | Private prepared request seam + offline mock execution proof |

### Iteration inventory and non-overlap

I288/I292 A/B, I293 C, I319 L, I326 T1, I327 T2, I329 T3a and I331 D2.2 are Complete/Closed in their own bounded scopes. I332 PROVIDER-008-D is independently owned in another workstream. I330 ARCH-035 E2, I324 GIT-COMPAT, I318/TUI and other existing Review/Planned tasks remain their own owners; they are not transferred or changed by I333. #957 **live/default** T3b remains Blocked/Unclaimed, #848 Exa/Parallel is independent and blocked, #624 remains Open. No overlapping open Bing PR found at selection.

### Scope, acceptance and tests

1. Build a private, test-invocable request plan with `www.bing.com/search` then `cn.bing.com/search` (one logical provider), both manifest-bound before dispatch.
2. Reuse `bing_variants.rs` exact failover guard matrix and source/local request budget. Tests must prove DNS/connect/refusal/source-local timeout-only fallback; **all** HTTP statuses (including 5xx), 403/429, CAPTCHA, redirect, parser/encoding error, valid-empty, denied scope, cancel/deadline/budget stop.
3. Preserve existing default Auto list, DDG-first order, proxy behavior, public `web_search` schema, Network permission profile and format; no production host enablement.
4. Locked `cargo fmt --check`, focused `talos-tools` tests, workspace check/Clippy/test, governance validators, applicable native CI and independent Network/security/API review. No live HTTP and no region-rights claims.

### Non-goals

No default WWW Bing call, new provider identity, new public API, Cargo dependencies, Browser/CDP, Gecko/Chromium, user-data retention, upstream gate bypass, mainland-China coverage assertion or E/F/G/I activation. Existing #957, #937 and #624 remain Open. Supplier access/output rights are unverified, not redefined by technical GO.

### Documentation to update

Owner [D3-T4](../backlog/active/SEARCH-001-D3-T4-bing-prepared-dual-host-integration.md), parent [D3](../backlog/active/SEARCH-001-D3-bing-direct-html-feasibility.md), SEARCH Epic, `docs/BOARD.md`, `docs/iterations/README.md`, and #991 status; mark technical progress vs unverified external gates accurately.

### Risks and rollback

Risk: fallbacks accidentally gain unapproved new host, extra request budget, reissued headers or a permission bypass. Roll back the isolated offline integration and retain accepted I329 policy and existing shipped CN-only path. No production route switch is authorized here.

## Actual activation/execution

| Date | Type | Record |
|---|---|---|
| 2026-11-20 | Draft selection | Proposed I333 technical GO under #991. Claim Pending is a GitHub condition only; owner/iteration stay Unclaimed until finalized governance PR exact-head reaches target main. |

## Verification evidence

Not started; this governance-only stage contains **no source or fixtures**.

## Completion evidence

Not available. Do not mark Complete before a separate actual source implementation is accepted and merged.

## Variance and residuals

#957 T3b real/default traffic activation remains separately gated; #937/#624 Open; no Microsoft usage rights, region observations, native provider independence or E/F/G/I acceptance inferred.
