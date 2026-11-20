# SEARCH-001-D3-T4: Prepared Dual-Host Bing Technical Integration (Offline)

**Status**: Refinement / Unclaimed — technical GO only; activation ineffective until governance PR merge
**Type**: Technical / Test Story
**Parent Epic**: [SEARCH-001 / #624](SEARCH-001-zero-config-global-search.md)
**Source Issue**: [#991](https://github.com/wjhuang88/talos/issues/991)
**Predecessor**: [SEARCH-001-D3-T3 / #957](SEARCH-001-D3-T3-bing-dual-host-zero-key-route.md), I329 T3a Complete / Closed
**Selected Iteration**: [I333](../../iterations/I333-search-bing-prepared-dual-host-integration.md) — proposed, inactive until claim merge
**Technical decision**: Maintainer [2026-11-20 Bing GO on technical feasibility](https://github.com/wjhuang88/talos/issues/957), distinct from permission to distribute automated search use

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | D3-T4/#991: offline, prepared, per-invocation permission-aware Bing dual-variant request-plan integration and mock execution; **no newly active network host or production default route** |
| Claimed At | Not applicable |
| Source Issue | #991 |
| Governance Claim PR | Pending — Draft only |
| Authorization Mode | Single-maintainer merge (proposed; ineffective until reviewed claim merge) |
| Authorization Evidence | Maintainer technical GO decision in #957/#937; final exact-head CI, independent Agent review, and merge CAS still required |
| Implementation PR | Not started |
| Last Updated | 2026-11-20 |
| Handoff / Release Condition | Claim only effective on target-main merge; subsequent implementation must be independently reviewed and measured; #957 T3b live/default remains unclaimed and #624 Open |

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

- [ ] Effective distinct I333/owner claim recorded on target `main` via governance-only PR before any implementation.
- [ ] Offline end-to-end mocks prove bounded WWW-first/CN-fallback using *both preauthorized* variants; fail closed on unprepared host, foreign redirect, access denial, HTTP 5xx, stale invocation and budget exhaustion.
- [ ] Existing DDG-first Auto production route, schema, snapshots and permission host list remain unchanged; compare code + fixture output.
- [ ] No new live calls/keys or dependency; locked focused/full Rust tests and applicable macOS/Linux/Windows CI, security/API/Network independent review and CAS.
- [ ] After accepted source implementation, owner-first status closeout cites **prior source Completion Commit**. Keep actual live/default, region, rights, provider-independence residuals open.

## Required reads

`AGENTS.md`, `docs/sop/REQUIREMENT-INTAKE.md`, `docs/sop/AGENT-COLLABORATION.md`, `docs/sop/START-ITERATION.md`, `docs/decisions/085-talos-owned-search-routing-boundary.md`, `docs/decisions/090-invocation-bound-search-endpoint-authority.md`, `crates/talos-tools/src/search_backend/bing_variants.rs`, `crates/talos-tools/src/search_backend/bing_parser.rs`, `crates/talos-tools/src/search_backend/transport/compatibility/{chain,plan}.rs`, existing T3a/I329 owner and #959.
