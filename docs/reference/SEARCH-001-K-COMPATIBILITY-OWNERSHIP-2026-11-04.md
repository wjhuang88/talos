# SEARCH-001-K Compatibility Ownership Inventory — 2026-11-04

> Evidence scope: source inspection of Talos `main` at
> `eed60f5a8587afc1a2cbc2483ee26ac2eb39ea08` and pinned upstream
> `Rust-Framework/rust-agent-framework` at
> `b30180a03609580e8b28dfb682284cb2ba249fba`.
> Intake evidence only: not a new provider admission, implemented code change, live search proof,
> legal permission determination, CI verdict or claim of `rust-websearch` removal.
> Owner: [SEARCH-001-K / #850](../backlog/active/SEARCH-001-K-compat-fallback-ownership.md).

## 1. Code truth: this is already Talos-owned for the production compatibility path

| Responsibility | Actual module | Current status |
|---|---|---|
| Tool factory, explicit configuration, one router invocation | `src/search_backend/production.rs`, `router_tool.rs`, `selection.rs` | Production-owned; Auto zero-key, premium/self-hosted opt-in |
| Request validation and invocation-bound network authority | `src/search_backend/{composition,prepared_invocation}.rs`, `transport.rs` | Talos-owned ADR-090 scope |
| Four internal source request shapes | `src/search_backend/transport/compatibility.rs` | Lite POST, Instant Answer GET, HTML GET, Bing CN GET |
| Four-source fallback order | `src/search_backend/transport/compatibility/chain.rs` | Lite → Instant Answer → HTML → Bing CN; 15s per-source, bounded network retry + 500ms wait |
| Query-free reachability and TTL | `src/search_backend/transport/compatibility/probe.rs` | Request-time HEAD then conditional GET; 3s timeout, 30s cache; no startup traffic |
| Result normalization and URL guards | `src/search_backend/transport/compatibility/admission.rs` | Talos-owned |
| Search response formats | `src/search_backend/{ddg_parser,instant_answer,bing_parser}.rs` | Talos-owned parser, bounded bytes/fields/results |
| Final compatibility knowledge fallback | `src/search_backend/transport/compatibility/wikipedia.rs` | Wikipedia OpenSearch; **not** a general web provider |
| Proxy/permission/deadline protection | `src/search_backend/transport.rs`, `src/search_backend/production.rs` | Shared budget, prepared transport; Auto 135s/24 physical units; opt-in explicit-plus-Auto 150s/26 |
| Behavior equivalence | `src/search_backend/transport/compatibility/chain/legacy_parity_tests.rs` | Includes actual `WebSearchTool` vs protected path query-recipient sequence tests |
| Dormant rollback | `src/web_search.rs` | Uses upstream `rust_websearch::search()` and `rust_websearch::SearchConfig`; **not** called by protected production factory |

All relative paths above start in `crates/talos-tools/`.

The production protected chain already **ported the method**; a second port of the coordinator
would create competing routing/caching/budget semantics and is not justified.

## 2. Remaining upstream source-level dependency

| Production call site | Upstream symbol | Suggested private target |
|---|---|---|
| `src/search_backend/ddg_parser.rs` | `html_utils::clean_html`, `html_utils::resolve_duckduckgo_url` | `search_backend/compatibility_helpers::html` |
| `src/search_backend/bing_parser.rs` | `html_utils::clean_html` | same |
| `src/search_backend/instant_answer.rs` | `html_utils::decode_html_entities` | same |
| `src/search_backend/transport/compatibility.rs` | `anti_detection::random_user_agent` | `search_backend/compatibility_helpers::user_agent` |
| `src/search_backend/transport/compatibility/chain.rs` | `anti_detection::random_user_agent` | same |

Historical rollback and parity calls stay under `src/web_search.rs` and test modules.
Thus K1 **does not remove the Cargo dependency or prove default-build dependency independence**.
A full build-graph replacement requires a separately gated later decision under SEARCH-001-I.

### Upstream primary-source anchors

- [HTML helper implementation (pinned)](https://github.com/Rust-Framework/rust-agent-framework/blob/b30180a03609580e8b28dfb682284cb2ba249fba/crates/websearch/src/html_utils.rs)
- [12-entry User-Agent pool (pinned)](https://github.com/Rust-Framework/rust-agent-framework/blob/b30180a03609580e8b28dfb682284cb2ba249fba/crates/websearch/src/anti_detection/user_agent.rs)
- [Upstream coordinator (pinned)](https://github.com/Rust-Framework/rust-agent-framework/blob/b30180a03609580e8b28dfb682284cb2ba249fba/crates/websearch/src/searcher.rs)
- Existing MIT text: `crates/talos-tools/LICENSES/rust-websearch-MIT.txt` (Copyright 2026 Rust-Framework Contributors).
- Existing provenance and archive SHA-256: `docs/reference/SEARCH-001-C-COMPATIBILITY-HEADERS.md` and source headers of Talos parsers.

### Pinned upstream helper behavior that fixture tests must cover

- `decode_html_entities` uses a sequential replacement chain for `&amp;`, `&lt;`,
  `&gt;`, `&quot;`, `&#x27;`, `&#39;`, `&nbsp;`. Do not silently introduce
  broader entity interpretation as if it were byte-for-byte parity.
- `clean_html` removes tags using an `in_tag` flag, decodes listed entities,
  collapses whitespace and trims. Talos should retain the source-result safety bounds.
- `resolve_duckduckgo_url` decodes the listed HTML entities; `/l/?uddg=` handling
  uses minimal percent decoding and `+` to space; protocol-relative `//host/path`
  becomes `https://host/path`. Malformed encodings and non-ASCII must be explicitly
  tested, including any **intentional fail-closed deviations**.
- `random_user_agent` chooses one of twelve fixed upstream browser UA strings using
  `rand 0.8`; compatibility snapshots already validate length/bytes and freeze a
  single draw during preparation. Migrating to owned pool selection must not pull in
  ambient env-based provider selection or change request authorization.

## 3. Security/behavior parity boundary

- Do not copy upstream coordinator/global mutable rate limiter, internal `probe_all`
  network client, extra SearXNG endpoint discovery, retry systems, arbitrary new domains,
  or user-agent-based CAPTCHA avoidance. Talos already owns these boundaries.
- Keep original request-order fallback **within one compatibility family**. DDG Lite,
  DDG HTML, Instant Answer and Bing CN are not counted as two new independent admitted
  zero-key providers; this is compatibility with existing behavior, not a bypass of D/#644.
- Preserve existing Network permission/endpoint manifest, proxy snapshot, deadline,
  cancellation, request byte count, SSRF/redirect safeguards and optional providers' explicit-only
  policy.
- Retain the rollback and pinned library until the separate I retirement gate.
- Document all differences in results/URL semantics rather than assuming that copied helpers
  are perfect parity or that live endpoint operation is guaranteed.

## 4. Work decomposition and checkpoint

**K1**: private helpers + targeted deterministic fixtures + all protected transport/chain/headers
regressions. Goal: no direct upstream **helper calls** in protected production path.

**K2 (outside current claim and still gated)**: evaluate full build dependency/legacy API
replacement after I's prerequisite evidence. Do not quietly expand K1 to dependency deletion.

Status now: **K1 is proposed, not implemented**. Formal scope selection/claim and applicable
review are required before code implementation. E/F admission, G region acceptance, D2 vendor
research and I retirement are separate.
