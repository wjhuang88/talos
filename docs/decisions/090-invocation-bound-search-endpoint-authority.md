# ADR-090: Invocation-Bound Search Endpoint Authority

*Status: Accepted contract / #712 activation proposed. This draft status becomes effective only on target-branch merge after independent exact-head security/API review and CI; no implementation authority before that merge.*

## Context And Authorization

The maintainer accepted the bounded production scope on 2026-10-13: invocation-bound search
planning, concrete endpoint authority, Agent/tool wiring, guarded transport and actual-dispatch
validation. This accepts scope, not an unreviewed public API. ADR-085 continues to govern routing,
private SearchBackend vocabulary, zero-key admission, explicit paid consent and compatibility.
I293 owns this prerequisite to its existing C outcome; I300 Git and PERM-007-H all-built-in Auto
coverage remain separate. No production implementation starts before the scope amendment is merged.

#708/#709 establish two gaps: the registered tool's untyped Network facet cannot compile default
Ask proposals; the current execution carrier is Path-only. Fresh audit on main `9cad344b` confirms
that Session grants match tool/provenance/facets only, not request fingerprint. Merely adding
manifest data to that fingerprint would still allow same-host cross-port/path grant reuse.

## Decision

### 1. Add An Opt-In Owned Prepared Network Path

Add defaulted methods on AgentTool, preserving existing implementers:

```rust,ignore
fn requires_network_invocation(&self) -> bool; // default false
fn prepare_network_invocation(&self, input: &Value)
    -> Result<Option<Box<dyn PreparedNetworkToolInvocation>>, ToolNetworkPreparationError>; // default Ok(None)
```

The new core trait is Send/object-safe and owns validated input, configuration and execution:

```rust,ignore
trait PreparedNetworkToolInvocation: Send {
    fn manifest(&self) -> &ToolNetworkManifest;
    fn permission_profile(&self) -> &[ToolPermissionFacet];
    async fn execute(self: Box<Self>, authority: Arc<dyn ToolNetworkAuthority>)
        -> ToolExecutionOutput;
}
```

Use async_trait as for existing core traits. Preparation performs local validation only, with
an independent closed preparation error type mapped to fixed, content-free error codes. Never
forward query/configuration URLs, proxy userinfo, headers or upstream diagnostics through errors
to the model or logs; redacted Debug alone does not protect returned error strings. Existing
hooks retain structured redacted input; the manifest has a separate safe approval preview and
does not rewrite model JSON. Agent prepares once after existing input/schema/presentation
checks, retains existing hooks/ledger gates and executes that same object. A required prepared
path with missing preparation, missing permission pipeline, failed/stale authority or unsupported
wrapper must fail closed; never fall back to ordinary execute. WebSearchTool's direct legacy
execute/execute_authorized paths must not bypass the protected path after activation.
The browser raw-argument/prepared path remains unchanged and cannot carry this authority.
Any existing normalization/hook that changes the execution input must finish before preparation,
or invalidate the prepared request before dispatch. Never approve one input and execute a
post-hook replacement through the old plan, or silently reprepare without fresh admission.

### 2. Introduce Separate Network Types, Not Fake Path Capabilities

New core types have private fields, validated constructors, immutable accessors, redacted Debug
and no serialization of authority. Do not change existing exhaustive enums or public struct
literals. A bounded ToolNetworkManifest contains:

- a fresh per-invocation ID, independent of query text;
- a stable tool-instance/configuration identity and revision;
- bounded actual destinations: scheme, canonical host, effective port, HTTP method and exact
  encoded request path; query values and credentials are not destination authority;
- the explicit connection/proxy/redirect policy needed to describe those requests truthfully.

Transport derives identity from the actual parsed request URL/method, not a claimed label.
Canonicalization and policy projection must use the same validated representation, preserving
existing resource/glob Deny semantics: matching Deny dominates, and first-match behavior is
retained within non-Deny rules. The initial #712 wording incorrectly summarized this as only
per-facet first-match; implementation source audit confirms the existing evaluator already scans
for terminal matching Deny. This is a factual correction, not a new policy scan or behavior change.
Domain facets cover every actual probe, request, fallback, CDN and
redirect host before approval; manifest binding is
separate from those rule-matching facets, not a fabricated Remote facet or model JSON field.
No wildcard authority or global relaxation of MissingResource is permitted. Generic untyped
Network requests retain their existing failure behavior.

Initial bounds are implementation-review gates: finite manifest, query/body/output/redirect
limits and one enclosing execution budget. They must be recorded and tested before activation;
this ADR does not treat unmeasured numeric values as accepted limits.

### 3. Bind The Entire Permission Lifecycle

Add a typed manifest binding through PermissionRequest's private state and an additive builder
or constructor. Existing constructors use None and preserve ordinary-tool behavior. Preserve
PermissionAuthorizationRequest's public fields; use a new authorize_network entry point with
separate typed manifest argument rather than adding a required public field.

