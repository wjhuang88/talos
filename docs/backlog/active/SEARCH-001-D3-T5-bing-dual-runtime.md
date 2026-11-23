# SEARCH-001-D3-T5: Bing WWW-first/CN-fallback Real Transport Implementation

**Status**: Complete / Closed for I335 bounded real transport + explicit opt-in **upon owner-first closeout publication**; Bing default/live activation not authorized
**Type**: Bounded Rust implementation and offline verification
**Parent**: [SEARCH-001 / #624](SEARCH-001-zero-config-global-search.md); [D3 / #937](SEARCH-001-D3-bing-direct-html-feasibility.md)
**Source Issue**: [#1001](https://github.com/wjhuang88/talos/issues/1001)
**Predecessors**: [I329/T3a](SEARCH-001-D3-T3-bing-dual-host-zero-key-route.md), [I333/T4](SEARCH-001-D3-T4-bing-prepared-dual-host-integration.md) Complete / Closed
**Selected Iteration**: [I335](../../iterations/I335-search-bing-dual-runtime.md), bounded implementation merged; closeout publication pending
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
| Authorization Evidence | Maintainer 2026-11-22 authorized code only; governance #1002 merged `1b3fc18bc323ffe71603766ecc6c0e61afa13412` before source; source #1003 exact-head [CI #45160764389](https://github.com/wjhuang88/talos/actions/runs/45160764389) 8/8 SUCCESS; independent Copilot [APPROVE](https://github.com/wjhuang88/talos/pull/1003#issuecomment-7103434725) for reviewed `e255c6d521cdc47377796a4c473a5503ac4586a7`; maintainer single-owner CAS merged with expected SHA. No supplier rights or independent human approval claimed. |
| Implementation PR | [#1003 MERGED](https://github.com/wjhuang88/talos/pull/1003); **Completion Commit: `bab2bdf31fc8200758397d38077dab57b9dd1aa4`** (pre-existing source merge) |
| Last Updated | 2026-11-23 |
| Handoff / Release Condition | I335 technical-only Complete / Closed after this owner-first closeout merges and Issue #1001 is synchronized; #1000 Gate-0, #957 T3b live/default, real-region validation, Epic #624 remain independently blocked/open. |

## Bounded acceptance

- [x] Replace mock-only execution in `compatibility/bing_dual.rs` with a non-public real `SearchTransport::execute_prepared` execution entry. Freeze both hosts before approval and send; use existing parser/admission, permission/redirect/proxy, one budget and header snapshot.
- [x] WWW succeeds without CN send; only typed DNS, connection establishment/refusal or source-local timeout may attempt preauthorized CN. 403/429/all 5xx/redirect/challenge/200-empty/parser/error/untyped Network/permission/cancel/timeout/budget remain fail-closed.
- [x] Provide an **explicit** opt-in prepared tool invocation or configuration path with independently authorized both endpoints; default shipped DDG Lite → Instant Answer → DDG HTML → CN-only chain and public web_search schema remain unchanged.
- [x] In-process deterministic real HTTP/DNS/TLS test fixtures and native macOS/Windows/Linux CI; no network calls to public Bing.
- [x] Independent API/Network/permission/security review of final SHA, governance validators, CAS, rollback and owner-first Completion Commit closeout.

## Non-goals, dependencies and rollback

No Bing license assertion, service bypass, CAPTCHA/429/403/5xx evasion, startup probes, ambient env-based WWW activation, new backend identity, public API changes, Browser/CDP, new Cargo dependencies, or production default source switch. Legal/rights #1000 and authorized real China/comparator qualification remain separately blocked. Roll back the isolated opt-in private path; original compatibility CN route remains unchanged.

## Required reads

`AGENTS.md`, `docs/sop/AGENT-COLLABORATION.md`, `docs/decisions/090-invocation-bound-search-endpoint-authority.md`, `crates/talos-tools/src/search_backend/transport/{compatibility.rs,compatibility/bing_dual.rs,compatibility/chain.rs}`, `crates/talos-tools/src/search_backend/bing_variants.rs`.

## Completion and native validation — 2026-11-23

**Completion Commit: `bab2bdf31fc8200758397d38077dab57b9dd1aa4`** ([source PR #1003](https://github.com/wjhuang88/talos/pull/1003), reviewed exact source HEAD `e255c6d521cdc47377796a4c473a5503ac4586a7`). This is an existing merged implementation/evidence commit, **not** the governance closeout commit. Status becomes effective on this owner-first closeout's target-branch publication.

- **Claim-first**: [#1002](https://github.com/wjhuang88/talos/pull/1002) merged `1b3fc18bc323ffe71603766ecc6c0e61afa13412` before I335 source implementation. Responsible Actor @wjhuang88; technical bounded work slice unchanged.
- **Functionality**: Rust-native preapproved two-host WWW-first/CN-fallback `SearchTransport::execute_prepared` path with `[search] backend = "bing-dual"` explicit opt-in. WWW→CN only on typed source-local DNS/connect/refusal/source-local timeout while permission, proxy, cancellation, budget and deadline remain valid. HTTP errors, redirect/challenge/DOM drift, shared proxy failures, missing authority and exhausted budgets do not rotate. Original DDG-first / CN-only Auto default and public `web_search` schema remain unchanged. Offline deterministic real transport/negative fixture coverage; no public Bing CI calls.
- **Verification**: [CI #45160764389](https://github.com/wjhuang88/talos/actions/runs/45160764389) on source HEAD `e255c6d...` SUCCESS **8/8 applicable jobs**: macOS native format/check/clippy/tests, Windows workspace and aggregate, Windows/Linux Desktop, remote owner reconciliation, classifier and installer fixture. Earlier unrelated intermittent macOS Fork / Windows Actor tests failed on first attempts and succeeded on repeat; historical failures remain preserved, not waived or attributed to this work.
- **Independent protected review**: Copilot exact-head [APPROVE](https://github.com/wjhuang88/talos/pull/1003#issuecomment-7103434725) with no source/API/Network/security blockers; maintainer used expected-head-SHA CAS on merge and GitHub reported mergeable/clean. No separate human approval claimed.
- **Rollback and residual**: disable opt-in `bing-dual` selection to revert to original default routing; no default/live activation, vendor service-access/output rights determination, genuine China/comparator reachability or broad provider admission was proven. Gate-0 [#1000](https://github.com/wjhuang88/talos/issues/1000), live/default T3b [#957](https://github.com/wjhuang88/talos/issues/957), D3 [#937](https://github.com/wjhuang88/talos/issues/937) and Epic [#624](https://github.com/wjhuang88/talos/issues/624) remain separate.
