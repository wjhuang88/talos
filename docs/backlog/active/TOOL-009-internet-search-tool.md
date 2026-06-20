# TOOL-009: Internet Search Tool

**Status**: Planned
**Priority**: P1
**Source**: User request 2026-06-20
**Depends on**: WEBFETCH-001 Phase 0 (for HTTP infrastructure)

## Problem

Talos has no internet search capability. The agent cannot answer questions
that require current information, look up documentation, or search for
solutions beyond its training data. WEBFETCH-001 covers *fetching specific
URLs* but not *discovering* which URLs to fetch.

This is a critical gap for a coding agent — users routinely ask agents to
search for API documentation, error messages, library usage examples, or
current best practices.

## Scope

Add a `web_search` tool that performs internet searches and returns
structured results:

### Input Parameters

| Parameter | Type | Description |
|---|---|---|
| `query` | string (required) | Search query |
| `max_results` | u32 (default 10, max 20) | Number of results |
| `include_snippets` | bool (default true) | Include text snippets |

### Search Backend (configurable)

Tiered approach — embed-first, no mandatory third-party API keys:

| Tier | Backend | Type | Key required? | Notes |
|---|---|---|---|---|
| **Default** | DuckDuckGo Instant Answer | Free API, no scraping | **No** | `api.duckduckgo.com/?q=...&format=json` — definitions, abstracts, related topics |
| **Fallback** | Wikipedia OpenSearch | Free API, no scraping | **No** | `wikipedia.org/w/api.php?action=opensearch` — encyclopedia results only |
| **Full search** | SearXNG (self-hosted) | Self-hosted meta-search | **No** (self-hosted) | Aggregates 70+ engines; 5 min Docker setup; JSON API |

**Not included** (per embed-first principle):
- DDG HTML scraping — aggressively blocked on datacenter/VPS IPs. Research
  (2026-06-20) confirmed CAPTCHA walls, HTTP 202 traps, and "anomaly in the
  request" blocks. Not reliable as a default backend.
- Brave Search API, Tavily — require third-party API keys.

Configuration:
```toml
# Optional: only needed for full multi-engine search
[search]
searxng_url = "https://search.example.com"  # Self-hosted SearXNG instance
```

If SearXNG is not configured, the tool uses DuckDuckGo Instant Answer →
Wikipedia fallback automatically. No config required for basic search.

### Output Format

Compact, model-friendly text:
```
Searched: "rust axum middleware example"
Results: 10

1. axum::middleware - Docs.rs
   https://docs.rs/axum/latest/axum/middleware/index.html
   axum::middleware - axum::middleware::from_fn - axum::middleware::from_extractor

2. Tower middleware with axum - Tokio blog
   https://tokio.rs/blog/2023-01-03-axum-middleware
   How to use Tower's middleware system with axum, including examples...

...
```

### Permission

- Nature: `Network`
- Requires explicit allow rule in permission config
- Can be disabled independently from `http_request` (WEBFETCH-001)

### Relationship to WEBFETCH-001

- `web_search` discovers URLs → the agent calls `http_request` or `fetch_url`
  to fetch the content
- WEBFETCH-001 Phase 0 provides the HTTP infrastructure this tool needs
- They share the same Network permission gate but are independently
  configurable

## Non-Goals

- Do not implement DDG HTML scraping — research confirmed it is unreliable
  (CAPTCHA walls, HTTP 202 traps, datacenter IP blocking).
- Do not require third-party API keys for basic functionality.
- Do not implement a built-in search engine or web crawler.

## Acceptance Criteria

- [ ] `web_search` tool is registered with Network nature.
- [ ] DuckDuckGo Instant Answer API works as default (zero config).
- [ ] Wikipedia OpenSearch API works as fallback.
- [ ] SearXNG self-hosted backend works when configured.
- [ ] Tool works out-of-the-box — no setup required for basic search.
- [ ] Results include title, URL, and snippet per result.
- [ ] Permission pipeline gates the tool; it can be disabled.
- [ ] `cargo test -p talos-tools` passes.

## Required Reads

- `docs/backlog/active/WEBFETCH-001-web-and-document-fetch-tools.md`
- `crates/talos-tools/src/` (existing tool pattern)
- `docs/decisions/010-git-search-tool-dependency-boundary.md` (ADR-010)
