# SEARCH-001-C: Backend Endpoint Authority Plan

> Status: Architecture/evidence proposal, not an accepted permission API or shipped behavior.
> Owner: I293 / SEARCH-001-C / #643, effective claim #664; parent #624 remains open.
> Audited baseline: #708 merge `f036111f6e969feea47ad784f3cb3effd7e20a33`.
> Decision boundary: accepted ADR-085, existing Agent-owned permission pipeline; protected
> permission/API changes require separate coordinated design, authorization and review.

## Two Separate Gaps

#708 proves that the actual registered WebSearchTool inherits an untyped Network facet. With
the default permission engine, Ask cannot compile Once/Session proposals: MissingResource(0).
Its configured Allow control is not a repair recommendation or evidence for every host policy.

The same baseline exposes a second, source-level boundary:

| Source at audited baseline | Observed behavior |
|---|---|
| `crates/talos-permission/src/grant.rs::compile_facets` | Network supports Domain/Remote proposal scopes. Domain parses a host; it does not bind URL scheme, effective port, method, path or redirect destinations. |
| `crates/talos-permission/src/session.rs::admit_locked` | Checks pending Session/revisions/context, exact tool/provenance/facets and request fingerprint, then builds execution authorizations. |
| `crates/talos-permission/src/lib.rs::execution_authorizations` | Produces Path authorizations only; non-path facets produce no execution authorization. |
| `crates/talos-core/src/tool/authorization.rs::ToolExecutionAuthorization` | Current public constructor/checker are for_path/authorizes_path, not an endpoint capability. |
| `crates/talos-core/src/tool/agent_tool.rs` | Default execute_authorized delegates to execute. The raw prepared route takes BrowserRawArguments and has a separate browser-oriented boundary. |
| `crates/talos-tools/src/web_search.rs` | Does not override permission_profile or execute_authorized; its clients have per-request timeouts but no search-specific redirect/endpoint authority check. |

Therefore adding a Domain facet may permit proposal compilation but cannot alone prove that
the approved endpoint bounds every outbound request. This packet does not exercise that repair
or assert an exploit, universal host failure, transport enforcement or full C-V10 acceptance.
Host scope also cannot distinguish HTTP from HTTPS or two ports at the same host.

## Recommended Shape: Prepare, Authorize, Execute The Same Plan

Prepare one owned, invocation-bound private search plan before approval. These are proposed
responsibilities, not new public Rust types or accepted trait changes:

| Boundary | Required invariant |
|---|---|
| Prepare | Validate query/limits locally; snapshot explicit selection, admitted route IDs, configuration generation and a bounded destination manifest. No HTTP, DNS, probe or paid usage. |
| Authorize | Present truthful query exposure and possible hedge/fallback destinations; use the authoritative permission engine for the whole plan. Policy Allow still needs endpoint-bounded execution; Deny cannot be discarded to salvage a partial grant. |
| Admission fence | Bind the plan to this request, configuration generation and current permission state. Changed selection, endpoint, provenance or stale authority fails closed and requires new admission. |
| Dispatch | Consume this plan once; the router may rank only its bounded authorized candidates. Health may reorder that set but cannot add a destination or paid route. No recomputation from mutable global configuration after approval. |
| Transport | Validate each initial request and each redirect before sending query/credentials. Enforce the same approved destination identity at the client/connection boundary, not just in a display preview. |
| End/drop | Release owned work and authority. No grant reuse across Sessions or shared last-plan cache; deadline/cancellation remain separate C integration gates. |

The destination manifest should represent canonical origin (scheme, host, effective port),
allowed method and backend-defined request-path constraints, plus an explicit redirect policy.
Query values are invocation input, never destination authority. Bound manifest size/redirect
hops under the same enclosing budget; numeric bounds need later implementation review.
Request identity must remain bound without exporting raw queries, keys, signed URLs or their
digests into health, logs or durable telemetry. Credential/configuration changes invalidate
preparation; credentials remain execution-only, not approval text.

Do not smuggle a plan, endpoint, token or deadline into the model-facing web_search input.
Do not change schema/output, contribution identity, Network nature or Provider vocabulary here.
Do not add a domain wildcard, synthetic URL, blanket Allow or second permission evaluator.

## Compatibility And Transport Disposition

