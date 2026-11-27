# Iteration I339: Bounded Hosted MCP Search Execution (SEARCH-001-D2-T4)

> Document status: Active / Claimed PROPOSED — effective only upon #1040 merge
> Published plan date: 2026-11-27
> Planned objective: implement a bounded, inactive-by-default search-scoped MCP execution path; no admission or external traffic.
> MVP deliverable: runnable local-only Rust execution and negative fixtures, with approved dependency and authority boundary.
> Baseline rule: preserve published objective; changed objectives require a new iteration ID.

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

## Published baseline

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| SEARCH-001-D2-T4 / #1039 | #848 / #624 | Ready | I337 Complete / Closed; ADR-085/090 | Bounded search-only execution capability, not registered |

### Scope and acceptance

- Establish an acyclic dependency and API design before coding. Main talos-mcp has a normal dependency on talos-agent; talos-agent references talos-tools only as a dev-dependency. Verify the complete graph and feature combinations with cargo metadata; do not presume a production cycle. Do not directly introduce a reverse dependency without proving acyclicity. Prefer existing protocol primitives or a narrow reviewed injection boundary; no speculative crate or second protocol stack. An ADR amendment is required if public API/dependency ownership changes exceed the accepted boundary.
- Reuse Streamable HTTP framing/dispatcher as applicable, ADR-090 immutable endpoint/method/path, per-send authority and proxy snapshot. Explicitly disable redirects; no ambient credentials, startup traffic, retry, or public requests. One invocation deadline and explicit finite limits for request count, outbound serialized request bytes, inbound response bytes (per response and cumulative), decoded bytes, SSE events and parser depth. Reject an over-limit request before send; reject advertised oversize early and enforce cumulative streamed-byte limits before appending/allocating the corresponding body buffer, including error responses, absent or false Content-Length, fragmented and encoded bodies. Never collect an unbounded response before checking limits; check decoded expansion before extending decoded buffers. Cancellation and stale authorization stop further logical sends.
- Pinned Exa web_search_exa and Parallel web_search profiles; execute and strictly validate the full initialize → notifications/initialized → tools/list → tools/call lifecycle, including negotiated protocol version, unique request correlation and schema/tool identity. Fail closed on missing or invalid initialization, wrong/duplicate IDs or unexpected state; no compatibility fallback to an uninitialized call. Reject missing/malformed/duplicate/drifted selected descriptors and unauthorized tool selection. Extra advertised tools have no authority; never invoke fetch, research, paid or arbitrary discovered tools. Search-only typed projection with HTTP(S) URL validation and bounded text/results; no arbitrary content or remote tool registration, no result fetch.
- Content-free diagnostic errors, including HTTP/RPC/parse failures; never expose query, raw response, headers, tokens or credentials. Define explicit typed terminal outcomes for invalid input/protocol/version/schema/ID/result, access denial or stale authority, redirects, DNS/connect/TLS/proxy failure, 401/403, 429, other HTTP status including 5xx, request/response/decode/event budget exhaustion, deadline, cancellation and no results; no automatic retry, fallback or billing. Record exact numeric limits and mappings in the implementation design before review. Retain generic MCP compatibility, including stdio; #1033 stdio supervision and #199 generic resilience remain separate.
- Deterministic local-only Rust fixtures for actual execution, full lifecycle ordering and failure, request/response/decode budget boundaries, oversize before buffering (including error bodies and false/missing lengths), fragmented JSON/SSE, malformed/mismatched/duplicate IDs, stale authority, cancellation/deadline, proxy/redirect, forbidden tools/schema, 401/403/429/5xx, typed terminal outcomes and redaction. Reuse I337 fixtures and retain its Completion Commit. No real supplier wire claims.

### Non-goals

No live vendor requests, default/Auto integration, provider admission, new config/tool schema, payment, permission widening, retry framework, supplier rights conclusion or geography claims. No changes to I331/I337/I335 terminal records. #848/#624 stay Open, E/F/G/I remain gated. #1039 authorizes only this bounded technical outcome after effective claim.

### Planned validation

Before implementation closeout, use the repository-pinned Rust toolchain and run focused fmt/check/clippy/test for all affected crates and feature combinations, followed by the mandatory full-workspace commands `cargo fmt --all -- --check`, `cargo check --workspace --locked`, `cargo clippy --workspace --locked -- -D warnings`, and `cargo test --workspace --locked`. Run additional required native/feature-specific gates, not instead of these commands. Explicitly execute generic MCP compatibility regression tests, including real local stdio initialize/list/call and existing lifecycle/cleanup behavior, and existing HTTP/SSE paths; preserve generic behavior without taking over MCP-002/#1033. Record named tests, commands, exit results, skips and unavailable checks; missing mandatory evidence blocks completion. Both governance validators, applicable native exact-head CI, independent protected Network/API/security review and merge-time CAS are required. Document numeric limits, rollback and outstanding release gates. Zero vendor/public requests in CI for this slice.

### Documentation and rollback

Update owner, operator-facing test/limits reference, derived views; remove isolated execution path to restore unchanged generic MCP and search behavior.

## Iteration inventory and non-overlap

I337 and I335 are terminal; I331 remains terminal source-only. D3-T3b/#957 and #1000 remain externally blocked, Browser/#891 separate. Parallel I324 Git, I334 ARCH-035 and I338 Provider work retain their owners; I324 research PRs #1037 and #1043 merged; I334/#1035 and I338/#1034 remain separately owned. #1041 intake is included unchanged from #1042 in this governance batch to satisfy global reconciliation, without claiming Git implementation. No competing search claim found; repeat live inventory before merge. MCP-002/#1033 intake owns stdio supervision; #199 owns generic resilience. No parent claim or new authorization by draft creation. I339 is proposed only until atomic claim merge.

## Verification evidence

Not started; no source or local validation claim.

## Completion evidence

None. Remain non-terminal until existing implementation evidence and owner-first closeout.
