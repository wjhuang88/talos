# Iteration I337: Offline Search MCP Observation and Replay

> Document status: Active / Claimed PROPOSED — ineffective until #1016 merges
> Published plan date: 2026-11-24
> Planned objective: implement bounded offline-only evidence infrastructure for #1015.
> MVP deliverable: deterministic local synthetic Exa/Parallel search protocol tests and sanitized replay metadata.
> Baseline rule: preserve published objective; changed objectives require a new iteration ID.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6.2 |
| Work Slice | D2-T3 / #1015: offline synthetic MCP search observation and replay only; no external query, runtime backend, provider admission or permission changes |
| Claimed At | 2026-11-24 |
| Source Issue | #1015 |
| Governance Claim PR | #1016 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer authorizes Talos-side engineering, 2026-11-23; proposed #1016 requires independent technical review, exact-head applicable CI and both governance validators, merge CAS. Single maintainer; no supplier rights granted. |
| Implementation PR | Not started |
| Last Updated | 2026-11-24 |
| Handoff / Release Condition | No implementation before finalized claim merge; owner-first closure requires tested existing implementation commit; #848 remains open |

## Published baseline

Selected story [SEARCH-001-D2-T3 / #1015](../backlog/active/SEARCH-001-D2-T3-mcp-offline-observation.md); parent #848, Epic #624.

## Bounded scope and acceptance

- Reuse existing `talos-mcp` Streamable HTTP transport and facade wherever their current contract permits; otherwise isolate test-only fixtures without changing shared runtime behavior. Reuse I294 observation metadata, privacy and bounds; do not create a second production MCP protocol stack.
- Offline deterministic local/synthetic initialize, initialized notification, tools/list and search-only tools/call; JSON/SSE decoding, request ID checks, pinned Exa web_search_exa and Parallel web_search identity, synthetic input/output schema, rejection of unknown and paid tools.
- Negative coverage for malformed/truncated and oversized bodies, HTTP/JSON-RPC errors, redirects, proxy/TLS-style injected failures, deadline, cancellation, request and response budgets, and invalid result URLs. No real vendor traffic or ambient credentials.
- Replay artifacts contain synthetic marker, UTC timestamp, Talos commit, LOCAL-ONLY provenance and query digest; no real queries, credentials, cookies, personal data, result titles or snippets. Fixtures do not assert observed vendor contracts.
- Runnable offline targeted tests, applicable locked/native CI, both governance validators, independent Network/API/security review and merge-time CAS. Existing web_search schema, SearchRouter, Auto and Network/ADR-090 behavior unchanged.
- Update operator-facing fixture instructions and actual test evidence; owner-first completion and issue synchronization only after source merge. Rollback: remove test-only utility/fixtures, with no runtime migration.

## Non-goals and dependencies

I331/D2.2 and D2.1 are terminal source-only evidence; do not repeat or reopen. #848 and #624 remain Open. Exa/Parallel remain NOT ADMITTED; live acquisition, default activation, supplier access/output rights, true mainland-China/comparator validation, E/F/G/I and billing are excluded. No result-rights or region assertions. Historical #644 ZERO-ADMISSION unchanged.

Required reads: AGENTS.md; docs/sop/AGENT-COLLABORATION.md; docs/sop/START-ITERATION.md; ADR-085/090; parent D2 owner; docs/reference/SEARCH-001-D2-HOSTED-MCP-RIGHTS-DELTA-2026-11-18.md; scripts/search_provider_observation.py; crates/talos-mcp/src/client/{transport,facade}.rs.

## Selection and concurrent work

I335/#1001 and I333/#991, I331/#848 documentary slice, I332 Bedrock and I330 storage composition are delivered; do not replay. I336 reserved by open claim #1014 (authentication architecture), I334 source #1010 (CLI/SDK startup parity), #997 Desktop acceptance remain distinct and unchanged. Retain existing nonterminal I318 and I277 Review, I249 deferred Planned, I164 Paused, and other unrelated active/review/blocked owner scopes from the current Board/index. No new claimant on I331. This isolated test-only slice does not preempt their validation or alter their plans.

## Actual activation

Proposed atomic claim and activation via #1016. No effective claim or implementation until the finalized governance-only PR is independently reviewed, validated and merged. I337 distinct from I336 reserved by #1014; earlier documentary and Bing slices remain closed.

## Verification and residuals

Pending. Exact-head evidence required. #848 external admission remains blocked regardless of this offline outcome.
