# I288 Search Architecture Audit — 2026-09-30

**Owner**: SEARCH-001-A / #625
**Iteration**: I288
**Scope**: current-code characterization, architecture/evidence contract, migration decomposition only
**Production behavior change**: none

## Executive Findings

1. `web_search` is one stable model-facing Network tool registered by the `talos-tools:network`
   contribution group. Its input schema (`query`, `max_results`, `include_snippets`), tool name,
   family and permission nature should remain stable through the migration.
2. Current routing is implemented inside `crates/talos-tools/src/web_search.rs`; there is no
   Talos-owned search-domain contract separating tool presentation from backend selection.
3. The current `tokio::select!` block is **first-completion**, not first-success. A fast error from
   any participating backend can immediately abandon still-running candidates and enter the
   Wikipedia fallback. I288 adds a deterministic test that reproduces this scheduling fact without
   issuing network traffic.
4. The `rust-websearch` call is more opaque than the local name `search_duckduckgo` implies.
   Talos calls upstream `rust_websearch::search(...)` and then discards the upstream
   `SearchResults.source`. Current upstream 0.1.2 documentation describes `search()` as an
   auto-selected backend coordinator with DuckDuckGo and Bing CN among its possible sources.
   Therefore Talos cannot currently attribute or health-score the actual zero-key endpoint while it
   unconditionally formats the local source as "DuckDuckGo".
5. Merely having `TAVILY_API_KEY` in the environment currently makes Tavily participate in the
   parallel race. That is incompatible with the SEARCH-001 target rule that a credential's
   existence alone must not silently opt a user into metered/paid usage.
6. Wikipedia OpenSearch is a knowledge source, not a generic web-search backend. It is acceptable as
   a compatibility fallback during migration but must not remain the target generic-web fallback.
7. A new public `talos-search` crate is **not justified yet**. Search routing currently has one
   consumer (`WebSearchTool`) and already sits behind the optional `talos-tools/network` feature.
   The first migration should be an internal module boundary. Extraction becomes a separate
   decision if a second non-tool consumer, SDK contract, independent release cadence or public
   capability boundary appears.

## Evidence Sources

Repository truth inspected for I288:

- `crates/talos-tools/src/web_search.rs`
- `crates/talos-tools/src/contributions.rs`
- `crates/talos-tools/src/lib.rs`
- `crates/talos-tools/Cargo.toml` and `Cargo.lock`
- `crates/talos-cli/src/registry.rs`
- `crates/talos-permission/src/lib.rs`
- TOOL-009, TOOL-012, TOOL-013, TOOL-014 and WEBFETCH owner documents
- ADR-072 (Capability / Provider / Plugin / Bundle)
- NET-001 / #199

External dependency observation:

- docs.rs reports `rust-websearch 0.1.2` released 2026-08-29 and documents
  `search(query, config)` as an auto-selected backend search whose `SearchSource` can include
  DuckDuckGo variants, Bing CN and SearXNG:
  https://docs.rs/crate/rust-websearch/0.1.2
- Upstream project:
  https://github.com/Rust-Framework/rust-agent-framework

Release freshness is therefore **not** the primary risk. The architectural risk is that Talos does
not own the routing/source/error semantics or parser maintenance boundary of its default search path.

## Current Call Graph And Ownership

```text
LLM / Agent
   |
   | tool name: web_search
   v
ToolRegistry / talos-tools:network contribution
   |
   v
WebSearchTool::execute
   |
   v
WebSearchTool::execute_search
   |
   +-- rust_websearch::search(...) -------- local label: DuckDuckGo
   |
   +-- direct Tavily reqwest request ------- if TAVILY_API_KEY exists
   |
   +-- direct SearXNG reqwest request ------ if SEARXNG_URL exists
   |
   +-- on first selected error:
       direct Wikipedia OpenSearch fallback
```

The search implementation owns HTTP clients, timeout values, provider activation, result parsing,
source labels and fallback. Registration/disclosure and permission remain outside it.

## Stable External Contract

The following are compatibility surfaces for later SEARCH-001 children unless a child explicitly
accepts a migration:

