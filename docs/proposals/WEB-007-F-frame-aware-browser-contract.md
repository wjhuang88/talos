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

Every request has `protocolVersion: 2` and one `operation` discriminator. The following 21
branches are closed objects: only common fields plus the listed required/optional fields are
accepted, with `additionalProperties: false` at every object. No implicit current tab/frame.

| Operation | Required fields (besides common fields) | Optional fields | Resource class |
|---|---|---|---|
| open | tabRef, url | visibility | Navigate |
| read | tabRef, frameRef | none | Observe |
| snapshot | tabRef, frameRef | none | Observe |
| current-url | tabRef, frameRef | none | Observe |
| screenshot | tabRef, frameRef | none | Observe |
| click | tabRef, frameRef, snapshotRef, elementRef | none | Interact |
| fill | tabRef, frameRef, snapshotRef, elementRef, text | none | Interact |
| select | tabRef, frameRef, snapshotRef, elementRef, option | none | Interact |
| hover | tabRef, frameRef, snapshotRef, elementRef | none | Interact |
| check | tabRef, frameRef, snapshotRef, elementRef | none | Interact |
| uncheck | tabRef, frameRef, snapshotRef, elementRef | none | Interact |
| press | tabRef, frameRef, key | none | Interact |
| scroll | tabRef, frameRef, direction | amount | Interact |
| wait-for-element | tabRef, frameRef, snapshotRef, elementRef | none | Observe |
| wait-milliseconds | milliseconds | none | SessionWait |
| tab-new | none | none | SessionCreate |
| tab-list | none | none | SessionInventory |
| tab-close | tabRef | none | TabLifecycle |
| tab-switch | tabRef | none | TabLifecycle |
| window-close | none | none | SessionClose |
| frame-tree | tabRef | none | FrameInventory |

`press.key` is required and its enum is Enter, Tab, Escape, Space, Backspace,
Delete, ArrowUp, ArrowDown, ArrowLeft, ArrowRight, Home, End, PageUp, PageDown. No chords or text.
`direction` is up/down/left/right; `amount` is an integer 1..5000 (default 500 CSS pixels).
`milliseconds` is integer 1..30000. Visibility is background/foreground (default background).
`url` is absolute HTTP(S), host required, credentials/NUL forbidden, at most 4096 UTF-8 bytes.
Fill text is at most 4096 bytes (empty allowed); option is 1..1024 bytes; both reject NUL.
All references are 1..128 ASCII bytes matching `[A-Za-z0-9_-]+`; prefix/shape never grants scope.
Reject nulls, fractional integers, duplicate JSON keys and any unlisted field. Schema string
length checks are supplemented with UTF-8 byte admission checks. Entire request limit: 16 KiB.
Ingress JSON parsers must reject duplicate keys before conversion to serde_json::Value; a tool
cannot recover discarded duplicate keys. Generated schema and typed semantic parser must reject
the same invalid fixture corpus except explicit byte-limit checks enforced by semantic admission.
The v2 admission boundary owns this full closed-schema check; current ToolRegistry validation is
not sufficient. Provider adapters must preserve and validate raw tool-argument JSON before
emitting a v2 tool call (including streamed OpenAI and Anthropic native/compat paths). Any other
entry point that already receives a parsed Value, including the current standalone MCP handler,
must keep v2 unregistered or return UnsupportedVersion before permission evaluation until its
transport can prove duplicate-key rejection at raw ingress. Re-serializing Value is not proof.
Non-v2 tools retain their existing parsing and registry validation behavior.

`open` navigates only the named tab's top document; it is never a child-frame navigation escape.
`press` targets the resolved document, never browser-global keyboard focus. An executor unable to
isolate this target returns UnsupportedOperation. Wait-for-element observes the existing element
for at most 5 seconds; it does not relocate or rebuild it. All calls have a 35-second maximum
execution deadline; approval wait has a separate ticket lifetime below. Timeout never triggers retry.

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

Snapshots are scoped to exactly one tab/frame and bounded by the v2 output budgets below.
Duplicate names are harmless because names never locate a frame. Unknown or mismatched tuple members fail closed.

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
ContextUnavailable, OriginNotAuthorized, UnsupportedVersion, UnsupportedOperation, ResourceLimit,
AdmissionExpired, InvalidRequest, PermissionDenied, IndeterminateExecution.
Unknown/tombstone-evicted refs may return InvalidReference; preserving exact historical diagnostics
must not require an unbounded tombstone store. No raw transport errors appear in these results.

## Trusted Admission And Permission Resource

Use a typed `BrowserPermissionResource` tagged union, not Domain strings or PathBuf. Every
variant binds host-created executor identity and browser/session generation; variants are:

| Variant | Additional exact scope | What it permits |
|---|---|---|
| SessionInventory | session epoch | Bounded tab identifiers and approved origin metadata only |
| SessionCreate | session epoch | Create one empty tab; no URL or content access |
| SessionWait | session epoch, milliseconds | Bounded delay only |
| SessionClose | session epoch | Close the bound browser window |
| TabLifecycle | tab identity/epoch, close or switch | Only the named lifecycle operation |
| FrameInventory | tab identity/epoch, top-origin identity | Bounded frame structure/origin metadata only |
| Navigate | tab identity/epoch, current top-origin identity, exact destination origin | One explicit top-level navigation |
| Observe | tab/frame epochs, exact top and frame origins, exact operation | One frame-local observation |
| Interact | Observe scope plus snapshot/element when applicable | One exact interaction |

Origin identity is `HttpOrigin(scheme, ASCII canonical host, effective port)` or a host-verified
`OpaqueDocument(documentEpoch)` for lifecycle/inventory only. Opaque identity is not serialized
as a reusable public origin. Observation/interaction requires HttpOrigin; sandboxed opaque frames
fail closed. Inherited about:blank/srcdoc origins require trustworthy effective-origin evidence.
Unknown/unavailable is not an origin and never matches. Origins contain no path/query/fragment,
userinfo or page text; forbid wildcards, suffix matching and lossy port normalization.

The host binds executor/session identity and maintains a trusted bounded lifecycle registry before
tool composition. Session inventory/create/wait/close therefore do not require a previously issued
tab/frame token. Tab-list yields opaque tabRef; frame-tree yields frameRef, including Pending refs
that cannot be observed or acted upon until fresh authorized discovery. A newly created empty tab
can be navigated using OpaqueDocument as its current origin identity. Host lifecycle subscriptions
update state without executing model operations or minting grants. On lost synchronization, deny
context-dependent operations with ContextUnavailable; trusted host resynchronization or explicit
session replacement is required before discovery can resume. Do not promise that a model call can
repair unavailable trusted state by bypassing permission.

Inventory authorization explicitly covers bounded tab/frame origin and structure disclosure only;
URLs, titles, names and content are absent. Unapproved inventory reveals no child origin metadata.
Parent grants never cover a cross-origin child read/action. Same-origin children still require
explicit valid frame scope. Ancestry identity is bound for context integrity, never authority.
All browser grants in this initial v2 design are invocation-only, with exact operation and request
binding. Existing Network allow rules and session grants do not imply a browser grant; host policy
may evaluate each new invocation but cannot skip browser admission. Business-write policy remains
host owned. Denials are final for that invocation.
The browser evaluator defaults to deny and accepts only an explicit browser-aware policy decision
on the exact typed resource or a per-invocation human approval. Legacy tool-name, ToolNature,
Network, path, glob, session and workspace-trust Allow rules cannot satisfy this resource, even
when they match the browser tool by name or would allow its underlying transport. A browser-aware
policy may authorize only one admitted invocation and cannot create a reusable grant. If a
composition root cannot route to this evaluator, it must not register or execute v2.

Navigate authorizes the requested destination origin and current tab context. Automatic cross-origin
redirects must be blocked before the unapproved request is sent, returning OriginNotAuthorized;
no automatic follow, reauthorization or replay. Same-origin redirects stay within the approved
origin and execution deadline. If a backend cannot enforce this, Navigate is UnsupportedOperation.
Interactive page actions may cause site behavior; this contract does not authorize unrelated
browser navigation. The executor must stop origin-changing navigation before network dispatch or
declare the affected operation unsupported. Already performed effects are never undone or replayed.

Ordering:

1. Exact closed schema, typed parse and semantic checks.
2. Resolve only the refs required by the selected branch from trusted lifecycle state; reject
   stale/mismatched contexts and unready observation/interaction targets. Inventory may return
   Pending frames; session operations do not require undiscovered frame refs.
3. Produce an immutable invocation-bound admission ticket carrying context epochs and origins.
4. Evaluate the browser permission resource from that ticket before prompts or reusable grants.
5. At authorized dispatch, validate the same ticket and refs again. Context changes invalidate
   the pending approval; do not retarget or reuse it for another origin.
6. Delegate exactly once only if authorization and epochs still match.
7. Executor checks the bound document/frame epochs immediately before mutation; project typed output.

Direct execute must not turn step 5 into an authorization bypass: require the same admitted,
authorized capability or return a bounded denial. A generic Network facet alone is insufficient.

### Prepared Invocation API And Migration

Selected design: add an opt-in prepared invocation path alongside existing AgentTool methods.
These are proposed API names/semantics, not claims that the API exists:

