# ADR-085: Talos-Owned Web Search Routing And Backend Boundary

*Status: Accepted / I288 (2026-10-03)*

## Context

SEARCH-001 / #624 requires a zero-configuration web-search capability that can choose usable
routes from observed network conditions, including best-effort operation from genuine
mainland-China networks, while keeping paid/self-hosted providers optional.

The current `WebSearchTool` mixes model-facing Tool semantics with provider activation, HTTP
requests, result parsing and fallback. Its `tokio::select!` block returns the first completed
branch, including errors, even though comments describe first-success behavior. The zero-key path
also delegates to `rust_websearch::search()`, whose current upstream API is itself an
auto-selected backend coordinator, while Talos labels the result as DuckDuckGo and ignores the
upstream source identity.

ADR-072 already defines **Provider** as a platform object that implements capabilities and has a
Registered/Ready/In Use/Unavailable lifecycle. NET-001/#199 owns the future generic retry,
backoff, breaker and transport-resilience contract. Search must not redefine either boundary.

## Decision

### 1. Keep `web_search` as the stable capability surface

Preserve the existing model-facing Tool identity, Network permission nature, Tool family, input
schema and contribution source through the migration unless a later child explicitly accepts a
compatibility change.

Provider selection is a runtime implementation concern. The model does not choose a backend.

### 2. Introduce a private `SearchBackend` boundary below the built-in Search Provider

The built-in Talos Search Provider is the ADR-072 Provider implementing the `web_search`
capability. Its internal engines/adapters are called **SearchBackend** and are not independently
registered ADR-072 Providers.

This prevents search-specific lifecycle and health details from leaking into the global
Capability/Provider registry.

### 3. Keep the first implementation inside `talos-tools`

Do **not** create a public `talos-search` crate in SEARCH-001-B.

The first boundary lives under the existing `talos-tools/network` feature because:

- `WebSearchTool` is currently the only consumer;
- the crate already owns the network Tool contribution and search dependencies;
- a new published crate would add release/API ownership before reuse is proven;
- module extraction is fully reversible while preserving tool compatibility.

A separate crate requires a new decision if a second non-tool consumer, supported Runtime SDK API,
independent release cadence, or dependency-isolation requirement emerges.

### 4. Talos owns routing and backend identity

The target SearchRouter receives explicit SearchBackend instances and owns:

- candidate ordering;
- first-valid-success semantics;
- bounded hedging/fallback;
- normalized result validation;
- backend source identity;
- search-specific health ranking;
- cancellation/deadline propagation.

Compatibility adapters may temporarily delegate internally to `rust-websearch`, but opaque
third-party auto-routing must not remain the authority for Talos's final default route.

### 5. No network probe at process startup

A fresh installation performs no search reachability traffic until a search is invoked.

Cold start uses a static admitted-backend order. Search outcomes update local backend health.
Recovery probing, if later required, must be bounded, privacy-aware and tied to user search
activity or an explicit diagnostic action.

### 6. Route from network evidence, not geography

Do not branch on country, GeoIP, account locale or a hard-coded "China mode".

Recent endpoint outcome and latency are the primary routing evidence. Query language may influence
request parameters or equal-health tie breaking but cannot override known reachability failure.

Regional environments are release/qualification evidence, not runtime geolocation inputs.

### 7. Use bounded hedging and first valid success

The router starts one healthy zero-key backend. A second candidate may start after a bounded hedge
delay or an immediately classified failure. Return the first **valid successful** normalized
response and cancel remaining work.

A valid success requires parse success plus at least one admissible result with a valid HTTP/HTTPS
URL. "Future completed with `Ok`" alone is insufficient.

Exact hedge delays and thresholds belong to SEARCH-001-C and measurement evidence.

### 8. Search-specific health is local selection state, not a second transport breaker

Search may remember backend ID, outcome class, latency and bounded timestamps/TTL for selection.
It must not store query text.

Retry count, exponential backoff, generic circuit-breaker state and transport replay policy remain
NET-001/#199 authority. Until that shared contract lands, the search router may fail over to a
different backend but must not build an independent generic retry framework.

### 9. Typed backend errors precede routing policy

Backends must distinguish at least:

- unreachable/connectivity;
- timeout;
- rate limit;
- challenge/blocking;
- HTTP status;
- auth/credential failure;
- parse/response drift;
- no results;
- invalid normalized results;
- cancellation;
- enclosing deadline expiry.

String formatting occurs at the Tool boundary, after routing decisions.

### 10. Premium/self-hosted providers require explicit selection

Credential discovery and provider selection are separate.

The presence of `TAVILY_API_KEY` or another credential may make an adapter available but must not
silently opt the user into metered usage. Future SEARCH-001-H configuration can explicitly select a
premium provider and optionally request fallback to `auto`.

SearXNG remains explicit self-hosted configuration; SEARCH-001 does not authorize a default Talos
relay.

### 11. Wikipedia is not a target generic SearchBackend

