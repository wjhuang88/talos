# SEARCH-001-C: Dormant Fixed Compatibility Source Chain

Private I293/#712/ADR-085/090 prerequisite based on main4e20ee95 (#741). The same four existing
fixed protocols can now execute sequentially under one prepared plan and existing transport budget.
No official root, default search, public API/model contract, provider or permission behavior changes.

## Owned Preparation

PreparedCompatibilityChain has exactly4 private owned PreparedCompatibilitySearch values, no
Clone/mutators and fixed redacted Debug. Preparation validates count1..20, then every fixed payload
and manifest endpoint membership before any header selection. Missing late scope, oversize query/
encoded GET after a valid Lite form, or invalid count returns without UA selection or network work.
Then one guarded pinned UA draw per source is frozen with the existing fixed profile. Capture
failure stops immediately without reroll. All requests, original query copies, count, protocol and
headers remain invocation-bound; future callers must retain these same objects across admission.
Preparation does not grant permission, resolve DNS, reserve an execution attempt or make a probe.

The immutable order is Lite → Instant Answer → HTML → Bing. There is no caller source list,
replacement query, callback/header map or destination override at execution. Each source consumes
its captured request through existing controlled transport and result admission. A usable response
must contain at least one admitted display URL; HTTP/parsing completion alone cannot win.

## Closed Continuation Policy

| Error family | Continue to the next declared source | Terminal whole-chain errors |
|---|---|---|
| Transport | Dns only | Every other variant, including Request, authorization/address/redirect denial and shared limits/deadline |
| DDG parser | Empty, Status, Challenge | Limit, Encoding, Parse |
| Bing parser | Empty, Status, RateLimited, Challenge | Limit, Encoding, Parse |
| Result admission | Empty, Invalid | Limit, Identity, Parse |

Exhaustive matches intentionally contain no permissive wildcard. Existing parser Parse can mean a
recovered dependency panic; it is not safely distinguishable from ordinary parse failure and stays
terminal. No error is reclassified using text. Terminal errors retain protocol and closed error;
exhaustion retains exactly4 actual errors in fixed order. Both Display strings are static, Debug
contains only enums, and no query/URL/body/dependency diagnostic is stored in outcomes or health.
This adds no health state. Internal exhaustion placeholders cannot escape before all4slots are
overwritten by observed DNS or semantic failures.

Typed resolver failure can proceed only to the next already prepared source. It does not retry
the failed source, add a destination, discard a facet or refresh any counter/deadline. A failed
resolution sends no HTTP query; its hostname lookup may already have occurred. Current Request
coarsens client construction, TLS verification, protocol/send and body-read failures, so it remains
terminal rather than being guessed to mean ordinary unavailability. Empty, oversized or forbidden
resolved-address sets remain Address failures; resolver timeout remains the enclosing Deadline.
Each later source still passes live authority before DNS and before send. Revocation after a DNS
failure, spent request reservations and expiry therefore stop further work. A forbidden address
is counted as a completed lookup and terminates without a send or another lookup.

The same transport performs live authority, invocation, DNS/address, proxy/TLS, redirect and
request/body checks for every actual hop. Budget checks precede each source and follow every
outcome, before returning success/terminal error, and before exhaustion. Requests, response body bytes and
deadline never reset between sources. A terminal failure cannot trigger another source, opaque
coordinator, legacy execution or knowledge/paid fallback. Success discards unused prepared data
without starting later sources. Production spawns no task; external future disposal and an already
started peer connection have separate controlled tests, not a guarantee to undo remote work.

## Provenance And Scope Limits

Order derives from pinned rust-websearch0.1.2 searcher.rs at VCSb30180a03609580e8b28dfb682284cb2ba249fba,
[immutable source](https://github.com/Rust-Framework/rust-agent-framework/blob/b30180a03609580e8b28dfb682284cb2ba249fba/crates/websearch/src/searcher.rs).
Archived bytes match freshly retrieved immutable source, SHA256
768a3c5135434ed2e1fea766e42f92580bfafca193d24178d2adf3d3cde6effe. Archive checksum remains
06b90d9a9ad9e7e347c4cd85c3fb37189256703db4fab1d6c6fb963ae1eed0ee. Complete MIT notice is
retained in tools LICENSES/rust-websearch-MIT.txt (full text plus Talos's final LF, as previously
documented). Chain ownership/safety/budget/result logic and synthetic fixtures are Talos-authored.

Upstream also rate-limits, probes/caches reachability, retries some network failures, draws UA at
client construction, and conditionally tries configured SearXNG. This chain omits those behaviors;
it performs4 upfront entropy draws, including unused sources, and terminates on all transport
errors except typed DNS failure.
It is only a traditional sequential fixed4 subset, not full source/retry/probe/entropy timing/
charset/proxy/service parity. It is one compatibility family, never4 native independent routes.
No live official-host search, result fetch or regional/provider qualification occurs.

Existing result admission remains display-only WHATWG acceptance, not raw-syntax/rendering/fetch/
SSRF authority. D/privacy/query exposure/result use, optional adapters, official CLI/SDK/MCP/
standalone roots, router/health/hedging/cancellation/rollback/default activation and release retain
their gates. I293 stays Active/Claimed, Completion Pending; #643/#624 stay open. Actual validation,
protected reviews, immutable candidates and native terminal receipts are indexed in I293/#643.

The [dormant direct plan](SEARCH-001-C-COMPATIBILITY-PLAN.md) retains this exact prepared chain
with its configuration-derived manifest/profile. Dispatch builds transport from that retained
manifest and reuses an injected existing budget; official permission/root integration remains gated.
