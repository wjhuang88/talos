# SEARCH-001-C Dormant SearchRouter Core

Status: implementation stage under I293; **not an activation or C completion record**.

Parent: SEARCH-001-C / #643, SEARCH-001 / #624  
Architecture: ADR-085 and ADR-090  
Integration plan: `SEARCH-001-C-ROUTER-INTEGRATION-PLAN.md`

## Purpose

This stage replaces the disposable scheduling model with a private Talos-owned routing core while
leaving every official search entry point unchanged. It establishes the reusable control-flow
boundary needed before compatibility adapters can be migrated into the router.

The implementation is private to `talos-tools::search_backend`. `WebSearchTool`, tool registration,
Agent/CLI/SDK/MCP roots and current default search behavior do not call it in this stage.

## Contract

For one explicit ordered candidate set, the router:

- launches each backend identity at most once;
- returns the first result batch that passes router admission rather than the first completed future;
- immediately fills an available slot after a failed/empty/invalid candidate;
- may start a later candidate after the explicit hedge delay, subject to the explicit in-flight cap;
- shares one caller-supplied monotonic deadline across all candidate and admission work;
- checks an explicit cancellation receiver before launch, during scheduling and before success;
- owns candidate futures directly and spawns no detached task;
- drops pending owned futures when the router returns or its own future is dropped;
- distinguishes no eligible candidates from finite exhaustion;
- retains backend identity with each exhausted failure.

The router deliberately accepts `Arc<dyn SearchBackend + 'a>`, not only `'static` backends.
Existing `CompatibilityBackend<'a>` borrows `WebSearchTool`; preserving that lifetime avoids a
new ownership redesign solely to enter the router. Because the router does not spawn candidate
work, this borrowing remains invocation-scoped.

## Result admission

A successful backend future is not itself a router winner. Before it can win, the router checks:

- response count is within the explicit examined-result bound;
- every result carries the backend identity that produced the response;
- title and snippet are each at most 16 KiB;
- URL bytes fit the explicit router bound;
- an admitted URL parses as absolute HTTP or HTTPS, has a host and contains no username/password;
- invalid URLs are filtered before requested-count truncation;
- an empty input batch is distinct from a non-empty batch with no admissible URL;
- deadline/cancellation fences run before and after admission work.

URL parsing is fail-closed across unwind. This stage only admits display data; it grants no fetch,
rendering, redirect or provider authority.

## Explicit non-goals

This stage does **not**:

- register or activate the router;
- select a shipping hedge delay, concurrency cap or default candidate order;
- discover environment configuration or credentials;
- silently enable Tavily, SearXNG or any paid provider;
- qualify a native provider or change SEARCH-001-D/E/F ownership;
- add query-derived health state, startup probes, GeoIP/country routing or generic retry/backoff;
- replace NET-001 transport resilience ownership;
- enrich the current coarse compatibility error taxonomy;
- prove HTTP/DNS/helper-task cancellation merely because owned futures are dropped;
- change the model-facing `web_search` schema, formatting or output;
- close C-V01..C-V11, EP gates, #643 or #624.

Search-specific bounded health, typed adapter error enrichment, caller/root integration,
compatibility/rollback proof and official/default activation remain separate subsequent gates.

## Deterministic evidence required for this stage

Private paused-time fixtures cover:

1. fast failure followed by later valid success without waiting for the hedge delay;
2. delayed hedge with a bounded in-flight peak and pending-loser drop;
3. invalid-URL filtering before truncation;
4. typed empty, invalid and backend-error exhaustion;
5. no-eligible versus exhausted distinction;
6. one enclosing deadline across hedged work;
7. pre-launch and in-flight cancellation;
8. backend-identity mismatch rejection;
9. caller drop releasing owned candidate futures;
10. zero-bound and duplicate-backend policy rejection;
11. request-count rejection above the explicit router admission bound.

Exact-head formatting, workspace check/Clippy/tests, native platform CI, governance/reconciliation,
independent technical review and fresh merge-time CAS remain required before this stage can merge.
Passing this stage still does not authorize official search activation.

## Validation history rule

Failed or superseded runs remain evidence. In particular, the first published candidate failed
Rust 1.99 format checks in the new test file; a formatting-only correction must not be described as
repairing runtime behavior. Later green validation, if obtained, does not erase that failed run.
