# SEARCH-001-C Router Integration And Acceptance Plan

> Status: Architecture/evidence proposal; no runtime implementation or C acceptance.
> Owner: SEARCH-001-C / I293 / #643; effective claim #664.
> Audited baseline: `41381f8055b66d54de42988e2b936a12e5af3ac3` (2026-10-09).
> Decision authority: accepted [ADR-085](../decisions/085-talos-owned-search-routing-boundary.md).

This packet connects the accepted routing policy to the current call path. It specifies the
next implementation and verification work without changing production search, public APIs,
provider admission, permissions, dependencies or configuration. Numeric hedge/health parameters
and the execution-context integration remain proposed work, not accepted or shipped behavior.

## Code-Truth Audit

The source links below pin the audited commit; later implementation must refresh this inventory.

| Evidence | Current source | Observed boundary / implication |
|---|---|---|
| C-001 | [Private backend seam](https://github.com/wjhuang88/talos/blob/41381f8055b66d54de42988e2b936a12e5af3ac3/crates/talos-tools/src/search_backend.rs) | Request carries query and result count, not deadline/cancellation. Errors distinguish only NotConfigured, InvalidResponse and RequestFailed. ADR-085's richer taxonomy is still absent. |
| C-002 | [Compatibility adapters and execute_search](https://github.com/wjhuang88/talos/blob/41381f8055b66d54de42988e2b936a12e5af3ac3/crates/talos-tools/src/web_search.rs) | Configured adapters enter the same unstaggered race; an early ready branch can win before another is polled. First completion wins; errors trigger Wikipedia immediately, and malformed URLs are not filtered. Error strings erase transport/status detail before classification. |
| C-003 | [AgentTool execution methods](https://github.com/wjhuang88/talos/blob/41381f8055b66d54de42988e2b936a12e5af3ac3/crates/talos-core/src/tool/agent_tool.rs) | Parsed-JSON execution receives input and, on the authorized path, grants. It receives no caller deadline/token. An internal router context cannot by itself establish caller-context propagation. |
| C-004 | [Agent tool dispatch](https://github.com/wjhuang88/talos/blob/41381f8055b66d54de42988e2b936a12e5af3ac3/crates/talos-agent/src/tool_execution.rs) | Permission deadlines bound approval, not search execution. Dispatch directly awaits the tool after admission/ledger reservation; do not reuse the approval deadline as a search budget. |
| C-005 | [Session turn forwarding](https://github.com/wjhuang88/talos/blob/41381f8055b66d54de42988e2b936a12e5af3ac3/crates/talos-agent/src/session/turn.rs) | Cancellation aborts and joins agent_task. This drops owned execution futures; it does not establish cleanup of tasks spawned inside a dependency. The sandbox cleanup receipt is not a search transport receipt. |
| C-006 | [Agent cancellation_token](https://github.com/wjhuang88/talos/blob/41381f8055b66d54de42988e2b936a12e5af3ac3/crates/talos-agent/src/configuration.rs) | Returns a new token for caller coordination. Merely creating/cancelling it does not inject it into a running WebSearchTool. |
| C-007 | [Search request methods](https://github.com/wjhuang88/talos/blob/41381f8055b66d54de42988e2b936a12e5af3ac3/crates/talos-tools/src/web_search.rs) | Tavily/SearXNG client timeouts are 15s; Wikipedia is 10s. These are individual request limits, not one enclosing search deadline. rust-websearch delegates to an opaque coordinator; its cancellation/retry internals are not established by this audit. |
| C-008 | [NET-001 owner](https://github.com/wjhuang88/talos/blob/41381f8055b66d54de42988e2b936a12e5af3ac3/docs/backlog/active/NET-001-network-resilience-policy.md) | Intake / Unclaimed. No accepted shared retry/circuit implementation is available to call. C can plan distinct-backend failover; it cannot invent or claim that generic transport policy. |

The #681 tests characterize the current select mechanism with disposable futures. They do not
drive C-002 through production adapters, qualify rust-websearch internals, or close C-003/C-005.

## Proposed Router Execution Rules

The future router stays private in talos-tools. Candidate eligibility precedes ranking:
only admitted auto routes or explicitly selected optional routes participate. Credential discovery
is not selection. C must coordinate its activation with H's explicit-selection contract; preserving
B's current environment-key behavior is not evidence that ADR-085's target opt-in rule is met.
Wikipedia stays outside the generic candidate list, and opaque compatibility identity cannot count
as a native independent route. D/E/F own native admission; this packet admits none.

| Event | Proposed transition | Bound / observable acceptance |
|---|---|---|
| Invocation | Snapshot eligible candidates, order by bounded observed health, start one | No startup I/O; no locale/country branch; each candidate starts at most once per invocation |
| Hedge delay elapses while primary pending | Start the next candidate if a slot and time remain | Proposed maximum two in flight; actual delay and cap require C review/measurements |
| Candidate fails, parses empty, or yields no admissible URL | Keep another pending candidate alive; start the next eligible candidate if capacity remains | No same-backend replay or retry/backoff loop; finite candidate set bounds total attempts |
| Candidate returns admissible results | Return that candidate's normalized bounded results and drop remaining owned work | Preserve source identity/order; do not combine unrelated backend lists into an invented source |
| All eligible candidates terminate without valid success | Return typed aggregate failure | Distinguish no eligible route from all attempted routes failing; no implicit premium/Wikipedia activation |
| Enclosing deadline expires or caller cancellation is observed | Stop launches, stop awaiting success, release owned work | One terminal outcome; do not reset the overall budget for a hedge/fallback |
| Router future is externally dropped | Owned work is dropped even without a returned typed outcome | Explicit-token cancellation and external drop are separate tests; spawned work needs its own ownership evidence |

Result admission must parse an absolute HTTP/HTTPS URL with a host, reject malformed/unsupported
schemes and credential-bearing URLs, retain at least one admissible result, and bound the output.
This is a proposed C result policy requiring compatibility/security review before activation.
Displaying a result is not permission to fetch it: endpoint/redirect/SSRF enforcement remains a
separate network boundary and must preserve existing Network permission. Never validate result
URLs by fetching them. A mixed response can retain valid entries; an entirely empty or rejected
response cannot win. These rules are future tests, not present compatibility behavior.

## Error And Health Ownership

Capture typed outcomes at the adapter boundary before formatting strings. In particular, classify
reqwest timeout/connect/status and parse errors from their actual values, not substring guesses.
Opaque upstream failures stay explicitly unknown/request-failed until the dependency supplies
reliable evidence; do not relabel them as DNS, challenge or rate-limit outcomes.

| Outcome | Search selection effect proposed for C | Ownership limit |
|---|---|---|
| Unreachable / timeout / HTTP status / rate limit / challenge | Record bounded outcome and latency; try a distinct eligible candidate within budget | NET-001 owns retry, server-delay hints, backoff and breaker accounting |
| Parse drift / invalid results | Mark response unusable and fail over | An HTTP 200 alone cannot establish challenge or success; adapter evidence is required |
| Legitimate no results | Continue to another eligible route without treating it as connectivity failure | Do not conflate query-specific absence with endpoint outage |
| Not configured / auth failure | Exclude unavailable selection or retain typed failure | Never discover a key and silently spend quota |
| Cancelled / enclosing deadline / loser dropped after success | Terminal control/cleanup outcome | Do not penalize a losing backend as unreachable or rate-limited |

Health contains only bounded backend IDs, coarse outcomes, latency and monotonic observation/expiry
times. No query, result URLs, raw error/body, user IP or credential belongs there. Expired evidence
returns to static eligible order; unknown endpoints must not create an unbounded map. A later user
search can reevaluate stale routes; no background probe or independent transport breaker. Exact TTL,
capacity, update rules and network/proxy-change invalidation remain C measurement/design work.

## Caller Context Integration Gate

First prototype the private router with an explicit internal monotonic deadline and cancellation
handle. This permits deterministic fixtures but is not end-to-end propagation evidence. Before
runtime wiring, resolve C-003 with the owners of AgentTool/composition roots: either use an agreed
caller execution-context seam that wrappers forward, or explicitly document the bounded local
search budget plus caller-owned future-drop behavior as partial acceptance. Do not silently change
a public trait, smuggle control fields into model input, or claim permission approval TTL as the
execution deadline. Any public API change needs the repository's ADR/migration review.

The final path must cover actual dispatch, permission wrappers and Session cancellation; direct
Agent callers need their own context/drop verification. Cancellation that destroys the future
cannot promise a returned SearchBackendError::Cancelled. Binary/Session terminal evidence must
reflect that distinction. Audit rust-websearch task ownership separately; keep unresolved cleanup
as a compatibility limitation, not proof of transport cancellation.

## Verification Matrix And Stage Order

These rows are pending acceptance, not tests already executed. Prefer controlled readiness,
paused time and counted starts/drops to live network timing. No extra provider setup is required
for deterministic synthetic adapter fixtures.

| Row | Required fixture / integration | Expected assertion |
|---|---|---|
| C-V01 | Fast typed failure; second route pending then valid | Router waits for valid route, retains its source, does not trigger legacy knowledge fallback |
| C-V02 | Empty, invalid-only and mixed valid/invalid response | First two cannot win; mixed response returns only bounded admissible entries |
| C-V03 | All-fail / no-eligible-route | Finite starts, typed terminal failure, no paid/Wikipedia activation |
| C-V04 | Hedge timer / immediate failure / three candidates | First launch before delay, capped concurrency, each route launched once, no transport replay |
| C-V05 | Deadline includes primary, hedge and result validation | One monotonic budget; no launch after expiry; owned tasks/futures released |
| C-V06 | Pre-cancelled and in-flight token; external future drop | Cancellation ends work; external drop verified separately; no synthetic timeout error for cancellation |
| C-V07 | Winner versus pending loser; intentionally spawned fixture | Owned loser dropped; spawned fixture aborted/joined by an owner, never merely assumed stopped |
| C-V08 | Health expiry, no-results and losing-route cleanup | Bounded query-free state; no false outage penalty; stale evidence reevaluated on user search |
| C-V09 | Missing config, ambient key and explicit selection | Eligibility follows reviewed selection policy; key-only presence cannot authorize metered usage |
| C-V10 | Real tool dispatch/permission wrapper and Session interruption | Network admission preserved; caller controls/drop reach search; no post-terminal owned work |
| C-V11 | Rebuilt talos binary, adapter wiring and rollback | Stable web_search schema/output; actual router path reached; compatibility rollback exercised |

Implementation sequence within I293: (1) agree context/selection seams and parameter bounds;
(2) private typed router plus deterministic fixtures; (3) adapters classify actual failures;
(4) wire tool/caller path and prove C-V10/C-V11 with rollback. Stage (2) alone cannot complete C.

## Partial Offline Scheduling Evidence — 2026-10-10

`crates/talos-tools/tests/i293_router_policy_experiment.rs` is a disposable twelve-case model;
production never calls it. Already-eligible lazy candidates return synthetic Failed/Empty/Invalid/
Valid outcomes. Paused Tokio time and owned-future counters verify finite starts, a fixture cap
of two, delayed hedging, immediate failure replacement, valid-winner selection, deadline and
cancellation boundaries, and local drop cleanup. Candidate polls that make cancellation ready
are checked again before replacement/hedge launch and before returning a winner.

This is partial evidence for C-V01/C-V03/C-V04 and the scheduling/owned-future portions of
C-V05/C-V06/C-V07. It does not test actual parsing or mixed-result filtering (C-V02), validation
budget, spawned task ownership, health, explicit-selection policy, real caller dispatch, binary
wiring or rollback. All full acceptance rows above remain Pending. Numeric timings, cap, oneshot
sender-drop interpretation and deadline/success tie ordering are experimental parameters;
simultaneous cancellation/deadline precedence is not accepted policy. Consult I293 for exact
source provenance and validation. No production router or native admission follows from this model.

### Pinned compatibility dependency source audit

Read-only local audit of Cargo.lock's `rust-websearch 0.1.2`, registry checksum
`06b90d9a9ad9e7e347c4cd85c3fb37189256703db4fab1d6c6fb963ae1eed0ee`, establishes these
source-level facts (no live search/probe was executed):

- Talos `crates/talos-tools/src/web_search.rs` initializes SearchConfig with dependency defaults.
  Its dependency file `types.rs` defaults to a 15-second request timeout and 3000-ms probe timeout.
- Dependency `searcher.rs::search` awaits a global rate-limiter, then request-time `probe_all` when
  enabled; reachable routes are attempted sequentially. DDG uses Lite, Instant Answer and HTML;
  then Bing CN and configured SearXNG are compatibility fallbacks. This is existing dependency
  behavior, not the proposed Talos router, generic native admission or a startup probe.
- Dependency `probe.rs` owns parallel probe futures through `join_all`, caches reachability for 30 seconds,
  and uses HEAD with a possible GET fallback. The search coordinator/probe code inspected does
  not spawn application tasks. This does not prove that reqwest/DNS internals stop on drop.
- Lite/HTML/Bing/SearXNG call dependency `anti_detection/mod.rs::retry_request` with one retry, on
  Timeout/Network errors, including a 500-ms wait; Instant Answer directly awaits its request.
  Per-request limits therefore do not establish one enclosing search budget. A router must not
  silently add transport replay around this compatibility chain.
- The subprocess spawn found in dependency `fetcher.rs` belongs to the separate `fetch_page` path;
  the inspected search chain does not call that path. This is a bounded call-path observation,
  not blanket dependency task/process isolation evidence.

These findings refine the earlier opaque-internals audit without closing C-V05/C-V07/C-V10.
Transport cancellation, caller budget propagation and production integration remain unproven.
Every behavioral stage requires fresh local convergence, exact-head CI and applicable API/security
review. The current session authorizes this packet and test-only work; it does not authorize these
production stages merely because their plan is written down.

## Current Disposition

#681 is merged at `41381f8055b66d54de42988e2b936a12e5af3ac3`. Its exact-head CI3057 passed
full macOS release preflight and Windows workspace/governance checks; Linux Desktop validation
also passed. No CI skip was introduced for the locally restricted Unix socket fixture. Independent
agent technical review and single-maintainer merge evidence are recorded on #681. This closes
that test-only stage and its deferred local-host validation, not any pending C-V row above.

C/I293 remains Active / Claimed, Completion Pending. D/I294 remains separately owned; no native
candidate is admitted and E/F remain gated. #643 and Epic #624 stay open. Provider correspondence
is not a prerequisite for this code-truth audit or offline tests. NET-001's unclaimed intake does
not authorize C to take over its generic resilience scope.
