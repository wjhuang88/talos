# Iteration I329: Bing Non-CN-First/CN-Fallback Offline Contract

> Status: **Active / Claimed** since governance PR #958 merged `58207831675365e35cc80601097b96f9432a68aa`; implementation review pending.
> Selected: 2026-11-16
> Story: [SEARCH-001-D3-T3 / #957](../backlog/active/SEARCH-001-D3-T3-bing-dual-host-zero-key-route.md)
> Parent: [D3/#937](https://github.com/wjhuang88/talos/issues/937) / [SEARCH-001/#624](https://github.com/wjhuang88/talos/issues/624)

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | D3-T3a/#957: deterministic offline non-CN-first/CN-fallback decision candidate and synthetic tests only. No live requests, production endpoints, Auto policy, Network grants or scope expansion. |
| Claimed At | 2026-11-16 |
| Source Issue | #957 |
| Governance Claim PR | #958 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | I329 activated by #958 merge `58207831675365e35cc80601097b96f9432a68aa`; exact-head CI #4273 4 applicable SUCCESS/4 skipped; independent no-blocker review #6907540766 and CAS. |
| Implementation PR | Candidate on `feat/i329-search-bing-offline-failover`; no accepted source PR yet |
| Last Updated | 2026-11-16 |
| Handoff / Release Condition | Implement and close only offline T3a after separately accepted claim; T3b production blocked pending rights/region/permission |

## Baseline and active-work inventory

SEARCH-001-C/#643 and L/#901 are Complete; D3-T1/#941 and D3-T2/#947 are Complete, while D3/#937, D2/#848, SEARCH Epic/#624 remain Open/Unclaimed and E/F/G/I unactivated. The actual existing compatibility chain is DDG Lite → Instant Answer → DDG HTML → **CN Bing**. Reuse existing parser, exact host request and prior source identity.

Parallel owned slices remain separate: I328/STORE-002 E activation, I325 D and I327 T2 closed; I324/GIT-COMPAT currently review/draft; I323 Azure, I318 TUI review and other Active/Planned/Blocked iterations remain their owners. This I329 source slice must not edit those owners or shared decisions; re-check active claims and changed paths before starting implementation. Existing obsolete K alternate #861 closed unmerged.

## Deliverable / offline acceptance

- Pure, isolated variant and failover policy for **`www.bing.com` first, `cn.bing.com` second** within one Bing family. No implicit country inference. Bare `bing.cn` not selected.
- Only typed DNS/connect establishment/refusal and source-local timeout with remaining invocation authority/budget qualify as source-unavailable. **All HTTP 5xx (500/502/503)** are **no-fallback** pending separately governed evidence; explicit denial/403/429, rate limits, CAPTCHA/challenge, unknown redirect, insufficient Network approval, stale/cancelled/budget-exhausted invocation and malformed/uncertain results also never fail over.
- Explicit tests distinguish valid 200 empty, parser drift, restriction responses including **500/502/503 all blocked**, and successful organic results. Same-family provenance; retained external chain and existing CN-only production behavior.
- Handwritten fixture tests and a compact reviewable rights/permission/rollback decision. No live query or assumed independent native provider.
- A later T3b activation requires independently cleared access+output rights, accepted per-invocation network/redirect authority, actual controlled China/comparator evidence, fresh native CI and independent review.

## Execution order and gates

1. This governance-only claim+iteration PR merges after exact-head docs CI and independent review; until then do **not** create source implementation branch.
2. Implement bounded offline candidate on current main; locally converge targeted tests and full locked validation; stage one reviewable implementation PR.
3. Native exact-head Windows/macOS/Linux CI; independent API/network/security review, permission/redirect negative controls, and source-head CAS merge.
4. Owner-first governance closeout with the **actual prior source Completion Commit**, then close the bounded T3a claim only; preserve parent open and T3b blocked.

## Explicit exclusions

No production `www` request, browser/CDP activation, new SearchBackend identity, public `web_search` schema, Cargo/lock changes, network manifest or budget widening, new proxy flow, startup probes, rate/challenge bypass, GeoIP selection, user cookies or secrets. No supplier/region/rights assertion. Rollback of T3a is isolated candidate/test revert; there is **no** production route switch to roll back.

## Claim activation and implementation checkpoint — 2026-11-16

Claim #958 merged at `58207831675365e35cc80601097b96f9432a68aa`, after [CI #4273](https://github.com/wjhuang88/talos/actions/runs/43805545051) and independent exact-head no-blocker review. No source implementation was started before that merge. The new branch includes only private inert Bing variant decision/tests and an evidence note; source CI, independent source security/API review and CAS **not yet accepted**. Production `www.bing.com` activation T3b remains BLOCKED, no new Network authority was granted.