| Surface | Current contract | I288 disposition |
|---|---|---|
| Tool name | `web_search` | Preserve |
| Tool family | `ToolFamily::Network` | Preserve |
| Permission nature | `ToolNature::Network` | Preserve |
| Input | query + max_results (1..20) + include_snippets | Preserve |
| Model-facing output | bounded text result list | Preserve through B; later metadata additions require review |
| Tool contribution source | `talos-tools:network` | Preserve |
| Cargo feature | `talos-tools/network` | Preserve |
| URL fetch separation | discovery by `web_search`; content ingestion by `fetch_url` / `http_request` | Preserve |

The current `is_read_only() == false` value is retained as code truth in I288; this story does not
reinterpret the generic Tool read-only flag.

## Confirmed Current-Behavior Risks

### R1 — first completion can masquerade as "all backends failed"

Current code:

```rust
let race_result = tokio::select! {
    res = ddg_fut => res.map(...),
    res = tavily_fut => res.map(...),
    res = searxng_fut => res.map(...),
};

match race_result {
    Ok(...) => ...,
    Err(_) => search_wikipedia(...),
}
```

Only one selected future must fail to reach the fallback; the remaining futures are dropped. The
comment "first successful response wins" is therefore not the implemented behavior. The I288
deterministic test `select_pattern_propagates_fast_error_before_later_success` captures this fact.

### R2 — zero-key backend identity is hidden

`search_duckduckgo()` calls upstream `rust_websearch::search()` rather than an explicitly pinned
DuckDuckGo-only entrypoint and ignores `SearchResults.source`. Under upstream 0.1.2 documentation,
one Talos-visible "DuckDuckGo" success can have been selected from another upstream backend. This
prevents trustworthy backend-specific health, diagnostics and regional evidence.

### R3 — environment key implies paid-provider participation

A non-empty `TAVILY_API_KEY` makes Tavily race automatically. Environment discovery is useful for
credential lookup, but it must not itself be the user's provider-selection consent in the target
architecture.

### R4 — knowledge fallback is presented as web fallback

Wikipedia may return useful knowledge even when generic web search is unavailable, but silently
substituting it changes the semantic class of results. The target router therefore does not model
Wikipedia as a generic `SearchBackend`.

### R5 — backend errors are flattened too early

Current branches reduce HTTP, timeout, format, auth/rate-limit and no-result conditions to strings.
The router cannot distinguish:
- network unreachable / DNS / TLS;
- deadline / timeout / cancellation;
- HTTP status and rate limit;
- auth/credential failure;
- anti-bot/challenge;
- parse drift / invalid response;
- legitimate no-results;
- invalid normalized result.

Typed errors are required before Talos can make safe routing, diagnostics and health decisions.

## Architecture Ownership Decision

ADR-085 proposes the following conceptual boundary inside `talos-tools`:

```text
WebSearchTool
    |
    v
SearchRouter
    |
    +-- SearchBackend: rust-websearch compatibility adapter
    +-- SearchBackend: native zero-key backend A   (future E)
    +-- SearchBackend: native zero-key backend B   (future F)
    +-- SearchBackend: Tavily explicit adapter     (future H)
    +-- SearchBackend: SearXNG explicit adapter    (future H)
```

`SearchBackend` is deliberately **not** ADR-072's platform `Provider`. The built-in Talos Search
Provider implements the stable `web_search` capability; SearchBackend is a private strategy below
that provider. Optional Plugin/remote Provider registration remains ADR-072 territory and is not
created by SEARCH-001-A.

Recommended initial internal layout for B/C (names may vary without changing the decision):

```text
crates/talos-tools/src/web_search/
  mod.rs
  backend.rs
  error.rs
  router.rs
  health.rs
  backends/
    rust_websearch.rs
    tavily.rs
    searxng.rs
    wikipedia_legacy.rs
```

No new crate is created in B. Reconsider extraction only if one of these becomes true:

1. a second non-`talos-tools` consumer needs the router contract;
2. `talos-runtime` must expose search composition as supported SDK API;
3. independent feature/release/version ownership is required;
4. the module would otherwise force network dependencies into a consumer that does not need them.

## Target Request / Result / Error Contract

Internal target types, not public API in I288:

