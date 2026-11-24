# Iteration I337: Offline Search MCP Observation and Replay

> Document status: Active / Claimed — effective #1016
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
| Implementation PR | [#1018](https://github.com/wjhuang88/talos/pull/1018) — draft; local convergence in progress |
| Last Updated | 2026-11-24 |
| Handoff / Release Condition | Local convergence, independent review and exact-head native CI before source closeout; #848 remains open |

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

The atomic claim and activation in #1016 merged to main as `9eee120a23cc446413a3db2e829f201b2aace7b0`. I337 is distinct from I336 reserved by #1014; earlier documentary and Bing slices remain closed.

## Verification and residuals

Pending. Exact-head evidence required. #848 external admission remains blocked regardless of this offline outcome.

## Execution checkpoint — 2026-11-24 (Partial)

Claim #1016 became effective on main at `9eee120a23cc446413a3db2e829f201b2aace7b0` after exact-head [CI](https://github.com/wjhuang88/talos/actions/runs/45310451699) (4 applicable jobs passed, 4 scoped skips) and independent [APPROVE](https://github.com/wjhuang88/talos/pull/1016#issuecomment-7119927117). I336 is now effective under separate #1014; no overlap. Stage checkpoint `9e67dde80a7bb93d57735c9d6f6c577c1cb450cf` on `test/i337-mcp-offline-observation` preserves the locally tested research-only Python standard-library fixture companion, following I294 operator tooling convention. This is not a second production MCP stack and not a replacement for actual Rust client validation.

At the saved Python-only checkpoint, `python3 -m unittest -v test_search_mcp_offline_observation` from the scripts directory passed **4 test methods**, including 50 provider/scenario subtests (25 each Exa/Parallel) and proxy, duplicate/boolean ID, invalid metadata checks. Local and remote script/test Git blob IDs matched exactly (`d2374fef50a595a3ae82f6ef8f3a9a89dfa6bcad`, `f14cf36c662f0ccbd99c62f1206a7180f4f84ab5`). Actual HTTP only on private IPv4 loopback; synthetic JSON/SSE and input/output shapes; fixed tool identities, no external network, redirects rejected, capped response, no credential or raw query/result in evidence. TLS/proxy failures were injected labels; cancellation and deadline were pre-call simulations, not real concurrent transport cancellation. At that checkpoint Rust workspace validation was unavailable and no implementation PR had been created. Do not assert vendor wire compatibility or production admission.

### Rust transport integration checkpoint — 2026-11-24 (Partial)

The saved implementation commits were preserved in draft PR #1018. Focused test-only Rust coverage passed **6 tests** using the real Streamable HTTP transport, dispatcher and facade with fixed synthetic loopback fixtures, no credentials, no proxy, and redirect following disabled. Python passed **4 methods / 50 case subtests**; metadata assertions cover UTC, synthetic/LOCAL-ONLY provenance, Talos commit format, query digest, and omission of raw query/results. `cargo fmt --all -- --check` passed.

The strict tool/schema and result-URL gates exist only in the fixture caller. Existing generic transport/facade behavior is not hardened and is not claimed safe for search: response buffering/error-body disclosure and absence of production tool allowlisting/body/request budgets remain residuals. Python proxy/TLS labels are injected; Rust's TLS-style failure is confined to a local sink. No external provider traffic or links were used.

`cargo fmt --all -- --check`, package Clippy, and all 19 `talos-mcp` library tests passed. Full package integration tests did not pass: `git_prepared` cases encountered the sandbox's inherited Git overrides, and a diagnostic run with those unset encountered Git maintenance executable-binding errors. Standard `release_preflight.sh` reached workspace tests and reported 763 passed / 6 unrelated `talos-agent` Git/auto-review failures; it is not a pass. Collaboration validation has two warnings on unrelated SEARCH-001-D3 owners. Keep the iteration Active; independent review, exact-head native CI and merge-time CAS remain pending. Do not claim vendor compatibility, admission, or issue closure.
