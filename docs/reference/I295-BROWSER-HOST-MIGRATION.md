# I295 Browser Host Migration

Status: local implementation candidate; not released or accepted as complete.
Decision: [ADR-086](../decisions/086-frame-aware-browser-invocation-boundary.md).
Acceptance owner: [I295](../iterations/I295-web007f-frame-aware-browser-contract.md).

## Explicit composition

Existing Runtime and tool consumers need no migration unless they opt into browser v2.
`RuntimeBuilder::new()` does not install a browser. V1 is not silently upgraded.
The host implementation currently lives in `talos_tools::browser_executor`, enabled by
the `network` feature. Embedded products use `talos-runtime` for Runtime composition and
explicitly depend on `talos-tools` for this optional adapter. The existing single-direct-Talos-
dependency SDK fixture still tests the core facade, not browser adapter dependency closure.

A host implements `BrowserExecutor`, constructs `BrowserHost`, and populates trusted lifecycle
state through `context_mut()` before transferring ownership to `ManagedBrowserTool::new(host)`.
Register the resulting tool through `RuntimeBuilder::tool`. Registration alone grants nothing:
install an explicit `BrowserPermissionResolver` using `with_permission_resolver` to bridge a
browser-aware policy or one-invocation human approval. Generic `ApprovalHandler`, Network rules,
workspace trust and session grants cannot approve browser operations.

`ExactBrowserPermissionEvaluator::for_resource` is a trusted authority primitive, not a policy
engine. Call it only after approving that exact resource; blindly accepting every resource is
not an appropriate production policy. The normalized payload digest binds fill/select values
without exposing them through approval metadata. Authorization cannot be reused for another
invocation even when operation and origins match.

## Provider and tool adapters

`LanguageModel::stream_with_invocation_integrity` has a default legacy implementation, so
existing provider implementations continue to compile. Its legacy events cannot authorize an
original-only tool. Providers supporting v2 return `ProviderInvocationStream::original` with
`ProviderInvocationEvent::BrowserToolCall`, constructed from the actual original argument bytes
using `BrowserRawArguments::parse_original`. Reserializing a parsed JSON value loses duplicate-key
evidence and must never be used to manufacture this proof. The carrier is transient and separate
from serializable conversation events.

`AgentTool` adds default-denying prepared methods and `requires_original_invocation`. Wrappers
around original-only tools must forward this capability and the prepared path, or refuse the
tool. They must not translate it into legacy parsed-value execution. The owned
`PreparedToolInvocation` and `AuthorizedToolInvocation` traits separate admission/authorization
from execution; the Agent retains its final permission hook before consuming execution.
The Runtime facade exports these generic types and the provider invocation types. Browser
resource, evaluator, executor and host types remain in `talos-tools`.

Standalone MCP lacks the required original-byte ingress proof and therefore hides/refuses these
tools. Adding v2 to MCP requires a separately proven ingress path, not a registration switch.

### Request-only screenshot attachments

`ContentPart` and the existing `ToolExecutionOutput` structure remain unchanged. Screenshot
bytes travel through the separate `PreparedExecutionOutput` and `EphemeralImage` carriers;
they are not serializable history events. The Runtime facade reexports both carriers and
`PreparedFailureCode`. Existing `AuthorizedToolInvocation` implementations can retain only
`execute`: the default `execute_with_attachments` produces no images.

Providers opt in through `LanguageModel::stream_with_ephemeral_images`. Its default refuses
nonempty images before dispatch and delegates empty requests to the legacy path. The Agent
requires image-input capability and reserves image cost, including the configured safety
margin, in the sealed request budget before dispatch. Attachments move into the immediately
following provider request once; hook previews, display summaries and persisted messages do
not receive their bytes. OpenAI and Anthropic wire adapters preserve existing text when
appending PNG blocks. The attachment deadline bounds the whole request setup, including
error-body reads and retry backoff; expired attachments cannot start a later attempt.

## Lifecycle, output and cancellation

Trusted host lifecycle state is not model input. Navigation, frame replacement, detachment,
tab lifecycle changes and session replacement invalidate affected references and approvals.
After rejection, perform fresh authorized discovery; do not retarget an old reference or retry
an ambiguous mutation. Backend support must attest document-bound action/capture semantics;
global keyboard focus or coordinate fallbacks are not equivalent.

After composition, retain `ManagedBrowserTool::lifecycle_handle()` for lifecycle subscriptions.
`invalidate_pending()` is lock-independent: it revokes pending approvals, wakes a parked dispatch,
and prevents subsequent admission against the old context. Restore trusted discovery using
`ManagedBrowserTool::synchronize_context`; this replaces the session and all references. A
concurrent invalidation during rebuilding requires another refresh. A panicking rebuild fails
closed, and an exhausted generation cannot wrap into an earlier identity.

Dispatch checks lifecycle generation before each backend poll and again after output validation
and projection. The last successful generation load is the result-release linearization point:
invalidation observed before it suppresses the result. This does not make arbitrary backend
effects atomic. The executor must still bind each actual action to the selected document and
reject identity changes at its own effect boundary; dropping its future cannot undo an effect
that already occurred. Such interrupted calls have indeterminate outcomes and must not replay.

Direct host callers pass their business cancellation future to `execute_prepared`. In the
Agent/Runtime route cancellation drops the owned execution future; the dispatch guard invalidates
context if execution had begun without a confirmed completion. Executors must cooperate with
future cancellation and must not detach work or replay it. Re-establish trusted lifecycle state
before admitting further context-dependent work after an indeterminate outcome.

Only validated typed results enter model projection. Display and persistence use content-free
summaries. Executors remain responsible for redacting sensitive fields and excluding descendant
frame content before constructing output; structural validation cannot infer confidentiality.
Native screenshot containment and action atomicity require real-backend evidence.

## Runnable local example

```sh
cargo run -p talos-tools --locked --features network --example i295_browser_host
cargo test -p talos-runtime --locked --test i295_browser_host
```

The example discovers a cross-origin child, authorizes one exact press, and rejects its reference
after invalidation. The Runtime integration tests cover opt-in registration, default denial,
explicit approval and shutdown of an in-flight executor. Neither is a native browser or a claim
that the full mandatory conformance matrix has passed. External independent host consumption,
the remaining conformance evidence and exact-head review remain I295 completion gates.