The canonical destination/configuration scope participates in proposal compilation, preview,
Session grant creation, grant_matches, pending admission and network authority minting. None
does not match Some: an old host-only grant cannot upgrade itself into network-manifest authority.
Session approval may cover a different query only when the complete destination/configuration
scope remains equal; each invocation still receives a fresh request/plan-bound execution authority.
Once authority remains invocation-local. Current Deny, provenance, context and state generations
remain authoritative; a partial approval cannot remove a denied facet to execute the rest.

Keep existing admit -> Vec<ToolExecutionAuthorization> for Path callers. Add network admission
on Arc<PermissionSessionState>, reusing the same internal fence. Manifest completeness and
facet coverage are validated before approval and again at admission. Approval preview must show
real origins, ports, methods, paths, fallback and proxy exposure; a host-only preview is insufficient.
Extend private preview state/additive accessors and update official renderers without breaking
existing public struct literals or exporting keys, signed URLs or query digests.

### 4. Inject A Live Checker From Permission State

Core defines an object-safe, non-serializable ToolNetworkAuthority checker; it does not depend
on talos-permission. Permission implements it using Arc state and the admitted tool/provenance,
request/plan ID, complete scope, context, configuration binding and authority source. Agent injects
it into the prepared object. Prepared objects never authorize themselves.

```rust,ignore
trait ToolNetworkAuthority: Send + Sync {
    fn check(&self, invocation_id: Uuid, destination: &ToolNetworkEndpoint)
        -> Result<(), ToolNetworkAuthorizationError>;
}
```

The endpoint/manifest/error types are new, privately constructed validated data; they do not
change ToolExecutionAuthorization's Path fields or constructors. Additive permission entry
points are PermissionRequest::with_network_manifest, PermissionSessionState::admit_network and
PermissionPipeline::authorize_network. Exact API definitions and external-implementer compile
fixtures must conform to this inventory before code approval.

Every initial/redirect request must pass this checker before logical request start. It rechecks
current Deny, relevant policy/mode/workspace/registration/restriction generations and continued
existence of its grant. Creating an unrelated Session grant must not itself invalidate an already
admitted request; revocation of its actual grant must. No Session mutex is held across network await.
The logical-start fence is the linearization point. Revocation after that point cannot unsend
bytes or guarantee zero races between checking and socket I/O. Future request starts are denied;
deadline/drop/cancellation and already-started exposure require separate truthful receipts.

Configuration is immutable per tool instance in the initial implementation. Replacing a tool
or credentials creates a new configuration identity and invalidates registration authority at
the official root. Future mutable configuration must invalidate old plans before it is supported.
No shared last-plan cache, cross-Session reuse or raw credentials in binding/telemetry.

This constrains model/MCP inputs and official execution paths. It is not an OS sandbox against
arbitrary malicious same-process native Rust code. MCP JSON cannot supply a trusted manifest or
native provenance. Unsupported standalone/MCP paths reject or use an explicitly trusted host
pipeline; they cannot silently obtain broad authority.

### 5. Own Transport Before Switching Default Search

Disable implicit redirects in controlled clients. Follow only explicitly scoped next hops, with
actual method/body handling, no credential forwarding to another origin and bounded total work.
DNS/resolved-address and proxy handling require connection-level fixtures; URL/origin checking
alone is not SSRF proof. Snapshot proxy/configuration before approval, preserve supported explicit
self-hosted/proxy behavior and track unsupported configurations instead of silently dropping them.

The verified rust-websearch 0.1.2 package has no injected-client/parser-only public seam:
SearchConfig has no client field, engine functions build clients internally, those clients follow
up to five redirects, and HTML parser functions are private. An outer checker cannot constrain
these sends. Do not call that opaque coordinator from the newly protected default path.

The bounded compatibility migration is to rehost the already-used request/parser protocol inside
talos-tools' private compatibility adapter, under Talos-owned guarded clients, retaining upstream
MIT provenance/license and reusable network-free helpers/types. No new crate, new provider,
third-party dependency or forked package is selected. This remains one compatibility family,
not D/E/F native admission or two independently qualified routes. The I293 scope amendment
explicitly covers this protocol internalization. Compatibility naming, an unchanged dependency
and the package's MIT declaration do not waive D's terms/privacy/parser gates. Coordinate a
bounded D review of migration differences and unresolved existing protocol concerns before
activation; no new native admission is implied. Before copying source, obtain the upstream
license/notice at the pinned source revision and preserve per-file provenance and modification
records. Qualified default activation requires request/parser/probe/retry/source parity, query
exposure review and connection-enforcement fixtures.

Lock/dependency identity remains. Rollback means a controlled version/deployment revert or a
separately accepted migration, coordinating ADR-085's rollback clause. A protected invocation
must never fall back to the opaque coordinator or legacy execute on missing/stale authority,
redirect denial or parity failure.
If parity or connection enforcement cannot be established, do not switch the default or silently
disable working search. Record a concrete migration amendment before relaxing this gate.

