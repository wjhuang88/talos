# SEARCH-001-D2-T3: Offline MCP Search Observation Fixtures

**Status**: In Progress / Claimed — effective #1016; partial implementation checkpoint

| Field | Value |
|---|---|
| Story ID | SEARCH-001-D2-T3 |
| Parent | [SEARCH-001-D2 / #848](SEARCH-001-D2-keyless-mcp-admission.md) |
| Epic | [SEARCH-001 / #624](SEARCH-001-zero-config-global-search.md) |
| Source Issue | [#1015](https://github.com/wjhuang88/talos/issues/1015) |
| Selected Iteration | [I337](../../iterations/I337-search-mcp-offline-observation.md), proposed atomic activation via #1016 |
| Prior evidence | [D2.2 / I331](../../iterations/I331-search-keyless-mcp-rights-evidence.md), Complete / Closed |

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
| Implementation PR | [#1018](https://github.com/wjhuang88/talos/pull/1018) — draft; local convergence in progress |
| Last Updated | 2026-11-24 |
| Handoff / Release Condition | Local convergence, independent review and exact-head native CI before source closeout; #848 remains open |

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

## Execution checkpoint — 2026-11-24 (Partial)

Claim #1016 became effective on main at `9eee120a23cc446413a3db2e829f201b2aace7b0` after exact-head [CI](https://github.com/wjhuang88/talos/actions/runs/45310451699) (4 applicable jobs passed, 4 scoped skips) and independent [APPROVE](https://github.com/wjhuang88/talos/pull/1016#issuecomment-7119927117). I336 is now effective under separate #1014; no overlap. Stage checkpoint `9e67dde80a7bb93d57735c9d6f6c577c1cb450cf` on `test/i337-mcp-offline-observation` preserves the locally tested research-only Python standard-library fixture companion, following I294 operator tooling convention. This is not a second production MCP stack and not a replacement for actual Rust client validation.

At this saved Python-only checkpoint, `python3 -m unittest -v test_search_mcp_offline_observation` from the scripts directory passed **4 test methods**, including 50 provider/scenario subtests (25 each Exa/Parallel) and proxy, duplicate/boolean ID, invalid metadata checks. Local and remote script/test Git blob IDs matched exactly (`d2374fef50a595a3ae82f6ef8f3a9a89dfa6bcad`, `f14cf36c662f0ccbd99c62f1206a7180f4f84ab5`). Actual HTTP only on private IPv4 loopback; synthetic JSON/SSE and input/output shapes; fixed tool identities, no external network, redirects rejected, capped response, no credential or raw query/result in evidence. TLS/proxy failures were injected labels; cancellation and deadline were pre-call simulations, not real concurrent transport cancellation. At that checkpoint Rust workspace validation was unavailable and no implementation PR had been created. Do not assert vendor wire compatibility or production admission.

### Rust transport integration checkpoint — 2026-11-24 (Partial)

The saved `test/i337-mcp-offline-observation` commits were preserved in the implementation branch. The current draft PR #1018 adds only `#[cfg(test)]` Rust code plus fixture/privacy assertions and documentation; no runtime behavior, dependency, schema, routing, permission, or provider registration changed.

- `cargo fmt --all -- --check`, `cargo clippy --locked -p talos-mcp --all-targets -- -D warnings`, and `cargo test --locked -p talos-mcp --lib`: passed; all **19 library tests** passed.
- `cargo test --locked -p talos-mcp --lib client::transport::offline_search_tests -- --nocapture`: **6 passed**. The local fixture uses the actual transport/dispatcher/facade for both pinned tool names, JSON/SSE, exact request IDs/parameters, tool-list drift/duplicates, unexpected-tool call denial, finite oversized/partial bodies, HTTP status and redirect behavior, synthetic URL filtering, in-flight timeout/cancellation, and a loopback TLS-style failure. The Python suite passes **4 methods / 50 provider-case subtests** and checks UTC timestamp, commit/hash metadata and sensitive-output absence.
- The tool-list and URL allowlists are test-only caller checks. The production facade still accepts extra descriptors and the generic transport still has no body-size/request-budget guard, follows its default policy unless a test client overrides it, and includes HTTP error body text in errors. No production guarantee is claimed; any hardening needs separate governance.
- Python proxy/TLS labels remain injected, not measured provider errors. Rust TLS-style failure uses only a local plain-HTTP sink; no Rust proxy failure or external URL is exercised. No live vendor calls, result fetches, credentials, rights, or region claims.

Full `cargo test --locked -p talos-mcp` and `PYTHONDONTWRITEBYTECODE=1 ./scripts/release_preflight.sh` were attempted. The full package command's `git_prepared` integration tests fail under the runner's inherited `GIT_*` overrides; a diagnostic rerun without them instead reports a Git maintenance executable-binding mismatch. The release preflight passed its validation/audit/check/clippy stages and reached workspace tests, where **763 passed / 6 failed** in unrelated `talos-agent` Git/auto-review cases. Do not treat workspace preflight as passed; no production Git behavior was changed. Collaboration validation passes with two warnings on unrelated SEARCH-001-D3 owner docs.

Still pending: independent Network/API/security review, exact-head applicable native CI, merge-time CAS, and owner-first source closeout. Keep the claim In Progress / Claimed; do not close #1015, #848, or #624.
