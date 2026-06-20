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

| Backend | Type | Notes |
|---|---|---|
| SearXNG | Self-hosted, open-source | Default/recommended — no API key, privacy-respecting |
| Brave Search API | Cloud API | Requires API key; good free tier |
| Tavily | AI-optimized search | Designed for agent use; paid |

Configuration in `~/.talos/config.toml`:
```toml
[search]
backend = "searxng"              # or "brave" or "tavily"
base_url = "https://search.example.com"  # for SearXNG
api_key_env = "BRAVE_API_KEY"    # for Brave/Tavily
```

If no backend is configured, the tool returns a clear error:
"web_search is not configured. Set [search] in ~/.talos/config.toml."

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

- Do not implement a built-in search engine or web crawler.
- Do not scrape search result pages (use structured APIs only).
- Do not auto-search without explicit agent invocation.
- Do not cache search results across sessions in v1.

## Acceptance Criteria

- [ ] `web_search` tool is registered with Network nature.
- [ ] SearXNG backend works with user-configured base URL.
- [ ] Brave Search API backend works with API key.
- [ ] Tool returns error when no backend is configured.
- [ ] Output format includes title, URL, and snippet per result.
- [ ] Results are truncated to max_results (default 10, max 20).
- [ ] Permission pipeline gates the tool; it can be disabled.
- [ ] `cargo test -p talos-tools` passes.

## Required Reads

- `docs/backlog/active/WEBFETCH-001-web-and-document-fetch-tools.md`
- `crates/talos-tools/src/` (existing tool pattern)
- `docs/decisions/010-git-search-tool-dependency-boundary.md` (ADR-010)
