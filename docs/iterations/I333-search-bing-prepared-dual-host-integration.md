# Iteration I333: Prepared Bing Dual-Host Technical Integration (Offline)

> Document status: Complete / Closed for I333 private, offline-only integration upon owner-first closeout merge; NOT a live/default Bing route
> Published plan date: 2026-11-20
> Planned objective: Integrate the accepted Bing WWW-first/CN-fallback pure decision into a safely prepared, mocked end-to-end request/permission execution seam, WITHOUT live/default traffic activation.
> Baseline rule: once committed, preserve this objective; change scope via a new iteration.
> MVP deliverable: Runnable native Rust deterministic mock tests proving both hosts were prepared before execution, only typed outages permit fallback, and the deployed CN-only/DDG-first route stays unchanged.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | I333 / SEARCH-001-D3-T4/#991 — inactive, offline-only prepared dual-host Bing integration + deterministic mock execution; not #957 live/default T3b |
| Claimed At | 2026-11-20 |
| Source Issue | #991 |
| Governance Claim PR | #992 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Effective governance #992 merged `510bfa71a43a37a8273c58df82450f233cbebb11`; exact-head CI #4483 attempt 4 8/8 SUCCESS; independent Agent [APPROVE](https://github.com/wjhuang88/talos/pull/994#issuecomment-7075229507); single-maintainer CAS and source merge #994 `c3b75e1e118dff37f29d7f028c99ac2336ba7b59`. No independent human approval claimed. |
| Implementation PR | #994 merged at Completion Commit `c3b75e1e118dff37f29d7f028c99ac2336ba7b59` |
| Last Updated | 2026-11-22 |
| Handoff / Release Condition | I333 offline-only Complete/Closed upon owner-first merge; #957 live/default and #937/#624 external rights/region gates remain separate and blocked/open |

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
| 2026-11-20 | Draft selection (historical) | Proposed I333 technical GO under #991, subsequently activated by #992 source-independent claim merge. |
| 2026-11-20 | Effective claim | #992 merged `510bfa71a43a37a8273c58df82450f233cbebb11`, valid before source work. |
| 2026-11-22 | Accepted source | #994 merged as existing source Completion Commit `c3b75e1e118dff37f29d7f028c99ac2336ba7b59` after exact-head 8/8 native CI, Agent APPROVE and CAS. |
| 2026-11-22 | Owner-first closeout | Canonical D3-T4 owner updated before iteration, Board, backlog, parent and Issue #991. Status effective upon closeout merge. |

## Verification evidence

Nine offline synthetic Rust tests in [#994](https://github.com/wjhuang88/talos/pull/994); [CI #4483 attempt 4](https://github.com/wjhuang88/talos/actions/runs/44868769900) **8/8 SUCCESS** across Windows workspace/Rust gate, macOS and Linux/Windows Desktop plus governance. [Independent Agent APPROVE](https://github.com/wjhuang88/talos/pull/994#issuecomment-7075229507) no blockers; old red/cancelled attempts retained. #141 tracks unrelated WAL contention.

## Completion evidence

**Completion Commit: `c3b75e1e118dff37f29d7f028c99ac2336ba7b59`** ([#994](https://github.com/wjhuang88/talos/pull/994) merged on main; exact source HEAD `bcc1014186d0ae883da7474c93a1597b989f9988`). The status-only closeout commit is not its own Completion Commit. Offline-only I333 is Complete/Closed upon owner-first merge, **not** #957 T3b production activation.

## Variance and residuals

#957 T3b real/default traffic activation remains separately gated; #937/#624 Open; no Microsoft usage rights, region observations, native provider independence or E/F/G/I acceptance inferred.
