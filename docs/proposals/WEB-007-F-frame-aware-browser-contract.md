# WEB-007-F: Proposed Frame-Aware Browser Contract

Status: Proposed — independent API/security acceptance required.
Owner: [WEB-007-F / #618](../backlog/active/WEB-007-F-frame-aware-contract.md).
Source: [#452](https://github.com/wjhuang88/talos/issues/452),
[#618](https://github.com/wjhuang88/talos/issues/618), #520 Request B.

## Versioning And Operations

Recommend a separately negotiated `talos.browser.executor/v2` contract. V1's 20 exact operation
branches remain unchanged; unknown protocol versions fail closed. A host explicitly selects v2
before publishing its exact schema. Do not publish a permissive union or silently downgrade a
v2 call to V1. Hosts may continue using V1, which carries no frame-interaction guarantee.

V2 adds `frame-tree(tabRef)` and requires explicit tabRef/frameRef on frame-local observation
and interaction. Snapshot creates a snapshotRef; element operations require snapshotRef and
an elementRef issued inside that snapshot. Press/scroll require an explicit frame even without
an element. Tab/window lifecycle operations remain session-bound, not child-frame operations.
The final exhaustive branch table and generated-schema parity are required before API acceptance;
this proposal does not claim to be that executable schema.

## Identity And Bounded Discovery

Use opaque, unguessable, non-reused references, each at most 128 ASCII bytes, minted by trusted
bound executor state. Internally bind every reference to browser/session generation, tab identity
and epoch, frame identity and epoch, and (for elements) snapshot generation. Identity is a tuple,
not a frame name, URL, tree index or raw CDP identifier. References convey identity, never authority.
Model-supplied generation/origin assertions are not trusted evidence.

Proposed reviewable discovery budgets: at most 128 nodes, depth 16 and 64 KiB serialized output.
Return parent reference, frame reference, sanitized origin and bounded readiness state; omit frame
names, URL queries/fragments, raw DOM and driver IDs. Mark truncation explicitly; no reference may
resolve to an omitted or ambiguous node. Excess depth/size returns bounded truncation or
ResourceLimit, never an unbounded traversal. Each repeated discovery is independently authorized.
Delayed frames are Pending until a fresh discovery observes readiness; no implicit polling/retry.

Snapshots are scoped to exactly one tab/frame and bounded by the accepted WEB-007 content budgets.
Until those numeric budgets are accepted, implementation readiness is blocked. Duplicate names
are harmless because names never locate a frame. Unknown or mismatched tuple members fail closed.

## Invalidation

Navigation (including document replacement), frame rebuild/detach, or origin changes revoke the
affected frame and descendant references and snapshots. Tab switch invalidates interaction refs
for the previous active context; tab close invalidates the closed tab. Session/browser replacement
invalidates all refs. Snapshot replacement revokes previous elements in that frame. Returning to
a prior tab does not revive old refs. No frame-name matching, selector search or similar-element
relocation is permitted. Same-document changes may conservatively invalidate; never silently
rebind an old token. Lifecycle stream loss/overflow marks context unavailable until explicitly
re-established and newly discovered.

Stable bounded failures: InvalidReference, StaleFrameReference, DetachedFrame, StaleElementReference,
ContextUnavailable, OriginNotAuthorized, UnsupportedVersion, ResourceLimit, IndeterminateExecution.
Unknown/tombstone-evicted refs may return InvalidReference; preserving exact historical diagnostics
must not require an unbounded tombstone store. No raw transport errors appear in these results.

## Trusted Admission And Permission Resource

Propose a browser-specific resource with exact normalized top-level origin, exact target-frame
origin, operation class and host-bound session/tab/frame generation scope. Origins use parsed
scheme/host/effective port, never paths, credentials, queries or fragments. For nested frames,
trusted ancestry identity is part of context binding; ancestry is not authority. Every additional
frame actually observed must be authorized before its content is disclosed.

The resource is derived from a trusted lifecycle registry associated with the bound executor;
it is not derived from model fields or from page text. Parent approval never grants cross-origin
child access. A same-origin child still requires valid explicit frame identity and operation scope.
Opaque/sandboxed origins fail closed for interaction in the initial contract. Inherited origins
such as about:blank/srcdoc require trustworthy effective-origin evidence; otherwise deny. No
wildcard or textual host-suffix matching. Business-write policy remains host owned.

Frame-tree discovery needs approval for the bound tab's structural enumeration. It may disclose
only the approved bounded origin/structure inventory, not child content. A subsequent child read
or action requires that child's resource; enumeration approval cannot authorize interaction.

Ordering:

1. Exact closed schema, typed parse and semantic checks.
2. Resolve refs from trusted bounded lifecycle state; reject stale/mismatched/unready contexts.
3. Produce an immutable invocation-bound admission ticket carrying context epochs and origins.
4. Evaluate the browser permission resource from that ticket before prompts or reusable grants.
5. At authorized dispatch, validate the same ticket and refs again. Context changes invalidate
   the pending approval; do not retarget or reuse it for another origin.
6. Delegate exactly once only if authorization and epochs still match.
7. Executor checks the bound document/frame epochs immediately before mutation; project typed output.

Direct execute must not turn step 5 into an authorization bypass: require the same admitted,
authorized capability or return a bounded denial. A generic Network facet alone is insufficient.

Repository gap: AgentTool execution_admission and permission_profile are synchronous and do not
currently carry this ticket. API review must choose an invocation-owned context integration with
concurrency-safe authorization binding. A tool-global “last request” cache is rejected. A stale
cache must not be refreshed by making a browser execution call before permission; fail closed and
require explicit discovery instead. Browser lifecycle synchronization is trusted host plumbing,
not a model-visible tool or authority grant.

## Lifecycle Race And Trust Boundary

Refs stale before admission or final dispatch must produce zero BrowserExecutor execution calls.
A frame may still navigate after dispatch: the executor must reject before mutation when the
bound document is no longer current. Test this separately; do not falsely promise zero delegation
for a change occurring after delegation. Serialize per-session operations and bind mutation to the
resolved document identity, never to whatever frame currently occupies a slot. If a backend cannot
establish this property, it is not conformant. Browser crashes or ambiguous completion return
IndeterminateExecution with no automatic replay, recreation or fallback.

Host and process executors are trusted implementations of this boundary, not hostile code confined
by a Rust trait. Plugin trust/isolation remains the process-carrier story's responsibility. A
conformance suite proves tested behavior; it cannot enforce honesty by a malicious executor.

## Projection And Composition

Keep fill/select values, credentials, cookies/storage, headers, raw DOM, selectors/scripts,
profiles, raw driver IDs and diagnostics out of ordinary observer projections. Permission
presentation may show bounded sanitized top/child origins, operation and opaque references;
model/display/persistence views remain separately bounded. Browser-page ingestion and disclosure
continuations cannot mint browser authorization or forward arbitrary frame commands. Default
registries remain unchanged; host and process adapters share the same Talos-owned schema,
admission, permission and projection rules.

## Mandatory Acceptance Matrix

For every rejection fixture assert the bounded error, approval/grant count, executor call count,
mutation count, and model/display/persistence output. Count executor calls independently of
mutations. Test both normal registry dispatch and direct/authorized entry points.

| Fixture | Required result |
|---|---|
| Same-origin iframe | Explicit frame/snapshot identity; admitted authorized action delegates once |
| Cross-origin iframe | Parent grant alone denies child read/action; exact child authorization required |
| Nested iframe | Correct ancestry binding; no ancestor grant inheritance |
| Delayed loading | Pending reference rejects; fresh authorized discovery required |
| Duplicate names / ambiguous frames | Opaque refs distinguish; names never select or relocate |
| Rebuilt/replaced frame | Old frame and all its element refs rejected before dispatch |
| Detached frame | Detached or invalid ref rejects, zero execution/mutation |
| Navigation | Old frame refs revoked; redirects/origin changes invalidate pending approval |
| Snapshot/frame rebuild | Old or foreign snapshot elements rejected, including cross-tab replay |
| Tab switch / close | Invalidated interaction refs never revive on returning to the tab |
| Browser/session replacement | All former-generation refs reject, including token replay |
| Approval-time race | Navigation while waiting for approval gives zero execution calls and no reusable stale authority |
| Post-dispatch race | Executor rejects stale document before mutation; no retry |
| Concurrent calls | Admission tickets cannot swap origins, refs or authorization |
| Lost lifecycle / opaque origin | Fail closed without execution or inherited authority |
| Schema/version abuse | Unknown fields, missing scope, V1/v2 mismatch reject before approval |
| Boundaries | Depth/node/byte limits, oversized tokens/results/errors are bounded/redacted |
| Transport ambiguity | One attempt, IndeterminateExecution, zero replay/fallback |
| Projection secrecy | Seed secrets in URLs/input/results/errors; assert absence in forbidden views |
| Optional composition | Default inventory unchanged; host executor works without native/plugin dependencies |

Before acceptance, reviewers must resolve the exhaustive v2 schema, numeric snapshot/result budgets,
permission type/ticket API and backend atomicity evidence. Then select the runnable host conformance
iteration and effective claim. Native iframe fixtures require real-browser evidence in the native
implementation story; fake-executor tests alone cannot satisfy downstream delivery for #520.
