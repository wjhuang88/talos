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
| Mwmbl standard JSON search | Source supports anonymous search; deployment unverified | Own index plus external reference results; source path must be distinguished from combined search | Versioned JSON; current source/test evidence available | Public-use terms, dataset conditions and query fan-out require route-specific review | Unknown; no live query acquired | Priority source-level candidate; not admitted |
| Purili core search API | Official docs state no key/account | Provider claims own crawler/index; infrastructure not independently measured | Documented JSON; server source/license unverified | Developer integration invitation and personal/noncommercial general terms need reconciliation | Unknown; no live query acquired | Candidate pending applicable-use/privacy evidence; not admitted |
| Stract hosted search | Historical candidate only | Archived engine claims own index; current hosted route unestablished | Repository archived; historic beta endpoint is not current evidence | Current hosted site describes a different product | Unknown; no search measurement | Park hosted default-route research; retain architecture reference only |

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

## Independent-Index / Public-API Review — 2026-10-09

This source-only review follows the maintainer's technical-first direction recorded in #644
comment `6055213313`. Published interface terms are evaluated per route; bespoke business outreach
is not a universal prerequisite for research or offline fixtures. Explicit restrictions and
unresolved applicability remain recorded. No provider contact, account, paid call, API key or
live search-response acquisition occurred. Documentation retrieval is not regional evidence.

| Evidence | Primary source (retrieved 2026-10-09) | Observation / disposition |
|---|---|---|
| D-005 | [Mwmbl search source](https://github.com/mwmbl/mwmbl/blob/ff482e225d99c694abbd3337a1b1be93b8c95dd8/mwmbl/tinysearchengine/search.py), [API registration](https://github.com/mwmbl/mwmbl/blob/ff482e225d99c694abbd3337a1b1be93b8c95dd8/mwmbl/api.py), [upstream auth tests](https://github.com/mwmbl/mwmbl/blob/ff482e225d99c694abbd3337a1b1be93b8c95dd8/test/test_search_api_key.py) | Standard v2 search accepts no key, returns JSON and null usage fields; an optional supplied key activates auth/quota processing. Upstream has an anonymous-request test. This is source/test inspection, not execution or proof of deployed behavior. Prioritize anonymous standard search; do not attach an ambient key. |
| D-006 | [Mwmbl routing setup](https://github.com/mwmbl/mwmbl/blob/ff482e225d99c694abbd3337a1b1be93b8c95dd8/mwmbl/search_setup.py), [ranker](https://github.com/mwmbl/mwmbl/blob/ff482e225d99c694abbd3337a1b1be93b8c95dd8/mwmbl/tinysearchengine/rank.py), [LTR implementation](https://github.com/mwmbl/mwmbl/blob/ff482e225d99c694abbd3337a1b1be93b8c95dd8/mwmbl/tinysearchengine/ltr_rank.py), [terms](https://mwmbl.org/terms), [privacy](https://mwmbl.org/privacy) | Standard setup includes Wikipedia; combined ranking is a separate path. Terms restrict scraping/excessive queries and assign dataset conditions. Privacy describes external query processing. Review actual route fan-out, provenance and result-use conditions; do not equate an open-source server or anonymous endpoint with admission. |
| D-007 | [Purili developer page](https://puri.li/developer), [API reference](https://puri.li/developer/documentation), [about](https://puri.li/about) | Provider documents uncredentialed core JSON search and describes its own crawler/index. This is a promising interface claim, not independently verified infrastructure or uptime. Talos can evaluate a native HTTP adapter without adding its JavaScript SDK or another runtime. |
| D-008 | [Purili terms](https://puri.li/terms), [privacy](https://puri.li/privacy) | General terms limit personal/noncommercial use and forbid bulk extraction/abusive requests; developer pages invite app integration. Applicable distributed-client use remains unresolved. Policy says access logs retain IP/path/time up to seven days; query-bearing request exposure requires review. No default admission or live fixture acquisition on the strength of marketing copy. |
| D-009 | [Stract repository](https://github.com/StractOrg/stract), [current hosted site](https://stract.com/) | GitHub reports archive on 2026-04-02; hosted site now describes media/outreach tooling. Historical independent-index README and beta API adapters do not establish a maintained current generic-search service. Park this hosted route; do not infer every fork or future deployment is unusable. |

### Mwmbl route-specific acquisition packet

The inspected upstream baseline is `ff482e225d99c694abbd3337a1b1be93b8c95dd8`; source paths
above are pinned. The [URL configuration](https://github.com/mwmbl/mwmbl/blob/ff482e225d99c694abbd3337a1b1be93b8c95dd8/mwmbl/urls.py)
and API registration place standard search at `/api/v2/search/` with query parameter `q`, and
legacy v1 list search at `/api/v1/search/` with parameter `s`. These are source-defined paths;
actual host, deployed version, redirect behavior and endpoint stability remain unverified.
The [official architecture guide](https://book.mwmbl.org/page/architecture/) discusses API-domain
deprecation, so a historical `api.mwmbl.org` URL must not be silently treated as the stable target.

| Packet item | Source-level finding | Remaining evidence |
|---|---|---|
| Credentials / cost | No-key standard v2 path is distinct from optional keyed and combined routes | Verify deployed anonymous contract; no automatic credentials or paid-path fallback |
| Normalization | v2 exposes result title, URL, content and engine provenance; v1 uses text segments | Synthetic fixtures for shape, Unicode, missing fields, malformed/empty and URL validation; no copied response fixtures in this stage |
| Own index / external sources | Standard setup wraps an LTR ranker with Wikipedia enabled; results can have different origins | Source LTRRanker.external_search calls get_wiki_results when enabled; verify deployed query fan-out; preserve provenance and ensure Wikipedia-only results never count as an independent generic route |
| Terms / privacy | Public-use API exists alongside service/dataset restrictions and external-processing policy | Identify applicable API conditions, attribution/result retention and model-context use; resolve policy versus route-specific behavior, without assuming bespoke outreach is mandatory |
| Maintainability | Source and upstream anonymous/auth-failure tests are inspectable at a fixed SHA | Adapter/parser owner, drift fixtures, payload/timeout/redirect bounds and rollback; inspect tests without claiming they were run |
| Failure domains | Different named service from DDG/Bing; own-index claim alone is insufficient | Paired actual host/TLS/network outcomes plus downstream/hosting comparison |
| Geography / quality | No endpoint search or relevance test performed | Controlled paired observations, including genuine mainland-China provenance, language coverage and valid-result yield |

The main Mwmbl response source labels and Wikipedia-enabled setup make provenance a technical
qualification task, not a reason to discard the candidate solely because it exposes a paid option.
Server code is not proposed as a Talos dependency, and no AGPL code or dataset is copied into Talos.
Source-code licensing, service access conditions and search-result reuse are distinct questions.

### Next technical checkpoint

Prioritize Mwmbl standard anonymous-search deployment and provenance verification. Purili remains
a second interface research queue with unresolved applicable-use conditions and unverified server
source/license; it is not described as an established open-source engine. A documentation fetch
failure cannot establish endpoint outage, and successful documentation retrieval cannot establish
search success. Regional outcomes, independent failure-domain proof and native fixture acceptance
remain unknown. No candidate is admitted; D/I294 remains Active / Claimed, E/F remain gated and
#624 remains open.
