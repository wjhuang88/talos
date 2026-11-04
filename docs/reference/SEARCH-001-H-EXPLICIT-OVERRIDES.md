# SEARCH-001-H: Explicit `web_search` Provider Overrides

This reference documents the user-visible configuration introduced by
SEARCH-001-H / [Issue #832](https://github.com/wjhuang88/talos/issues/832).
It is independent of SEARCH-001-D's BLOCKED / ZERO-ADMISSION disposition.

## Default, selection and ordered fallback

The optional `[search]` section is in `~/.talos/config.toml`.
If omitted, `backend = "auto"` and `fallback_to_auto = false`.
Only an explicit backend value selects premium or self-hosted search.

| `search.backend` | First candidate | If `fallback_to_auto = true` | Credential/endpoint needed |
|---|---|---|---|
| `auto` (default) | DuckDuckGo compatibility | Not valid | None |
| `duckduckgo` | DuckDuckGo compatibility | Not valid | None |
| `tavily` | Tavily | Then DuckDuckGo, sequentially | Explicit Tavily API key |
| `searxng` | SearXNG | Then DuckDuckGo, sequentially | Explicit `search.url` |

`fallback_to_auto` defaults to `false`. It is only valid for Tavily and
SearXNG. For an explicitly selected optional backend, missing or invalid
credential/endpoint fails **during configuration/composition**. Fallback does
not repair invalid explicit configuration. When enabled, fallback is attempted
only **after** a validly selected provider executes and fails, or produces no
admissible results. A successful first candidate suppresses later candidates.
Execution still obeys one shared budget, deadline and cancellation policy.
Auto retains the original 135-second/24-physical-request-unit limit. An explicit
Tavily/SearXNG selection with opted-in Auto fallback uses a bounded 150-second/
26-unit envelope to admit the optional HTTPS request plus full proxied Auto
compatibility attempts. Fallback is ordered, not a race; deadline exhaustion
still terminates the invocation rather than creating an unbounded retry.

## Examples

Keep the existing top-level `provider` and `model` settings for the LLM.
These snippets describe the independent **search** selection.

Default, free/zero-key search (also the behavior when `[search]` is absent):

```toml
[search]
backend = "auto"
```

Explicitly select the same zero-key DuckDuckGo candidate:

```toml
[search]
backend = "duckduckgo"
```

Explicit Tavily with **opted-in** Auto fallback:

```toml
[search]
backend = "tavily"
fallback_to_auto = true
```

Store the Tavily key separately in `~/.talos/credentials.toml` (the existing
provider-name-to-key credentials store):

```toml
tavily = "YOUR_TAVILY_API_KEY"
```

The effective config loader merges that stored key into the `tavily`
provider entry. An explicitly configured `[providers.tavily].api_key` can
also be read, but putting secrets in `config.toml` is not recommended.
The H projection does **not** inspect `TAVILY_API_KEY` or
`providers.tavily.api_key_env` as an ambient key source. A present
key alone **never** selects Tavily: `backend = "tavily"` is mandatory.

Explicit SearXNG with no fallback:

```toml
[search]
backend = "searxng"
url = "https://search.example.org"
fallback_to_auto = false
```

The endpoint must satisfy the existing controlled SearXNG URL and
private-address policy; setting a local/private endpoint does not implicitly
authorize that network destination. A present `SEARXNG_URL` environment
variable does not activate SearXNG. `search.url` is required only for
`backend = "searxng"` and is rejected for other backends.
Do not embed usernames, passwords or tokens in URLs.

## Privacy, transport and compatibility

- Tavily requests can consume provider quota **only after explicit selection**.
  The adapter uses sensitive/redacted bearer headers and does not follow
  redirects carrying credentials. Do not publish `credentials.toml`.
- User-facing errors, selection diagnostics and backend health metadata must
  not expose API keys, auth headers or credential-bearing URLs. The explicit
  SearXNG URL is **persisted** in `config.toml`, so avoid secrets in it and
  protect that file appropriately.
- All selected candidates share one frozen production proxy-policy snapshot,
  including the actual transport proxy. This preserves existing proxy
  behavior; H introduces no proxy discovery, VPN or region-based routing.
- The older public `WebSearchTool::new()` constructor is now zero-key even if
  `TAVILY_API_KEY` or `SEARXNG_URL` are set. Its DuckDuckGo/Wikipedia compatibility
  implementation remains available for deployment rollback; optional providers
  require the protected explicit-selection path. This is an intentional security-
  driven behavior migration for callers previously relying on ambient activation.
- The model-facing `web_search` tool schema and Network permission remain
  unchanged. The existing router owns fallback, deadline, cancellation and
  typed-error semantics. The shipped `rust-websearch` compatibility
  dependency remains; SEARCH-001-I owns any future removal.
- SEARCH-001-E/F, Mwmbl and Purili are not admitted by this feature.

## Disable / rollback

To disable an optional provider without deleting its stored credential, set
`[search] backend = "auto"` and **remove** `fallback_to_auto` and `url`,
or remove `[search]` entirely. This immediately returns to the default
zero-key route; ambient keys/endpoints remain inert. No migration of old
configurations is required: absence of `[search]` deserializes to Auto.

If rolling back the implementation itself, revert the SEARCH-001-H
configuration/composition commits and retain the existing compatibility
search path; do not remove `rust-websearch`. The new config section is
additive, not a prerequisite for legacy Auto.
