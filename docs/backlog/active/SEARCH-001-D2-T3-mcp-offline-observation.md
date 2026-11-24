# SEARCH-001-D2-T3: Offline MCP Search Observation Fixtures

**Status**: Ready / Unclaimed — draft intake only

| Field | Value |
|---|---|
| Story ID | SEARCH-001-D2-T3 |
| Parent | [SEARCH-001-D2 / #848](SEARCH-001-D2-keyless-mcp-admission.md) |
| Epic | [SEARCH-001 / #624](SEARCH-001-zero-config-global-search.md) |
| Source Issue | [#1015](https://github.com/wjhuang88/talos/issues/1015) |
| Selected Iteration | [I337](../../iterations/I337-search-mcp-offline-observation.md), proposed only |
| Prior evidence | [D2.2 / I331](../../iterations/I331-search-keyless-mcp-rights-evidence.md), Complete / Closed |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | D2-T3 / #1015: offline synthetic MCP search observation and replay only; no external query, runtime backend, provider admission or permission changes |
| Claimed At | Not applicable |
| Source Issue | #1015 |
| Governance Claim PR | Pending |
| Authorization Mode | Not applicable |
| Authorization Evidence | Maintainer technical direction 2026-11-23; independent review and applicable exact-head checks required before activation |
| Implementation PR | Not started |
| Last Updated | 2026-11-24 |
| Handoff / Release Condition | No implementation before finalized claim merge; owner-first closure requires tested existing implementation commit; #848 remains open |

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
