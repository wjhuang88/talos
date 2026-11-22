# SEARCH-001-D3-T5: Bing WWW-first/CN-fallback Real Transport Implementation

**Status**: Active / Claimed PROPOSED; effective only after governance PR #1002 merge, source work not yet started
**Type**: Bounded Rust implementation and offline verification
**Parent**: [SEARCH-001 / #624](SEARCH-001-zero-config-global-search.md); [D3 / #937](SEARCH-001-D3-bing-direct-html-feasibility.md)
**Source Issue**: [#1001](https://github.com/wjhuang88/talos/issues/1001)
**Predecessors**: [I329/T3a](SEARCH-001-D3-T3-bing-dual-host-zero-key-route.md), [I333/T4](SEARCH-001-D3-T4-bing-prepared-dual-host-integration.md) Complete / Closed
**Selected Iteration**: [I335](../../iterations/I335-search-bing-dual-runtime.md), Active / Claimed PROPOSED until governance #1002 merges
**External release gate**: [Gate-0 / #1000](https://github.com/wjhuang88/talos/issues/1000) unresolved; T3b/#957 real live/default activation is still BLOCKED / Unclaimed

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | D3-T5/#1001: real, Rust-native prepared dual Bing host transport and **explicit opt-in** invocation path; existing default compatibility/Network authority unchanged; no live Bing CI, no default distribution admission |
| Claimed At | 2026-11-22 |
| Source Issue | #1001 |
| Governance Claim PR | #1002 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer 2026-11-22 instructed implementation first; [governance claim PR #1002](https://github.com/wjhuang88/talos/pull/1002) proposed; exact-head CI, independent API/Network/security review and CAS mandatory before source merge; no vendor rights grant or independent human approval claimed |
| Implementation PR | Not started |
| Last Updated | 2026-11-22 |
| Handoff / Release Condition | Active only after governance claim merge. Source requires exact-head CI and protected independent review. Gate-0/#1000 remains separate from code completion; default/live route not admitted. |

## Bounded acceptance

- [ ] Replace mock-only execution in `compatibility/bing_dual.rs` with a non-public real `SearchTransport::execute_prepared` execution entry. Freeze both hosts before approval and send; use existing parser/admission, permission/redirect/proxy, one budget and header snapshot.
- [ ] WWW succeeds without CN send; only typed DNS, connection establishment/refusal or source-local timeout may attempt preauthorized CN. 403/429/all 5xx/redirect/challenge/200-empty/parser/error/untyped Network/permission/cancel/timeout/budget remain fail-closed.
- [ ] Provide an **explicit** opt-in prepared tool invocation or configuration path with independently authorized both endpoints; default shipped DDG Lite → Instant Answer → DDG HTML → CN-only chain and public web_search schema remain unchanged.
- [ ] In-process deterministic real HTTP/DNS/TLS test fixtures and native macOS/Windows/Linux CI; no network calls to public Bing.
- [ ] Independent API/Network/permission/security review of final SHA, governance validators, CAS, rollback and owner-first Completion Commit closeout.

## Non-goals, dependencies and rollback

No Bing license assertion, service bypass, CAPTCHA/429/403/5xx evasion, startup probes, ambient env-based WWW activation, new backend identity, public API changes, Browser/CDP, new Cargo dependencies, or production default source switch. Legal/rights #1000 and authorized real China/comparator qualification remain separately blocked. Roll back the isolated opt-in private path; original compatibility CN route remains unchanged.

## Required reads

`AGENTS.md`, `docs/sop/AGENT-COLLABORATION.md`, `docs/decisions/090-invocation-bound-search-endpoint-authority.md`, `crates/talos-tools/src/search_backend/transport/{compatibility.rs,compatibility/bing_dual.rs,compatibility/chain.rs}`, `crates/talos-tools/src/search_backend/bing_variants.rs`.
