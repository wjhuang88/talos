# SEARCH-001-D2-T4: Bounded Hosted MCP Search Execution

**Status**: In Progress / Claimed PROPOSED — effective only upon #1040 merge
**Type**: Bounded Rust implementation / protected Network/API
**Parent**: [SEARCH-001-D2 / #848](SEARCH-001-D2-keyless-mcp-admission.md); [SEARCH-001 / #624](SEARCH-001-zero-config-global-search.md)
**Source Issue**: [#1039](https://github.com/wjhuang88/talos/issues/1039)
**Selected Iteration**: [I339](../../iterations/I339-search-mcp-bounded-execution.md), proposed activation via #1040, ineffective until merge
**Prerequisite**: [I337 / #1015](SEARCH-001-D2-T3-mcp-offline-observation.md) Complete / Closed, Completion Commit `9eddf0c5703d77f89bd754c5bf22c5b26bc84a6f`

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6.2 |
| Work Slice | D2-T4 / #1039: bounded inactive search-only MCP execution, reviewed dependency seam and local fixtures; no provider registration, live/default traffic or rights grant |
| Claimed At | 2026-11-27 |
| Source Issue | #1039 |
| Governance Claim PR | #1040 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer directs continuing technical work; #1040 proposed atomic claim, effective only on merge after independent protected Network/API/security review, exact-head CI, both governance validators and CAS; no supplier rights granted. |
| Implementation PR | Not started |
| Last Updated | 2026-11-27 |
| Handoff / Release Condition | No implementation until finalized claim and activation merge; independent protected Network/API/security review required. |

## Objective

Provide an inactive-by-default, production-capable bounded execution policy for search-only MCP using existing protocol and invocation authority. I337 demonstrates loopback fixtures, not production guards. See [#848 refinement](https://github.com/wjhuang88/talos/issues/848#issuecomment-7212051578). Supplier rights and default admission are independent gates.

## Scope and acceptance

- Establish an acyclic dependency and API design before coding. Main talos-mcp has a normal dependency on talos-agent; talos-agent references talos-tools only as a dev-dependency. Verify the complete graph and feature combinations with cargo metadata; do not presume a production cycle. Do not directly introduce a reverse dependency without proving acyclicity. Prefer existing protocol primitives or a narrow reviewed injection boundary; no speculative crate or second protocol stack. An ADR amendment is required if public API/dependency ownership changes exceed the accepted boundary.
- Reuse Streamable HTTP framing/dispatcher as applicable, ADR-090 immutable endpoint/method/path, per-send authority and proxy snapshot. Explicitly disable redirects; no ambient credentials, startup traffic, retry, or public requests. One invocation deadline and explicit finite limits for request count, outbound serialized request bytes, inbound response bytes (per response and cumulative), decoded bytes, SSE events and parser depth. Reject an over-limit request before send; reject advertised oversize early and enforce cumulative streamed-byte limits before appending/allocating the corresponding body buffer, including error responses, absent or false Content-Length, fragmented and encoded bodies. Never collect an unbounded response before checking limits; check decoded expansion before extending decoded buffers. Cancellation and stale authorization stop further logical sends.
- Pinned Exa web_search_exa and Parallel web_search profiles; execute and strictly validate the full initialize → notifications/initialized → tools/list → tools/call lifecycle, including negotiated protocol version, unique request correlation and schema/tool identity. Fail closed on missing or invalid initialization, wrong/duplicate IDs or unexpected state; no compatibility fallback to an uninitialized call. Reject missing/malformed/duplicate/drifted selected descriptors and unauthorized tool selection. Extra advertised tools have no authority; never invoke fetch, research, paid or arbitrary discovered tools. Search-only typed projection with HTTP(S) URL validation and bounded text/results; no arbitrary content or remote tool registration, no result fetch.
- Content-free diagnostic errors, including HTTP/RPC/parse failures; never expose query, raw response, headers, tokens or credentials. Define explicit typed terminal outcomes for invalid input/protocol/version/schema/ID/result, access denial or stale authority, redirects, DNS/connect/TLS/proxy failure, 401/403, 429, other HTTP status including 5xx, request/response/decode/event budget exhaustion, deadline, cancellation and no results; no automatic retry, fallback or billing. Record exact numeric limits and mappings in the implementation design before review. Retain generic MCP compatibility, including stdio; #1033 stdio supervision and #199 generic resilience remain separate.
- Deterministic local-only Rust fixtures for actual execution, full lifecycle ordering and failure, request/response/decode budget boundaries, oversize before buffering (including error bodies and false/missing lengths), fragmented JSON/SSE, malformed/mismatched/duplicate IDs, stale authority, cancellation/deadline, proxy/redirect, forbidden tools/schema, 401/403/429/5xx, typed terminal outcomes and redaction. Reuse I337 fixtures and retain its Completion Commit. No real supplier wire claims.

- Mandatory ADR-090 §5 connection-level enforcement fixtures before protected implementation acceptance: use deterministic injected DNS/resolver and controlled local connection/proxy endpoints to prove the actual resolved and connected address is within approved policy, reject disallowed/private or mixed permitted/disallowed DNS answers and resolution/connection drift, and verify that the immutable approved proxy snapshot governs actual connections, including proxy bypass, unsupported proxy and post-approval environment/configuration changes. Verify no connection to a denied or unapproved target or proxy and no credential leakage, using observed logical sends and connection receipts rather than URL/origin checks alone. No vendor or public network requests; preserve ADR-090's existing permitted proxy semantics and generic MCP behavior. Record actual results and limitations; failure or absent evidence blocks implementation closeout.

## Non-goals

No live vendor requests, default/Auto integration, provider admission, new config/tool schema, payment, permission widening, retry framework, supplier rights conclusion or geography claims. No changes to I331/I337/I335 terminal records. #848/#624 stay Open, E/F/G/I remain gated. #1039 authorizes only this bounded technical outcome after effective claim.

## Validation and rollback

Before implementation closeout, use the repository-pinned Rust toolchain and run focused fmt/check/clippy/test for all affected crates and feature combinations, followed by the mandatory full-workspace commands `cargo fmt --all -- --check`, `cargo check --workspace --locked`, `cargo clippy --workspace --locked -- -D warnings`, and `cargo test --workspace --locked`. Run additional required native/feature-specific gates, not instead of these commands. Explicitly execute generic MCP compatibility regression tests, including real local stdio initialize/list/call and existing lifecycle/cleanup behavior, and existing HTTP/SSE paths; preserve generic behavior without taking over MCP-002/#1033. Record named tests, commands, exit results, skips and unavailable checks; missing mandatory evidence blocks completion. Both governance validators, applicable native exact-head CI, independent protected Network/API/security review and merge-time CAS are required. Document numeric limits, rollback and outstanding release gates. Zero vendor/public requests in CI for this slice. Roll back the isolated execution path without altering the existing generic MCP or default web_search route. Document architecture, limits, errors, test evidence and release residuals.

## Required reads

AGENTS.md, docs/sop/AGENT-COLLABORATION.md, docs/sop/START-ITERATION.md, ADR-085/090, I337 fixture reference, talos-mcp client transport/dispatcher/facade, talos-agent and talos-tools Cargo manifests, existing prepared SearchTransport and permission contracts.
