# SEARCH-001-D Provider Admission Evidence Register

> Status: Working evidence register; no provider is admitted by this document.
> Owner: SEARCH-001-D / I294 / #644
> Claim: Governance PR #665, merged as `7019f63d3df3d6eb58473a53a499b9b7e8b98925`
> Baseline: `4a4e358e9f6dacc75af1f2a6f06eb400f0f0da2a`

This register turns ADR-085's provider-admission gate into an auditable research packet. It is
evidence-only: it does not add a provider, change routing, enable a relay, or change the
model-facing `web_search` contract.

## Candidate matrix

| Candidate | Zero-key target | Upstream independence | Parser/maintenance question | Terms/privacy question | Regional evidence | Disposition |
|---|---|---|---|---|---|---|
| `rust-websearch` auto path | Compatibility only | Unknown until the upstream source is surfaced | Talos does not control upstream backend selection or source identity | Must re-check upstream terms and query exposure | Existing behavior is not regional admission evidence | Keep as rollback adapter; never count as native route |
| DuckDuckGo HTML/Lite direct adapter | Candidate | Requires direct endpoint confirmation and failure-domain testing | HTML/anti-bot drift and fixture ownership are open | Automated-query policy and query exposure require source review | Must be tested from genuine controlled regions | Not admitted; evidence pending |
| Bing CN direct adapter | Candidate | Requires direct endpoint and infrastructure independence from DuckDuckGo | HTML/protocol stability and parser ownership are open | Automated-query policy, query handling and regional terms require source review | Must be tested from genuine mainland-China and comparison networks | Not admitted; evidence pending |
| Mojeek direct adapter | Candidate | Requires endpoint and index independence confirmation | Protocol stability, parser ownership and rate limits are open | Terms, automation and query retention require source review | Must be tested from controlled regions | Not admitted; evidence pending |
| Brave Search API | No; credentialed API | Independent index claim requires source evidence | Structured API is maintainable, but credential/error semantics apply | Key handling, quota, metering and query policy require explicit review | API reachability is not zero-key evidence | Explicit/paid candidate only; excluded from E/F zero-key admission |
| SearXNG | No; instance required | Depends on the selected instance and its upstreams | Instance-specific protocol and result quality | Instance trust, SSRF, query exposure and operator policy are open | Instance/network evidence is not generic-provider evidence | Explicit self-hosted compatibility path only |
| Tavily | No; credentialed API | Provider infrastructure is independent but metered | Structured API is maintainable, with auth/quota semantics | Paid usage must remain explicit opt-in | API reachability is not zero-key evidence | Explicit premium path only |

The matrix deliberately leaves all native candidates unadmitted. Two wrappers around the same
opaque upstream auto-router do not satisfy ADR-085's independence rule.

## Required evidence packet per candidate

For every candidate that remains viable, record:

1. Exact endpoint, protocol shape, redirect behavior and TLS assumptions at a dated commit.
2. Terms/automation interpretation, rate-limit/challenge behavior and query exposure.
3. Deterministic fixtures for success, empty results, malformed response, blocked/challenge,
   rate-limit and URL validation failure.
4. Parser ownership, update surface, dependency impact and rollback path.
5. Endpoint and failure-domain comparison against every other candidate.
6. Reproducible reachability observations from controlled regions, including a genuine mainland-
   China network when available. Record coarse region/network class and direct-vs-user-proxy
   status; never store personal IPs or query text.

## Regional evidence record

The following fields are mandatory for each live observation; an unavailable environment remains an
explicit unknown rather than a simulated result:

| Field | Required value |
|---|---|
| Talos commit | Full commit SHA |
| Date/time | UTC timestamp |
| Coarse region | For example US-West, EU, Singapore, mainland China |
| Network class | Residential, mobile, cloud, or unknown |
| Proxy | Direct, user-configured proxy, or unknown |
| Endpoint | Host/path without query text or secrets |
| HTTP/TLS outcome | Status, timeout, connect/TLS failure or challenge class |
| Parse outcome | Valid, empty, malformed or blocked |
| Result URL policy | Count of admissible HTTPS/HTTP URLs after normalization |
| Comparison route | Whether another candidate was tested in the same environment |

No locale, GeoIP header, foreign-region cloud result or synthetic proxy result can be recorded as
mainland-China availability evidence.

## Admission decision gate

A native zero-key candidate may move to E/F only when the packet has dated source links, deterministic
fixtures, an owned parser/update path, acceptable terms/privacy review, independent endpoint/failure
domain evidence and controlled regional observations. Until then, the candidate remains outside the
default distribution and the accepted compatibility path remains the rollback boundary.