- `prepare_invocation(input)` returns an owned, non-Clone `PreparedToolInvocation`, or a bounded
  error. It owns the validated canonical request, invocation nonce, executor identity, lifecycle
  epochs, derived typed browser resource, creation time and 120-second monotonic expiry.
- The permission-aware composition root evaluates only that object's typed resource. It issues a
  `BrowserInvocationAuthorization` bound to the invocation nonce, entire canonical request digest,
  resource and expiry. Resource equality alone cannot authorize a different fill value or operation.
- `execute_prepared(prepared, authorization)` consumes both exactly once. Talos validates matching
  request/context/expiry and epochs before consuming into an executor call; denial, cancellation,
  timeout or mismatched capability destroys the ticket. A reused nonce is rejected even if a
  transport message was duplicated. The bounded nonce store fails closed on capacity exhaustion.
- Constructors/access to authorization are restricted to trusted permission composition; no model
  or plugin may deserialize or mint one. Plugin transport carries a host-generated invocation ID,
  the admitted request and expected epochs, never a permission object to reinterpret.
- ManagedBrowserTool refuses legacy `execute`, `execute_authorized` and output variants with a
  bounded PermissionDenied even if supplied existing path authorizations. A missing prepared-path
  implementation fails closed; other tools retain legacy behavior without forced migration.

Current code facts: permission_profile is computed before execution_admission in
`crates/talos-agent/src/tool_execution.rs`; that admission result selects foreground/background,
not an invocation ticket. `ToolExecutionAuthorization` in
`crates/talos-core/src/tool/authorization.rs` is path-bound, and Domain matching is not exact-origin
browser authorization. Do not insert browser resources into either representation. The new
prepared path must run exact validation/admission BEFORE deriving/evaluating the permission
resource; legacy hook order stays untouched for non-browser tools.

The implementation slice must add explicit support in talos-agent dispatch/permission pipeline,
talos-runtime's permission-aware wrappers, and talos-mcp's server permission/handler path, with
wrapper forwarding tests. Any composition root not migrated must reject browser registration or
execution. Public defaulted AgentTool hooks preserve existing implementers; new resource and
capability types are separate from existing exhaustive enums/structs. Record an ADR and migration
plan before code/API publication; no existing path-authority contract is widened by this proposal.
For MCP, migration includes a raw-argument integrity route before rmcp converts JSON to an object;
without it, v2 remains unavailable on standalone MCP rather than claiming schema parity.

## Lifecycle Race And Trust Boundary

Refs stale before admission or final dispatch must produce zero BrowserExecutor execution calls.
A frame may still navigate after dispatch: the executor must reject before mutation when the
bound document is no longer current. Test this separately; do not falsely promise zero delegation
for a change occurring after delegation. Serialize per-session operations and bind mutation to the
resolved document identity, never to whatever frame currently occupies a slot. If a backend cannot
establish this property, it is not conformant. Browser crashes or ambiguous completion return
IndeterminateExecution with no automatic replay, recreation or fallback.

A backend readiness check declares supported operation/document-binding pairs before approval;
unsupported pairs return UnsupportedOperation with zero mutation. Per-session serialization alone
is insufficient because pages navigate autonomously. All interactions (including press/scroll)
must use a document-bound primitive that fails if the document ceases to be current. Coordinate
clicks or browser-global keyboard/scroll dispatch based only on a preceding epoch check are
nonconformant. Reads/snapshots/screenshots must likewise be captured from the bound document and
validate its continued identity before releasing output; a mismatch discards the entire result.
Tests insert deterministic barriers between final epoch check, action/capture, and result release.
No native implementation is claimed to meet these requirements until its implementation story
supplies real-browser evidence; unsupported operations remain unavailable without fallback.

Host and process executors are trusted implementations of this boundary, not hostile code confined
by a Rust trait. Plugin trust/isolation remains the process-carrier story's responsibility. A
conformance suite proves tested behavior; it cannot enforce honesty by a malicious executor.

## Typed Outputs, Budgets And Projection

All output objects are closed tagged variants, validated before model/display/persistence projection:

| Variant | Allowed payload | Numeric bounds |
|---|---|---|
| Ack | operation, status=ok | 1 KiB total |
| TabCreated | tabRef | 1 KiB total |
| Tabs | entries(tabRef, origin-or-opaque-state), truncated | 64 entries, 16 KiB total |
| Frames | entries(frameRef, parentFrameRef-or-null, origin-or-opaque-state, readiness), truncated | 128 nodes, depth 16, 64 KiB total |
| PageUrl | sanitized origin and path | 4096 bytes total; no query/fragment/userinfo |
| Read | plain frame-local text, truncated | 32 KiB text, 40 KiB total |
| Snapshot | snapshotRef, nodes(elementRef, parentElementRef-or-null, role, name, states), truncated | 512 nodes, depth 32, name 256 bytes, 64 KiB total |
| Screenshot | transient artifactRef, mime, width, height, byteLength | PNG only, 2 MiB encoded, 2048 per dimension, 4 megapixels |
| Failure | code from fixed enum, operation-or-null, outcome=notExecuted/unknown/partial | 1 KiB, no free-form driver message |

