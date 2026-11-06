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
| Tool factory, explicit configuration, one router invocation | `crates/talos-tools/src/search_backend/production.rs`, `router_tool.rs`, `selection.rs` | Production-owned; Auto zero-key, premium/self-hosted opt-in |
| Request validation and invocation-bound network authority | `crates/talos-tools/src/search_backend/{composition,prepared_invocation}.rs`, `transport.rs` | Talos-owned ADR-090 scope |
| Four internal source request shapes | `crates/talos-tools/src/search_backend/transport/compatibility.rs` | Lite POST, Instant Answer GET, HTML GET, Bing CN GET |
| Four-source fallback order | `crates/talos-tools/src/search_backend/transport/compatibility/chain.rs` | Lite → Instant Answer → HTML → Bing CN; 15s per-source, bounded network retry + 500ms wait |
| Query-free reachability and TTL | `crates/talos-tools/src/search_backend/transport/compatibility/probe.rs` | Request-time HEAD then conditional GET; 3s timeout, 30s cache; no startup traffic |
| Result normalization and URL guards | `crates/talos-tools/src/search_backend/transport/compatibility/admission.rs` | Talos-owned |
| Search response formats | `crates/talos-tools/src/search_backend/{ddg_parser,instant_answer,bing_parser}.rs` | Talos-owned parser, bounded bytes/fields/results |
| Final compatibility knowledge fallback | `crates/talos-tools/src/search_backend/transport/compatibility/wikipedia.rs` | Wikipedia OpenSearch; **not** a general web provider |
| Proxy/permission/deadline protection | `crates/talos-tools/src/search_backend/transport.rs`, `crates/talos-tools/src/search_backend/production.rs` | Shared budget, prepared transport; Auto 135s/24 physical units; opt-in explicit-plus-Auto 150s/26 |
| Behavior equivalence | `crates/talos-tools/src/search_backend/transport/compatibility/chain/legacy_parity_tests.rs` | Includes actual `WebSearchTool` vs protected path query-recipient sequence tests |
| Dormant rollback | `crates/talos-tools/src/web_search.rs` | Uses upstream `rust_websearch::search()` and `rust_websearch::SearchConfig`; **not** called by protected production factory |

Paths above are repository-root-relative; production source files are under `crates/talos-tools/`.

The production protected chain already **ported the method**; a second port of the coordinator
would create competing routing/caching/budget semantics and is not justified.

## 2. Remaining upstream source-level dependency

| Production call site | Upstream symbol | Suggested private target |
|---|---|---|
| `crates/talos-tools/src/search_backend/ddg_parser.rs` | `html_utils::clean_html`, `html_utils::resolve_duckduckgo_url` | `search_backend/compatibility_helpers::html` |
| `crates/talos-tools/src/search_backend/bing_parser.rs` | `html_utils::clean_html` | same |
| `crates/talos-tools/src/search_backend/instant_answer.rs` | `html_utils::decode_html_entities` | same |
| `crates/talos-tools/src/search_backend/transport/compatibility.rs` | `anti_detection::random_user_agent` | `search_backend/compatibility_helpers::user_agent` |
| `crates/talos-tools/src/search_backend/transport/compatibility/chain.rs` | `anti_detection::random_user_agent` | same |

Historical rollback and parity calls stay under `crates/talos-tools/src/web_search.rs` and test modules.
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

Historical intake status on 2026-11-04: **K1 was proposed, not implemented**. Formal scope selection/claim and applicable
review are required before code implementation. E/F admission, G region acceptance, D2 vendor
research and I retirement are separate.

## 2. Implementation candidate (2026-11-06; not yet accepted)

- Effective owner: governance #851 merged as `7b482a4243b37a4f79ef3fe460b69b0740400682`; Draft K1 implementation #860 starts from this exact commit.
- Private `crates/talos-tools/src/search_backend/compat_helpers.rs` now owns pinned upstream HTML entities/tag cleanup, DDG redirect decode and the exact 12-UA pool; the protected parser/compatibility preparation imports are rewritten to use it.
- Parity tests compare actual `rust_websearch::html_utils` outputs for deterministic input, invalid encoding and redirect edge cases; random UA tests check membership of both the pinned upstream and new local supplier. The new supplier uses existing network-enabled `uuid/v4` entropy, with unbiased rejection sampling. Exact random *sequence* is not claimed identical to upstream rand.
- No `Cargo.toml`, lockfile, legacy `WebSearchTool`, endpoint, permission, shared transport budget or fallback scheduler changed. Remaining upstream crate references in the dormant legacy and test-only upstream comparison are intentionally preserved; removing the dependency belongs to separately gated SEARCH-001-I.
- These are candidate/source facts, **not** a successful compiler, CI, native platform, security review or live-provider acceptance claim. #860 remains Draft pending those gates.

## 3. Validation evidence after 2026-11-06 branch replay

Previous code SHA `f2c978ea71bb0c6e9d603908f46c444a991a4650` passed all eight jobs in CI3847/run 42179784778. The branch was replayed onto current main to preserve independent #768/#862 and QUALITY-001 changes. This historical success does not establish a new exact-head CI result; no independent API/security review or live backend verification is claimed. Source-only helper parity remains the boundary.
