# TOOL-009: Internet Search Tool

**Status**: In Progress (I039)
**Priority**: P1
**Source**: User request 2026-06-20
**Depends on**: WEBFETCH-001 Phase 0 (for HTTP infrastructure)
**Iteration**: [I039 Network Tools & TUI Polish](../iterations/I039-network-tools-tui-polish.md)

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

Uses the `websearch` Rust crate (MIT, 8 providers) for multi-engine search
abstraction. No need to implement each backend from scratch.

| Tier | Provider | API Key? | Config | What it gives |
|---|---|---|---|---|
| **Default** | DuckDuckGo (via `websearch`) | No | 零配置 | Full web results (HTML scraping with anti-block headers) |
| **Fallback** | Wikipedia OpenSearch | No | 零配置 | Encyclopedia results |
| **Enhanced** | Tavily | Yes (free: 1K/mo) | `TAVILY_API_KEY` env | AI-optimized, LLM-ready structured results |
| **Self-hosted** | SearXNG | No | `searxng_url` in config | Full multi-engine search, user's own instance |
| **Optional** | Google CSE, Brave, Exa, Serper | Yes | Per-provider env vars | Via `websearch` crate's other providers |

**Dependency**: `websearch = "0.1"` (MIT, pure Rust, async, multi-provider
strategies: aggregate/failover/race). Already handles DuckDuckGo with
anti-block headers, result parsing, and rate limiting.

### Multi-Provider Strategy

The tool queries providers in parallel using a **race + fallback** strategy:

```
web_search("rust async tokio")
  │
  ├── DuckDuckGo (no key) ──────┐
  ├── Tavily (if key set) ───────┤ parallel
  └── SearXNG (if URL set) ──────┘
       │
       ▼
  First response wins → return results
  All fail → Wikipedia OpenSearch as last resort
```

### Configuration

Follows Talos's `api_key_env` pattern — the env var name is stored in config,
**never the key itself** (same convention as `[providers.<name>.api_key_env]`):

```toml
# ~/.talos/config.toml
[search]
# Tavily AI-optimized search (1,000 free queries/month)
tavily_api_key_env = "TAVILY_API_KEY"

# Self-hosted SearXNG instance
searxng_url = "https://search.example.com"
```

All fields optional. Tool works with zero config (DuckDuckGo + Wikipedia).

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
- [ ] DuckDuckGo works as default (zero config, via `websearch` crate).
- [ ] Wikipedia OpenSearch works as last-resort fallback.
- [ ] Tavily works when `TAVILY_API_KEY` is set.
- [ ] SearXNG self-hosted works when `searxng_url` is configured.
- [ ] Multi-provider race strategy: parallel query, first response wins.
- [ ] Tool works out-of-the-box with zero setup (DDG + Wikipedia).
- [ ] Permission pipeline gates the tool; it can be disabled.
- [ ] `cargo test -p talos-tools` passes.

## Required Reads

- `docs/backlog/active/WEBFETCH-001-web-and-document-fetch-tools.md`
- `crates/talos-tools/src/` (existing tool pattern)
- `docs/decisions/010-git-search-tool-dependency-boundary.md` (ADR-010)
