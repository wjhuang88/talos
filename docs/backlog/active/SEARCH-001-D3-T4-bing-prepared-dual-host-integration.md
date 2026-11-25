# SEARCH-001-D3-T4: Prepared Dual-Host Bing Technical Integration (Offline)

**Status**: Complete / Closed for I333 offline-only technical integration upon owner-first closeout merge; production Bing WWW remains NOT ACTIVATED
**Type**: Technical / Test Story
**Parent Epic**: [SEARCH-001 / #624](SEARCH-001-zero-config-global-search.md)
**Source Issue**: [#991](https://github.com/wjhuang88/talos/issues/991)
**Predecessor**: [SEARCH-001-D3-T3 / #957](SEARCH-001-D3-T3-bing-dual-host-zero-key-route.md), I329 T3a Complete / Closed
**Selected Iteration**: [I333](../../iterations/I333-search-bing-prepared-dual-host-integration.md) — offline technical slice Complete / Closed upon owner-first closeout merge; claim #992 already effective
**Technical decision**: Maintainer [2026-11-20 Bing GO on technical feasibility](https://github.com/wjhuang88/talos/issues/957), distinct from permission to distribute automated search use

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | D3-T4/#991: offline, prepared, per-invocation permission-aware Bing dual-variant request-plan integration and mock execution; **no newly active network host or production default route** |
| Claimed At | 2026-11-20 |
| Source Issue | #991 |
| Governance Claim PR | #992 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Technical GO #957/#937; claim #992 merged `510bfa71a43a37a8273c58df82450f233cbebb11` before source; #994 exact-head CI #4483 attempt 4 **8/8 SUCCESS**, independent Agent [APPROVE](https://github.com/wjhuang88/talos/pull/994#issuecomment-7075229507), single-maintainer authorization + merge-time CAS. No independent human approval claimed. |
| Implementation PR | #994 MERGED, Completion Commit `c3b75e1e118dff37f29d7f028c99ac2336ba7b59`; source HEAD `bcc1014186d0ae883da7474c93a1597b989f9988` |
| Last Updated | 2026-11-22 |
| Handoff / Release Condition | I333 source accepted and closed only upon owner-first status publication; #991 closes after closeout merge. #957 T3b remains BLOCKED / Unclaimed; #937/#624 Open; external rights/region still unverified. |

## Goal and technical boundary

Accept Bing HTML + existing owned parser as a **GO for engineering**; finish the missing private integration between I329's already accepted inert `bing_variants.rs` policy and ADR-090 prepared request/authority/transport with deterministic, isolated mock tests. No further research spike needed to decide whether Bing is a plausible HTML search provider; however engineering evidence is **not** a Microsoft rights license nor genuine mainland-China/comparator reachability observation.

The existing deployed source-family sequence remains DDG Lite → Instant Answer → DDG HTML → Bing CN. Keep the single compound Bing family and existing public `web_search` schema/format/permission unchanged.

## Scoped implementation

1. Prepare BOTH precise search hosts and immutable query-bearing request forms **before** approval/execution, including proxy/redirect constraints and per-host Network facets. Deny missing scope; never discover/authorize CN only after a WWW failure. Preserve same single invocation deadline and transport budget.
2. Wire `first_variant` / `next_variant` into a private, non-production-wired testable request/execution seam that reuses owned parser, preparation, transport and result admission. Preserve typed source-local DNS/connect/refusal/timeout only; never infer host outage from status/HTML ambiguity.
3. Run deterministic isolated mocks: WWW organic success; authorized typed-source failure → CN; CN rejection; 200 empty; 302/foreign hop; HTTP 403/429/all 5xx; CAPTCHA/challenge; malformed/encoding/DOM drift; Network/proxy denial; cancellations/deadline/budget; no host fallback on rate/access restriction. Include negative permission/redirect checks and no query text in logs.
4. Keep no default Auto registration or new production-origin request in this Story; avoid public API changes and new dependency. Provide targeted rollback evidence and document technical-only outcomes and the residual live/default switch.

## Hard non-goals and dependencies

- No live Bing query or even a new production URL admission; no product default activation, user cookies, anti-automation bypass, GeoIP, startup probe, new Browser/CDP, paid API, generic retry/circuit-breaker or separate Bing provider ID.
- Microsoft automated-access and result/LLM-context use rights, plus actual China/comparator measurements, are **unverified** separate gates. Technical GO does not invent those facts. #957 T3b (live/default) and #937/#624 remain Open; historical D/#644 ZERO-ADMISSION is untouched. E/F/G/I are not unlocked.
- Reuse ADR-[085](../../decisions/085-talos-owned-search-routing-boundary.md), [090](../../decisions/090-invocation-bound-search-endpoint-authority.md), ADR-072 and existing protected permission boundary.

## Acceptance and verification

- [x] Effective distinct I333/owner claim recorded on target `main` via governance-only PR before any implementation.
- [x] Offline end-to-end mocks prove bounded WWW-first/CN-fallback using *both preauthorized* variants; fail closed on unprepared host, foreign redirect, access denial, HTTP 5xx, stale invocation and budget exhaustion.
- [x] Existing DDG-first Auto production route, schema, snapshots and permission host list remain unchanged; compare code + fixture output.
- [x] No new live calls/keys or dependency; locked focused/full Rust tests and applicable macOS/Linux/Windows CI, security/API/Network independent review and CAS.
- [x] After accepted source implementation, owner-first status closeout cites **prior source Completion Commit**. Keep actual live/default, region, rights, provider-independence residuals open.

## Completion and native verification — 2026-11-22

**Completion Commit: `c3b75e1e118dff37f29d7f028c99ac2336ba7b59`** (pre-existing `main` source merge for [implementation PR #994](https://github.com/wjhuang88/talos/pull/994); reviewed exact source HEAD `bcc1014186d0ae883da7474c93a1597b989f9988`). This is a source-implementation commit, **not** the governance closeout commit.

- Claim-first: [#992](https://github.com/wjhuang88/talos/pull/992) was merged before implementation. Review: [independent Agent APPROVE](https://github.com/wjhuang88/talos/pull/994#issuecomment-7075229507) confirms no source/API/Network/security blockers; no separate natural-person approval claimed. Single-maintainer authorization and target CAS recorded in #994.
- Private integration: both fixed Bing WWW and CN search plans preauthorized under one ToolNetworkManifest before callbacks, zero redirects and one header snapshot; 9 offline synthetic Rust tests exercise parser/admission, typed DNS/connect/refusal/source-local-timeout failover and fail-closed Network/HTTP/redirect/denied host/cancel/deadline/shared request+response budget cases. No new production sender, outbound HTTP/DNS, provider identity or Network permission.
- Native validation: [CI #4483 attempt 4](https://github.com/wjhuang88/talos/actions/runs/44868769900) **SUCCESS, all 8 applicable jobs** including macOS, Windows workspace and Rust aggregate, Windows/Linux Desktop, governance checks. Earlier failing/cancelled attempts are retained, not waived; unrelated [Windows SQLite WAL contention diagnostic](https://github.com/wjhuang88/talos/issues/141#issuecomment-7067750909) transferred to DATA-002/#141.
- Rollback: revert the two isolated `talos-tools/search_backend/transport/compatibility` Rust file changes. Shipped DDG Lite → Instant Answer → DDG HTML → Bing CN behavior stays unchanged; no public Tool changes or new dependencies.
- Residual separate gates: [#957](https://github.com/wjhuang88/talos/issues/957) **T3b production/live BLOCKED / Unclaimed**, [D3/#937](https://github.com/wjhuang88/talos/issues/937) and [epic #624](https://github.com/wjhuang88/talos/issues/624) Open, historical #644 ZERO-ADMISSION unchanged. Microsoft automated-search/result/LLM use rights and mainland-China/comparator reachability remain **unverified**; E/F/G/I not unlocked.

## Required reads

`AGENTS.md`, `docs/sop/REQUIREMENT-INTAKE.md`, `docs/sop/AGENT-COLLABORATION.md`, `docs/sop/START-ITERATION.md`, `docs/decisions/085-talos-owned-search-routing-boundary.md`, `docs/decisions/090-invocation-bound-search-endpoint-authority.md`, `crates/talos-tools/src/search_backend/bing_variants.rs`, `crates/talos-tools/src/search_backend/bing_parser.rs`, `crates/talos-tools/src/search_backend/transport/compatibility/{chain,plan}.rs`, existing T3a/I329 owner and #959.