| Route class | What must be covered before claiming endpoint authority |
|---|---|
| Talos-authored Tavily | Fixed search destination/method and actual redirect behavior; a discovered key never supplies usage consent. Explicit-selection migration remains coordinated with H. |
| Explicit SearXNG | Validate the actual configured origin and search path before approval. Private/self-hosted endpoints require explicit reviewed policy, not silent rejection or automatic trust. No default relay. |
| Wikipedia compatibility fallback | If retained for compatibility, disclose and authorize it in the frozen plan before its first request. Do not activate it after a narrower grant or count it as a native generic route. |
| rust-websearch compatibility | Treat as a compound route. Its audited search path includes request-time probes, DDG variants, Bing CN and configured SearXNG, with internal retry. A DuckDuckGo-only facet is not truthful coverage. Exact destinations and client/redirect control remain qualification gates; no new source/transport proof is claimed here. |
| Future native zero-key backends | D/E/F must supply admitted endpoint/parser provenance and an enforceable manifest. This document admits no provider or region and does not retire compatibility. |

The pinned upstream call-path findings are in the [router integration plan](SEARCH-001-C-ROUTER-INTEGRATION-PLAN.md).
They are inherited source evidence, not newly recovered dependency bytes or live network results.
An unknown/unbounded compatibility destination set cannot be promoted as endpoint-enforced.
Do not silently disable the current default in this architecture stage; record a migration decision
before any production route is restricted, adapted or removed.

Origin checks alone are not SSRF proof. DNS rebinding, resolved addresses, proxies, redirects,
scheme downgrade and self-hosted exceptions need actual connection-boundary evidence. Coordinate
those controls with network/security owners without taking over NET-001 retry/backoff/breaker work.
Returned result URLs authorize neither fetch_url nor follow-up browsing. A denied redirect must
not leak a query/credential to its target; dropping a future cannot undo an already sent request.

## Carrier Decision Gate

| Option | Disposition |
|---|---|
| Add Domain facets to current WebSearchTool | Useful future metadata characterization; insufficient as the complete endpoint-enforcement design. |
| Add an invocation-bound plan/authority carrier through real Agent dispatch | Recommended design direction. Cross-crate representation, resolver projection, stale checks, wrappers and standalone execution must be reviewed together before implementation. No public API chosen here. |
| Reuse the browser raw prepared route unchanged | Not selected: browser raw arguments, closed failure/operation vocabulary and dedicated authority semantics are not a generic search permission escape hatch. Any generalization needs its own API/security decision. |

PERM-007-H is currently Refinement/Unclaimed. Supply this search inventory to its separately
claimed all-built-in review work; do not activate it or broaden I300's Git slice. Manual approval,
Auto resolution and configured Allow must share the authoritative fence and concrete execution
bounds. Unsupported enforcement needs an explicit migration/limitation decision, not a fake grant.

## Required Acceptance Before Production Wiring

These rows are planned tests, not implemented fixtures or passing results. Full C acceptance
remains Pending; existing #708 state tests do not close them.

| Gate | Required observable evidence |
|---|---|
| EP-01 | Real registered query schema unchanged; preparation produces zero outbound requests and exposes no credential. |
| EP-02 | Real Agent Ask/Allow/Deny paths consume the same plan; rejected/stale admission produces zero tool/transport starts. |
| EP-03 | Every possible hedge/fallback/probe request is inside the bounded approved manifest; adding an endpoint or changing config after approval cannot send it. |
| EP-04 | Same-host scheme/port, path/method changes and concurrent invocations cannot substitute authority; no cross-Session reuse or shared cached plan. |
| EP-05 | Controlled redirect fixtures prove no query/credential reaches an unapproved target, and preserve explicit self-hosted behavior without bypass. |
| EP-06 | Connection-level DNS/proxy/address fixtures establish actual enforcement separately from URL parsing. No fabricated regional availability. |
| EP-07 | Whole-plan Deny wins; no facet stripping, synthetic resource, wildcard permission or approval-deadline-as-search-budget workaround. |
| EP-08 | Real CLI/SDK/other applicable dispatch and direct/standalone calls cannot bypass the carrier; resolver/headless limitations are cause-specific. |
| EP-09 | Caller cancellation, enclosing deadline, loser drop and dependency-owned work have separate receipts; already-sent query exposure is reported honestly. |
| EP-10 | Exact-head independent permission/API/security review, locked tests, full relevant CI, compatibility rollback and owner synchronization pass before activation. |

This stage's deliverable is the auditable design/acceptance packet and #708 reconciliation only.
No production change, new permission rule/type, resolver grant, new dependency or native admission
is authorized by merging it. Actual plan/carrier/transport implementation requires the agreed
permission/API ownership and explicit production scope, then the appropriate claim and review.
