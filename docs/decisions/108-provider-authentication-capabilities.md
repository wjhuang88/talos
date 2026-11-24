# ADR-108: Provider authentication capabilities

**Status**: Proposed for PROVIDER-003-A / I336 / #1013. Independent security review pending.

## Existing routes

| Route | Evidence on main | Remaining owner |
|---|---|---|
| Static API key | talos-config/src/credentials.rs resolves existing configuration. Keep existing header construction and precedence when auth is omitted. | B verifies legacy equivalence |
| GitHub Copilot | talos-provider/src/copilot.rs pins the public origin for a supplied bearer. factory_copilot_agent_tests.rs exercises that transport. | D owns device authorization, exchange, client identity and storage |
| Google Vertex | talos-provider/src/gemini_auth.rs supplies bounded ADC for the selected Vertex route. #838 covers the Gemini factory path. | F validates reusable acquisition under B/C |
| Azure | talos-provider/src/azure_transport.rs binds the configured resource and credential mode. #978 covers supplied Entra tokens. | B/C preserve supplied-token semantics |
| Bedrock | talos-provider/src/bedrock_transport.rs signs the final serialized request with regional environment credentials. #1009 includes signing-vector and controlled transport tests. | E owns additional credential sources and expiry |

Paths in this table are relative to crates/. The transport evidence establishes no interactive login or general credential chain.

## Proposed decisions

B resolves an opaque credential lease for an explicit profile, account, audience and configuration generation. Keep static-key behavior as the default. Reject ambiguous explicit credential sources before network access. Store access tokens in bounded process memory; persistent refresh material requires an approved OS secret store and explicit enrollment. A headless host without that store can accept a supplied credential or return an actionable configuration error. Never fall back to writing tokens in TOML.

Use one refresh flight per lease key. A cancelled waiter detaches without cancelling other waiters. Logout, shutdown or profile replacement cancels the producer and increments its generation. Publish a result only if the generation still matches. Convert wall-clock expiry to a bounded monotonic deadline with a safety margin. Discard expired leases and cap cache cardinality. B defines numeric limits and tests clock changes before activation.

C authorizes a sealed outbound request after route, method, path, query, payload and relevant headers are final. The authorizer verifies the profile's exact approved destination before attaching credentials. SigV4 signs that sealed representation anew on each admitted attempt. No redirect inherits authorization. Acquisition endpoints and inference endpoints have separate audiences. TLS verification remains enabled.

A 401 permits at most one generation-checked invalidation and reacquisition. Resending requires explicit adapter evidence that the operation is replay-safe, a reproducible body, a live deadline and no delivered stream event. Without that evidence surface the authentication error. Share a single attempt budget with transport retries; never switch protocol or provider on failure.

D limits device login to an explicit interactive action and respects polling interval, expiry, cancellation and denial. E binds signing identity to region and service. F constrains ADC acquisition and refresh destinations. G requires a separately authorized local helper, literal argv without shell evaluation, restricted environment, bounded stdout, timeout, process cleanup and redacted stderr. Repository configuration alone cannot authorize helper execution.

## Review and negative acceptance

| Threat or compatibility failure | Required observation | Owner |
|---|---|---|
| Concurrent refresh and stale completion after logout | One acquisition for parallel waiters; stale result cannot restore the session | B |
| Token sent to redirected host or different tenant | Capture both destinations; second receives no credential | C/D/F |
| Request changed after signing or retried after first output | Signature covers exact bytes; reject mutation or replay | C/E |
| Secret in debug, preview, history or failure | Synthetic secret absent from all four sinks, including nested HTTP errors | B–G |
| Malicious helper input or hung descendant | Refuse missing grant; bounded execution, cleanup and output | G |
| Static configuration drift | Compare effective route, headers and setup behavior across CLI, desktop and SDK | B/C |

B and C remain separately governed. Vendor-specific implementations require their own fixtures, source review and claims. Product code, schema migration and production enablement remain outside I336.

## Authorization invariants and review fixtures

B identifies each acquisition by both the profile generation and a unique lease revision. A dispatch carries the revision it used. Invalidate by compare-and-swap against both fields. A late 401 from lease L1 cannot evict L2 within the same generation. Before handing a sealed request to transport, check the current generation, revision, expiry and binding together. If any changed, discard the authorized request; any permitted new attempt requires fresh authorization within the original budget. Test by holding L1's response until L2 is installed, then delivering its 401; L2 remains available and the delayed request cannot dispatch under L1.

C takes destination authority from built-in reviewed policy or explicit user/admin enrollment outside project-controlled input. Bind the account, tenant, credential source, issuer, exchange endpoint and inference resource independently. Pin approved HTTPS origins, ports and route constraints. A project may select an enrolled profile; a project endpoint override, model catalog entry, discovery response or redirect cannot expand its grant. Test substitution of each destination independently; the capture endpoint must see zero requests containing the credential. Re-enrollment changes the generation and revokes cached authorizations.

B's compatibility fixtures compare an omitted auth selector with legacy static-key resolution, including configured key/environment precedence, missing variables and explicit provider headers. Compare redacted errors and effective requests using synthetic values. When persistent enrollment is requested and the OS secret store is locked or unavailable, fail that enrollment without writing a fallback file. G drains helper stderr under a fixed byte limit and discards its contents; expose only the exit class or timeout. Test a helper that prints a synthetic token on stderr, and inspect exported diagnostics.

Protocol references: [OAuth Device Authorization Grant](https://www.rfc-editor.org/rfc/rfc8628), [GitHub device flow](https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/authorizing-oauth-apps#device-flow), [AWS signed requests](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_sigv-create-signed-request.html), [Google ADC](https://cloud.google.com/docs/authentication/application-default-credentials). These references support protocol research; specific accounts, exchange services and destinations still require enrollment.