```rust
struct SearchRequest {
    query: String,
    max_results: usize,
    include_snippets: bool,
}

struct SearchHit {
    title: String,
    url: Url,
    snippet: Option<String>,
}

struct SearchResponse {
    backend: SearchBackendId,
    hits: Vec<SearchHit>,
}

enum SearchBackendError {
    Unreachable,
    Timeout,
    RateLimited,
    Challenge,
    HttpStatus(u16),
    Auth,
    Parse,
    NoResults,
    InvalidResult,
    Cancelled,
    DeadlineExceeded,
}
```

The concrete Rust API remains a B/C implementation detail. The semantic requirements are:

- HTTP/HTTPS URL validation before a result is considered valid;
- no empty result set presented as provider success;
- typed cancellation/deadline distinct from provider failure;
- auth/metering failures do not poison zero-key health;
- parse drift and challenge are distinguishable from ordinary no-results.

## Router Contract

Target behavior:

1. **No startup network probe.** A fresh process performs no search reachability request until the
   user actually invokes search.
2. **Network truth, not geography.** Runtime selection uses actual recent outcome/latency evidence.
   Locale/language can shape query parameters or tie-breaking but cannot hardcode
   `country == CN -> backend X`.
3. **First valid success.** Failure of one backend does not cancel a still-viable candidate.
4. **Bounded hedging, not broadcast.** Start one healthy zero-key backend; after a bounded hedge
   delay or classified failure, start the next candidate. This lowers tail latency without sending
   every query to every provider by default.
5. **Local health contains no query text.** Store backend ID, outcome class, latency bucket/time and
   bounded timestamps/TTL only.
6. **No hidden transport retry policy.** Per-request retry/backoff/breaker behavior belongs to
   NET-001/#199. Until that contract exists, SearchRouter can choose a different backend but should
   not build a competing generic transport retry loop.
7. **Explicit premium use.** A credential may make an adapter *available*; it does not make it
   preferred or active. Future configuration must explicitly select the premium backend.
8. **Cancellation/deadline propagation.** Hedge timers and backend futures stop when the enclosing
   tool request is cancelled or its deadline expires.

## Target Configuration Semantics

No config migration is implemented in I288. H owns the eventual schema. Required semantics:

```toml
[search]
provider = "auto"          # implicit default; need not be written to a fresh config

# Example future explicit override:
# provider = "tavily"
# fallback = "auto"

[search.tavily]
api_key_env = "TAVILY_API_KEY"
```

Rules:

- missing `[search]` means `auto`;
- `auto` uses admitted zero-config backends only;
- environment credentials are lookup sources, never implicit spending consent;
- explicit premium/self-hosted selection may opt into `fallback = "auto"`;
- invalid explicit provider configuration fails clearly and does not silently reinterpret another
  provider as selected.

## Provider Admission / Independence Matrix

I288 admits **no new production backend**. D must fill evidence before E/F implementation.

| Candidate class | Zero config | Target role | Main unknowns / admission evidence |
|---|---:|---|---|
| rust-websearch compatibility adapter | yes | Transitional only | exact locked-source routing, source attribution, parser/reachability behavior, upstream change containment |
| DuckDuckGo Lite/HTML native candidate | yes | Candidate native route | terms/acceptable automation, anti-bot stability, HTML fixtures, direct reachability by region |
| Bing CN native candidate | yes | Candidate independent route | endpoint/terms stability, parser fixtures, direct reachability by region, independence from first native route |
| SearXNG | no (instance required) | Explicit self-hosted | instance trust, URL/SSRF policy, auth if any, result normalization |
| Tavily | no (credential/metered) | Explicit premium | auth/quota/rate-limit semantics, cost/consent UX |
| Wikipedia OpenSearch | yes | Knowledge source, not generic web backend | future knowledge-capability owner; legacy compatibility only |

Two routes count as independent only when all of these are true:

- distinct upstream search infrastructure/hostname ownership;
- distinct Talos parser/adapter path;
- neither is just another mode selected inside the same opaque compatibility crate;
- a single known endpoint failure/challenge does not make both routes fail by construction;
- separate fixture and live-smoke evidence exists.