Wikipedia OpenSearch may remain behind a legacy compatibility adapter while B/C preserve behavior,
but the target generic web router does not count it as a web-search backend or as one of the two
independent native zero-key routes.

A future knowledge-source capability may own it separately.

### 12. Native backend admission is evidence-gated

SEARCH-001-A names candidate classes but admits none.

SEARCH-001-D must evaluate terms/automation policy, parser maintainability, independent failure
domains, deterministic fixtures, live reachability and provenance-backed regional evidence before E
or F implements a native default route.

Two native routes count as independent only if they use distinct upstream infrastructure and
distinct Talos adapter/parser paths; two labels or wrappers around the same opaque upstream
auto-router do not qualify.

## Compatibility Contract

| Surface | Migration rule |
|---|---|
| `web_search` name/schema | Preserve |
| Network permission | Preserve |
| Tool family/contribution | Preserve |
| Model-visible provider choice | None; runtime-owned |
| Current output text | B preserves; later metadata changes separately reviewed |
| `rust-websearch` | Compatibility adapter first, then optional/removable after G/H evidence |
| Tavily/SearXNG | Preserve current behavior during B; H owns explicit target-selection migration |
| Wikipedia fallback | Preserve only as necessary for compatibility; not target generic backend |
| Cargo/public API | No new public crate/API in B |

## Target Internal Shape

Illustrative, not a public API commitment:

```text
WebSearchTool
  -> SearchRouter
       -> SearchBackend (rust-websearch compatibility)
       -> SearchBackend (native zero-key A)
       -> SearchBackend (native zero-key B)
       -> SearchBackend (explicit Tavily)
       -> SearchBackend (explicit SearXNG)
```

Likely module ownership:

```text
crates/talos-tools/src/web_search/
  mod.rs
  backend.rs
  error.rs
  router.rs
  health.rs
  backends/...
```

## Migration Stages

1. **B — compatibility boundary**: extract internal request/result/error/backend seams; wrap all
   existing behavior; no routing/output change.
2. **C — Talos router**: typed errors, first-valid-success hedging, cancellation/deadline, source
   identity and bounded search health. Coordinate with NET-001; do not add generic transport retry.
3. **D — provider admission**: evidence/terms/fixtures/live-regional qualification.
4. **E — native route #1**: admitted zero-key backend; rust-websearch remains rollback fallback.
5. **F — native route #2**: independently admitted backend and failover proof.
6. **G — global hardening**: network-change recovery, smoke/soak and provenance-backed multi-region
   evidence including a real mainland-China path or an explicitly unresolved gate.
7. **H — explicit premium/self-hosted config**: no credential-implies-usage behavior.
8. **I — rust-websearch disposition**: optional compatibility adapter or removal based on measured
   evidence; default search must survive without it.

## Validation And Reversal Triggers

Validate each stage with deterministic parser/router fixtures, cancellation/deadline tests, exact
tool-schema compatibility and live smoke appropriate to the changed backend.

Revisit this ADR if:

- search routing needs a supported consumer outside `talos-tools`;
- ADR-072 introduces a domain backend contract that makes SearchBackend redundant;
- NET-001's accepted policy requires a different ownership split;
- provider terms or platform policy prohibit the intended zero-key automation;
- multi-region evidence shows the two-route default cannot meet SEARCH-001's best-effort goal.

Rollback for B/C is the existing `WebSearchTool` path plus `rust-websearch` compatibility
adapter; E/F must retain the last accepted route until their own evidence is stable.

## Rejected Alternatives

- **Create `talos-search` immediately** — premature public/release boundary with one consumer.
- **Reuse ADR-072 Provider for each search engine** — leaks domain-internal health/routing into the
  platform lifecycle and complicates Plugin/Bundle semantics.
- **Country/GeoIP routing** — geography is an unreliable proxy for actual reachability and creates
  privacy/maintenance cost.
- **Broadcast every query to every backend** — unnecessary privacy exposure and possible metered
  cost.
- **Let rust-websearch remain the final router** — Talos cannot own source identity, health,
  fallback guarantees or parser failure semantics.
- **Fork rust-websearch as the primary architecture** — improves emergency maintenance control but
  does not solve the single-router/opaque-source boundary; a fork remains a possible contingency
  dependency strategy, not the domain architecture.
- **Wikipedia as permanent web fallback** — changes semantic class from generic web results to
  encyclopedia knowledge.

## Evidence

I288 code-truth and migration evidence:
`docs/reference/I288-SEARCH-ARCHITECTURE-AUDIT-2026-09-30.md`.

The deterministic characterization test added in I288 proves that the exact `tokio::select!`
pattern used by current routing propagates a fast error before a later success. No production
search behavior changes in I288.

Acceptance record: PR #629 was merged to `main` as `4567bf85ef1fb151fbb07a30e1f5f26a81feeed7`
after the architecture package, deterministic test, governance validation and locked focused
tests were reviewed. The decision authorizes dependency-ready B/C/D planning; it does not itself
authorize a provider implementation or change production search behavior.
