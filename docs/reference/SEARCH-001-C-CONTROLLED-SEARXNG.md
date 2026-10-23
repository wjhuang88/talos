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