Therefore two wrappers around `rust_websearch::search()`, or two local labels that still delegate to
the same upstream auto-router, do **not** satisfy SEARCH-001's two-native-route gate.

## Multi-Region Evidence Protocol

Runtime routing never needs GeoIP, but release qualification needs controlled regional evidence.

For each admitted zero-key native route, D/G should record:

- date/time and exact Talos commit;
- coarse test region (for example US-West, EU, mainland China);
- network class (residential/mobile/cloud when known), without storing personal IP addresses;
- direct-vs-user-configured proxy status;
- DNS/TLS/connect/HTTP/parser/result-normalization outcome;
- latency bucket and result count for a small innocuous query set;
- whether anti-bot/challenge/rate limiting occurred;
- whether a second route succeeded under the same environment.

For mainland-China acceptance, evidence must come from a genuine network environment and state
whether traffic was direct or used a user-configured proxy. Simulated locale, GeoIP headers or a
foreign cloud region are not substitutes. If such an environment is unavailable, record
**evidence unavailable** and keep the regional acceptance gate open.

Live probes should be low-frequency and non-sensitive. CI fixtures remain the parser regression
authority; live smoke detects reachability and upstream drift, not correctness of every query.

## Privacy And Safety

- Hedging duplicates a query only after the configured delay/failure condition; do not broadcast a
  query to all engines by default.
- No query text enters health persistence or telemetry by default.
- Network permission remains the authority for each `web_search` invocation.
- Backend URLs and redirects must use existing URL/SSRF safety contracts where applicable;
  self-hosted endpoints require explicit trust/config review.
- Do not bypass CAPTCHA, access controls or regional restrictions.
- No default Talos relay is authorized by SEARCH-001.
- Provider terms and automation policy are an explicit D admission input, not inferred from
  technical reachability alone.

## Revised Migration Chain

| Child | Deliverable | Entry gate | Exit gate / rollback |
|---|---|---|---|
| B | Internal SearchBackend boundary + compatibility adapters | ADR-085 accepted | Existing output/config behavior preserved; revert module extraction if parity fails |
| C | Talos SearchRouter, typed errors, first-valid-success hedging | B | Deterministic router tests; no generic retry duplication; legacy compatibility fallback retained or explicitly migrated |
| D | Provider feasibility/admission evidence | ADR-085; may overlap B/C research | At least candidate evidence matrix; unknown providers remain unadmitted |
| E | First native zero-key backend | C + D admission | fixture + live evidence; rust-websearch remains rollback fallback |
| F | Second independent native zero-key backend | C + D + E interface | independence criteria + failover proof |
| G | Global routing hardening / network-change recovery | E + F | multi-region evidence incl. mainland-China path or explicitly open gate |
| H | Premium/self-hosted explicit override normalization | C | no implicit paid usage; secret/config migration tests |
| I | rust-websearch de-risk/retirement | G + H | default search survives with dependency removed/optional; rollback decision recorded |

D is an evidence gate, not a promise that any named candidate will be admitted.

## Acceptance Mapping For SEARCH-001-A

- **Repository truth**: this audit records current call graph, feature/config/permission boundaries.
- **First-completion evidence**: deterministic unit test added under `web_search.rs`.
- **Architecture decision**: ADR-085.
- **ADR-072 / NET-001 alignment**: SearchBackend remains internal; platform Provider and generic
  resilience ownership are not duplicated.
- **Zero-config/premium semantics**: target rules above; implementation deferred to B/C/H.
- **Provider and regional evidence**: admission/independence matrix plus controlled regional protocol.
- **B–I migration/rollback**: revised chain above.
- **Production behavior**: unchanged by I288 except test/documentation evidence.

## Residual Questions Deliberately Deferred

- Exact first and second native zero-key engines: SEARCH-001-D.
- Exact hedge delay and health TTL numbers: SEARCH-001-C based on measurements.
- Exact shared retry/breaker API: NET-001/#199.
- Config schema/version migration: SEARCH-001-H.
- Whether to expose backend/source metadata to users or model output: separately reviewed behavior
  change after router ownership exists.
- Whether a future SDK needs a public Search Provider contract: trigger a separate extraction/API
  decision instead of preemptively publishing `talos-search`.