Failure operation is null only when no valid request discriminator was admitted. notExecuted
requires proof that no action ran; partial denotes known effects before a blocked redirect or later
failure; unknown covers ambiguous completion. A post-action failure must never claim notExecuted.
Frames readiness is Pending or Ready; opaque-state is the literal opaque and contains no document
identity. Output discriminator is `kind` with exactly the variant names above; no extra fields.

Ack covers action/lifecycle/wait success; TabCreated covers tab-new; all observation/discovery
operations map only to their named variant. Snapshot node fields are exactly elementRef,
parentElementRef-or-null, role, name and states.
Roles are generic, button, checkbox, radio, textbox, combobox, option, link, heading, list, listitem,
table, row, cell, image and statictext. States are a unique array drawn from checked, unchecked,
mixed, selected, expanded, collapsed, disabled, readonly and required (at most nine). Role names
never contain raw DOM attributes; unknown roles reduce to generic and unknown states are omitted.
No unvalidated arbitrary JSON payloads. Per-origin
strings are at most 2048 bytes; over-budget results reject or truncate only at validated boundaries.
Truncation never leaves dangling references or dangling snapshot tree links. V2's normal transport
JSON envelope is at most 96 KiB; screenshot bytes travel through a separate bounded transient
artifact channel. No base64/data URL or raw filesystem path is embedded in tool text.

Registry budgets per bound session: 64 tabs, 128 live frames per tab, one current snapshot per
frame, 512 elements per snapshot, 256 outstanding tickets/nonces. Tickets expire after 120 seconds;
completed nonces remain only until original expiry, after which every replay is expired anyway.
Opaque references are never reused; eviction invalidates references and returns InvalidReference.
Capacity exhaustion returns ResourceLimit before approval/execution, never silently drops a live
authorization. Host session destruction clears registry and changes generation.

Frame-local text/snapshot traversal MUST stop at embedded browsing-context boundaries, including
same-origin children. No descendant text, form values or accessibility subtree may be included.
Screenshot capture must exclude/mask every descendant frame's pixels before creating the transient
artifact; if exact containment cannot be proven, return UnsupportedOperation. Tests seed child
secrets and inspect the image as well as text output. Choosing a parent frame does not approve
visual access to its children. Password/OTP fields are omitted; sensitive classification/redaction
is required before any model projection. Page data is untrusted content, never tool instructions.

Fill/select values, credentials, cookies/storage, headers, raw DOM, selectors/scripts, profiles,
raw driver IDs and diagnostics are never projected. Approval display may include sanitized origins,
operation and opaque references, at most 4 KiB. Model view uses the typed budgets above; display
and persistence each retain at most 4 KiB summary with no page text, snapshot names, image bytes,
input values or raw URLs. Screenshot artifact capabilities expire within 120 seconds, are bound
to the requesting invocation/session and are never durable or executable filesystem handles.

Browser-page ingestion and disclosure continuations cannot mint browser authority or forward
frame commands. Default registries remain unchanged; host and process adapters share these
Talos-owned admission, permission and projection rules. A host executor lacking an optional
operation must return UnsupportedOperation; it may not substitute a broader operation.

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
| Bootstrap / empty tab | Session inventory/create works without frame refs; unknown trusted state never grants authority |
| Redirect / focus escape | Unapproved origin requests blocked before dispatch; frame-local key actions never use global focus |
| Child observation isolation | Text/AX traversal excludes descendants; screenshots mask child secrets or fail closed |
| Ticket replay / payload swap | Expired, reused, foreign-executor or changed fill-value tickets reject before executor call |
| Canceled approval / capacity | Cancel destroys ticket; 257th outstanding ticket fails closed without eviction of live authority |
| Origin normalization | Scheme/port differences stay distinct; Unicode host canonicalization is shared; suffix/wildcard grants reject |
| Document-action barrier | Navigation between check and press/scroll/capture rejects without wrong-document mutation or output |
| Optional composition | Default inventory unchanged; host executor works without native/plugin dependencies |

Independent review must evaluate the above selected v2 schema, budgets, prepared API and
fail-closed backend requirements. Actual backend atomicity, generated-schema parity and runtime
behavior require implementation evidence before delivery; design approval does not supply it.
Then select the runnable host conformance iteration and effective claim. Native iframe fixtures
require real-browser evidence in the native
implementation story; fake-executor tests alone cannot satisfy downstream delivery for #520.