Paid availability is not selection. A key alone never enters the protected auto candidate set.
Explicit trusted-host selection can enable an existing optional adapter; H continues to own the
complete public configuration/fallback-to-auto policy. Wikipedia, if needed for compatibility,
is declared before approval and remains a knowledge fallback, not a native generic route.

## Migration And Acceptance

### Ordinary Invocation Coexistence — 2026-10-27

The maintainer, also the responsible actor recorded for I293 and I308, explicitly confirmed
this boundary in the handoff closure session. This supplies the owner concurrence requested by
[I308's accepted proposal](../proposals/i308-native-invocation-authorization.md); the amendment
becomes effective on target-branch merge after independent security/API review and exact-head CI.

- Ordinary implementation-owned HTTP may use I308's whole-invocation review, including retained
  proxy/redirect configuration and transitive effects. It does not promise per-endpoint isolation
  or inherit this ADR's protected endpoint authority. Positive trusted registration and an explicit
  ordinary disposition are required; absence of prepared evidence is insufficient.
- WebSearch, search compatibility/default transport and every tool requiring prepared-network
  admission remain under I293 and this ADR. Naming a compatibility adapter ordinary does not
  change ownership. Missing, failed, stale or unsupported preparation/authority must reject;
  neither ordinary HTTP nor Shell/Exec review is an internal fallback for that protected call.
- A separately requested ordinary Shell/Exec/HTTP operation retains its accepted invocation
  semantics, including possible network effects; this amendment does not claim to confine arbitrary
  commands or prevent users from independently requesting network access. Explicit Deny and
  mandatory-human policies, including incompatible finer restrictions, remain authoritative.
- This decision does not activate ordinary Auto, alter search routing, waive transport/parity/D
  gates, or deliver any runtime behavior. I308 must prove trusted eligibility, actual dispatch,
  protected non-downgrade and the rest of its acceptance before delivery. I293 retains its existing
  scope and acceptance; no search implementation is transferred to I308.

- Preserve web_search name, model schema/formatter contract, Network family/contribution and
  private SearchBackend vocabulary. No model-selected provider, GeoIP or startup probe.
- Add only defaulted trait methods/new types/additive permission entry points. Do not add fields
  to public literal-constructible request structs, extend existing exhaustive enums, repurpose
  PathBuf as endpoint storage or generalize browser errors. API inventory/adapter migration and
  external-implementer compile tests are required; no release is authorized.
- Official CLI/SDK/MCP/standalone paths must have tested dispositions. Old wrappers that do not
  forward the prepared path fail closed. No approval TTL substitution for execution budget.
- Tests must cover same-host port/method/path changes, query reuse versus scope changes, legacy
  grants, Deny/canonicalization, concurrent invocations, registration/config changes, revoke versus
  unrelated grant creation, redirects/leakage, real dispatch and actual controlled transport.
- Full C/EP acceptance, native admission/global coverage and dependency retirement stay Pending
  until their own evidence passes. Independent exact-head security/API review, locked full checks,
  owner synchronization, compatibility rollback and final CAS precede production activation.

## Evidence And Alternatives

Pinned dependency archive checksum: `06b90d9a9ad9e7e347c4cd85c3fb37189256703db4fab1d6c6fb963ae1eed0ee`;
freshly downloaded and verified, no live search performed. Source: types.rs, anti_detection/mod.rs,
duckduckgo/lite.rs/html.rs, searcher.rs, Cargo.toml (MIT). The package's VCS metadata pins
`b30180a03609580e8b28dfb682284cb2ba249fba`, `crates/websearch`. The upstream root
[LICENSE](https://github.com/Rust-Framework/rust-agent-framework/blob/b30180a03609580e8b28dfb682284cb2ba249fba/LICENSE)
was retrieved at that revision: MIT, Copyright (c) 2026 Rust-Framework Contributors, blob
`5760f660f796a3a8bacec0af53e4c51ef5caeada`. Retain its full notice with any substantial copied
source; this evidence is not provider terms/privacy admission. Main audit and independent design preview
are distinct from unimplemented transport proof.

Rejected: Domain facet alone; only fingerprint binding; fake Remote scope/model control fields;
prepared self-authorization; browser route reuse; ambient paid selection; opaque client bypass;
new talos-search crate or dependency fork. None resolves the complete boundary within this scope.

Related: ADR-066/067/085; [I293](../iterations/I293-search-router.md);
[endpoint authority plan](../reference/SEARCH-001-C-ENDPOINT-AUTHORITY-PLAN.md).

## Acceptance Record

The maintainer's 2026-10-13 acceptance authorizes the bounded scope described above. Independent
security/API design preview found no remaining design blocker after the safe-error, D migration
and rollback corrections. That preview is not exact-head PR approval or transport proof. #712
must record exact-head independent protected approval, CI and final CAS before merge; only then
is this contract Accepted and the amended I293 scope effective. Runtime acceptance remains gated
by the Migration And Acceptance section; no production implementation is delivered by #712.
