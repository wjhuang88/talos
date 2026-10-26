# Controlled SearXNG adapter (dormant)

The private SearXNG adapter is preparation and transport infrastructure for I293. It is
not registered as a tool, selected by the router, or activated by configuration/environment
variables. The existing `web_search` behavior and model-facing contract remain unchanged.

A caller must explicitly construct a configuration from an HTTP(S) base URL. Its path prefix
is preserved when appending `/search`. Credentials, query, fragment, raw control characters,
and surrounding whitespace are rejected. Configuration creates a stable query-free endpoint
scope, not permission. Each prepared request owns a fresh invocation manifest and the exact
encoded query. Approval must bind that manifest; execution consumes the prepared request and
requires live authority plus an existing shared transport budget. No replacement URL, query,
proxy, DNS resolver, or manifest can be supplied at execution. Direct transport retains the
existing DNS/address checks; explicitly configuring a private host does not authorize its IP.
Redirects and ambient proxy discovery are disabled.

The request preserves the legacy `format=json&q=...` encoding, including `%20` for spaces.
Query bytes are bounded before encoding, and the encoded query is at most 4096 bytes; the
complete URL is at most 8192 bytes. Output count is 1–20. Transport remains subject to its
shared request, byte and deadline limits. JSON input is additionally bounded to 1 MiB and
at most 20 response candidates. Larger arrays fail rather than silently truncating. This is
a deliberate policy difference from legacy SearXNG parsing, not evidence of full parity.

Missing titles use `Untitled`; `content` takes precedence over `snippet`. Existing result
admission checks every materialized candidate's field sizes before output truncation, then
selects HTTP(S) URLs without credentials. Invalid URL candidates are filtered; URL validation
may stop once the requested number of admissible rows is reached. Titles/snippets are bounded
to 16384 bytes and URLs to 8192 bytes. Results retain `SearXng` identity and grant no fetch or
HTML rendering authority. Error codes and Debug output contain no query or response data.

Budget checks fence parsing and admission completion, including error outcomes. Synchronous
JSON parsing cannot be preempted by those checks. Unwind recovery does not suppress panic
hooks or recover from process aborts/OOM. Native CI, source/parity/privacy evidence and the
remaining router/entry-point gates are still required before activation.

## Prepared invocation bridge (dormant)

The private SearXNG invocation implements the existing prepared-network trait. It retains the
original plan and delegates its exact manifest/profile without rebuilding configuration or
authority. Query text and snippets options are captured before approval. A single execution
budget starts only on actual trait-object dispatch, then covers controlled HTTP, parsing,
admission and the final formatting fence. Waiting for approval consumes no execution time.

Formatting uses a fixed SearXNG source label and the existing display format, including snippets
selection and UTF-8 truncation. Source labels do not come from response data or user input;
display conversion is not result admission. A final deadline failure takes precedence over
formatted success and transport/parser errors. Caller cancellation drops the owned execution.
This bridge is not an AgentTool registration, configuration discovery or official entry point.


## Prepared-only tool wrapper (dormant)

The private tool wrapper owns a caller-supplied validated SearXNG configuration and transport
limits. It uses the unchanged `web_search` input schema, defaults and 1–20 result-count clamp.
Preparation captures query and snippets options in a fresh invocation bound to that exact
configuration; later input changes and wrapper disposal cannot replace them. Malformed input
returns a fixed preparation error without echoing input or endpoint data.

The wrapper requires network preparation. Every legacy execution surface returns
`search_prepared_invocation_required`, including when path authorization is supplied. Only the
prepared object can execute with matching live network authority. There is no environment/config
lookup, official registration, background execution or Router/default activation in this stage.
