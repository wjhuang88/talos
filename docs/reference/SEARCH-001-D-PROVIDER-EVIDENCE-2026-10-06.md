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
| Mojeek direct adapter | Candidate on hold | Requires endpoint and index independence confirmation | Protocol stability, parser ownership and rate limits are open | Official terms restrict automated access and scraping; see dated review below | No live acquisition authorized by this review | Blocked for direct HTML admission pending provider permission |
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

## Dated Source Review — 2026-10-07

This is a source-level admission review, not legal advice, provider permission, a regional
reachability measurement, or a completed privacy assessment. Reviewed sources are official
provider pages; the date is the retrieval date, not an inferred policy effective date.

| Evidence | Official source | Observation | Talos disposition |
|---|---|---|---|
| D-001 | [Mojeek Terms of Service, section 3.5](https://www.mojeek.com/about/terms.html) | The terms restrict automated access except for authorized API users and prohibit scraping without prior consent. | Direct HTML candidate is blocked pending documented provider permission; do not acquire search-response fixtures or perform automated endpoint tests on this route meanwhile. An authorized API is a separate admission path, not proof of zero-key eligibility. |
| D-002 | [DuckDuckGo result sources](https://duckduckgo.com/duckduckgo-help-pages/results/sources) | DuckDuckGo describes multiple sources and its own crawler/indexes, but says traditional links and images largely originate from Bing. | DDG and Bing must not count as two independent routes solely because they have different adapter names or front-end hosts. Shared upstream dependency requires a documented failure-domain comparison. This does not prove identical reachability or total dependence. |

These observations narrow the research queue without admitting any route. Endpoint accessibility,
robots directives, a working third-party scraper or an existing compatibility dependency do not
establish permission for a new Talos-native adapter.

### Remaining acquisition gates

- DuckDuckGo HTML/Lite: review automation permission and privacy, then document exact endpoint,
  parser ownership and authorized fixtures. D-002 remains an independence risk.
- Bing CN: review applicable automation terms and privacy before any live search acquisition;
  do not infer mainland-China reachability from the hostname.
- Mojeek: retain the blocked direct route until permission is recorded; no outreach or paid API
  enrollment is authorized by this document.
- Regional observations: no genuine mainland-China test environment has been established in this
  research session. All regional availability remains unknown. A search-service retrieval of a
  documentation page is not a Talos endpoint test or regional availability evidence.

E/F remain blocked: this review supplies neither two admitted native routes nor a regional evidence
packet. SEARCH-001-D remains Active / Claimed with incomplete acceptance.

## Dated Terms Review — 2026-10-08

| Evidence | Official source | Observation | Talos disposition |
|---|---|---|---|
| D-003 | [DuckDuckGo Terms](https://duckduckgo.com/terms) and [Acceptable Use Policy](https://duckduckgo.com/acceptable-use) | Terms incorporate the AUP. The AUP restricts displaying portions of the service inside another service, resale, unauthorized access and service disruption. | Whether normalized search results in Talos fall within the display restriction remains unresolved. This review does not establish permission for a distributed HTML/Lite adapter or a blanket ban on every automated query. Keep admission pending. |
| D-004 | [Microsoft Services Agreement](https://www.microsoft.com/en-us/servicesagreement), Code of Conduct 3.a.vi and Bing/MSN 14.f.i | The retrieved page states publication on July 30, 2026 and effectiveness on September 30, 2026. It restricts circumvention and impermissible scraping; Bing/MSN materials have personal noncommercial-use conditions and qualified conditions for copying, redistribution and building products. | This is a general consumer agreement, not established applicable Bing CN authorization. Record an unresolved product-use/automation and region-specific terms gate; do not infer that every search is prohibited or that consumer access licenses a Talos adapter. |

Dates above record source retrieval; D-004's publication/effective dates are taken from the page.
No live search query or search-response fixture was acquired in this review.

### Research disposition and next deliverable

The current candidate set has no native zero-key route ready for production admission:

- Mojeek HTML: permission blocker (D-001).
- DDG HTML/Lite: distribution/display interpretation and automation permission unresolved (D-003),
  with shared-upstream risk (D-002).
- Bing CN: applicable regional terms and product-use permission unresolved (D-004),
  with independence evidence still missing.
- Credentialed APIs and self-hosted instances remain outside E/F's default zero-key requirement.

The next D deliverable is a route-specific acquisition packet, not a parser implementation:
identify the applicable terms and permitted interface, record privacy/query exposure and endpoint
ownership, then acquire authorized fixtures and paired regional observations. Alternative zero-key
candidates may be researched under I294, but no new provider is admitted by adding it to the queue.
No provider outreach, enrollment or account creation was performed.

Partial research evidence is merged in #668 and #673. D acceptance remains incomplete; do not mark
I294 or SEARCH-001-D Complete, and do not close #624.
