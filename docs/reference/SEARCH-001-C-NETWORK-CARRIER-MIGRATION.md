# SEARCH-001-C: Invocation-Bound Network Carrier Migration

Carrier stage under effective I293 amendment #712 (989fec2e). This adds permission/dispatch
infrastructure; WebSearchTool, model schema/output, transports, selection, dependency and lockfile
remain unchanged. C-V01..11 and EP-01..10 remain Pending; D/compatibility activation gates remain.

## API Inventory

| Surface | Addition / disposition |
|---|---|
| AgentTool | Default requires_network_invocation=false and prepare_network_invocation=Ok(None); old implementers compile unchanged |
| Core | Separate endpoint/scope/manifest, closed errors, owned PreparedNetworkToolInvocation and live ToolNetworkAuthority |
| PermissionRequest | Private optional binding; additive with_network_manifest; old constructors retain None |
| GrantPreview | Private boxed optional scope and network_scope accessor; public facet/request literals unchanged |
| PermissionSessionState | admit_network / nonblocking try_admit_network; old Path admit rejects network-bound requests |
| PermissionPipeline | authorize_network(request, manifest, user_intent); old public request fields/entrypoints retained |
| Agent | BeforeToolCall, normalization/schema checks precede one preparation; permission hooks, admission and ledger precede owned execution |
| CLI/TUI | Shared exact preview of method/origin/port/path, proxy, address exceptions and redirect limit |
| SDK | Scoped callback receives full preview; argument-only default denies Some(network_scope); None preserves old behavior |

The new canonical core types/traits/errors are re-exported by talos-runtime, preserving the
single-direct-Talos-dependency SDK contract. ToolNetworkScope::new_configuration creates one
fresh immutable configuration identity; store that scope on the tool and clone it into each
fresh manifest. The independently locked external consumer exercises this through the SDK
actor in both minimal and coding feature modes, without adding a dependency or changing locks.

Native manifest requests require complete exact Network/Domain host facets, including proxy.
MCP provenance, wildcard and untyped profiles cannot compile authority. Domain policy remains
separate from exact scope. Preserve existing matching Deny precedence and non-Deny first-match.

Complete configuration/destination scope participates in Session match; query and fresh invocation
ID are not reusable scope. Exact input/ID/scope remain fingerprint-bound for proposal/admission.
None/Some grants cannot upgrade or downgrade. Live authority rechecks Session identity, relevant
policy/mode/workspace/registration/restriction generations, current Deny, actual grant existence
and explicit clear reset. Unrelated grant creation does not revoke admitted authority. Contention
fails closed, without a lock across await. Logical-start fencing cannot unsend exposed bytes.

## Storage Bounds

| Bound | Limit | Limitation |
|---|---|---|
| Destination entries | 1..64 before deduplication | Constructor fixtures; not a request-count budget |
| Encoded path | 1..2048 ASCII bytes, leading slash | No query/fragment/control characters or dot-segment aliases |
| Host | Canonical domain <=253 bytes / labels <=63, or canonical IP | Caller derives identity from actual parsed HTTP URL; no DNS during preparation |
| Private address exceptions | <=16 before deduplication | Explicit trusted-host choices; not resolved-address enforcement |
| Redirect metadata | <=5 | Actual client must disable implicit redirects before activation |
| Method | GET / HEAD / POST / CONNECT | Existing search/probe/proxy vocabulary; no wildcard method |

Positive/negative constructor bounds are tested. Query/body/response bounds, actual request count,
hedging/deadline and DNS/proxy/connection enforcement remain controlled-transport gates. These
storage limits alone establish neither those runtime limits nor SSRF protection.

The IP regression derives destination identity from actual parsed HTTP URLs, including mapped
IPv6, then compiles Domain policy and admits/checks the plan. HTTP URL serialization's mapped
IPv4 tail is hexadecimal; std::net display uses dotted IPv4. Core validation deliberately uses
the HTTP representation, with no new dependency or copied URL serializer. This is host identity
evidence only; mapped-address/private-network connection classification remains a transport gate.

## Root And Wrapper Dispositions

| Path | Carrier stage |
|---|---|
| Real Agent loop | Offline native fixture executes the approved owned object; final hook rejection/revocation, ledger replay, missing pipeline/preparation and caller-drop tested |
| Real SDK actor | Native offline fixture executes after scoped approval; legacy AlwaysApprove callback is not invoked or upgraded |
| CLI/TUI | Complete preview/channel tested; binary WebSearch activation remains Pending |
| SDK explicit policy Allow | Trusted-host policy can admit complete scope without callback; native manifest admission remains required |
| Existing test-only legacy wrappers | Unsupported for the new path; not promoted to production network adapters |
| External wrappers | Must forward opt-in/preparation or fail closed; WebSearch legacy execute/execute_authorized must deny before activation to prevent hidden-opt-in bypass |
| MCP/standalone search | No new authority; activation requires explicit fail-closed or trusted-host pipeline disposition and real-dispatch evidence |

No PathBuf carrier, browser route or model control field is repurposed. Native embedding is trusted,
not an OS sandbox. Errors are content-free; Debug/fingerprints remain redacted. Existing untyped
Network behavior is preserved. This stage claims no live query or HTTP/DNS cancellation evidence.

## Activation And Reversal

No default switch before guarded compatibility request/parser/probe/retry/source parity, bounded
D migration/terms/privacy review, MIT notice/per-file provenance and actual connection/redirect
evidence. Never automatically call opaque/legacy execution after stale authority or denial.
Reversal is controlled version/deployment revert or separately accepted migration. No E/F native
admission, all-built-in Auto/H, I300 Git, generic NET retries, new crate/dependency or release.
