# SEARCH-001-D Provider Evidence Refresh — 2026-11-01

> Status: Research evidence only; no provider is admitted by this document.
> Owner: SEARCH-001-D / I294 / #644.
> Parent: SEARCH-001 / #624.
> Baseline: `e542b0276acf863c37b890d1e8155846bae2e5b1`.

This refresh rechecks the two strongest independent-index candidates after the 2026-10-09/10
research stages. It does not add a provider, change production routing, authorize E/F, establish
legal advice, or claim regional availability. Published provider documentation and current source
are evidence inputs; they are not substitutes for live controlled-network qualification.

## Executive disposition

| Candidate | 2026-11-01 evidence movement | Remaining blocker | Disposition |
|---|---|---|---|
| Mwmbl standard v2 API | Official API pages now explicitly describe anonymous no-key access; API privacy says search queries are not logged/retained; current source still exposes an unauthenticated v2 route | Result-use licence applicability, documented/source quota inconsistency, live valid-query evidence, transport/failure-domain evidence and controlled regional evidence | Stronger technical candidate, **not admitted** |
| Purili core Search API | Official developer docs explicitly support applications, AI agents and RAG with no account/key/fixed quota; search is documented as Purili crawler/index output; privacy has bounded access-log retention | Current general service/use terms were not independently recovered in this refresh; no live endpoint observation, parser fixture acceptance, failure-domain measurement or controlled regional evidence | Strong second candidate, **not admitted** |

The acceptance condition remains unchanged: two independent native zero-key routes must either
satisfy the complete admission packet or remain explicitly blocked. This refresh does not satisfy
that condition and does not authorize SEARCH-001-E/F.

## Mwmbl — current public API and source refresh

### Published API surface

Sources retrieved 2026-11-01:

- [Mwmbl API developer page](https://developer.mwmbl.org/)
- [Mwmbl API Terms and Conditions](https://developer.mwmbl.org/terms/), effective 2026-05-15
- [Mwmbl API Privacy Policy](https://developer.mwmbl.org/privacy/), effective 2026-05-15

The developer page describes an `https://api.mwmbl.org/api/v2/search/` JSON API and an anonymous
plan of 1,000 requests/month at 1 request/second without an API key or signup. The API terms plan
table also lists an Anonymous tier, while the same terms' introductory access section says API
access requires registration and an API key. Treat this as a published-document inconsistency, not
as permission to ignore either statement. The deployed anonymous contract still needs a successful
bounded observation.

The API terms permit API use for lawful purposes, including commercial applications. Separately,
the intellectual-property section says search results returned by the API are made available under
CC BY-NC-SA 4.0 and remain subject to original-content-owner rights. The API-use permission therefore
does **not** by itself establish that Talos may redistribute/model-context-use result material in
all default-distribution or commercial contexts. D admission keeps a result-use/licence applicability
gate. This is a technical governance interpretation, not legal advice.

The API privacy policy says search queries submitted through the API are not logged or retained and
API request IP addresses are held transiently in memory rather than persisted. Account, billing and
usage metadata have separate retention rules. This materially narrows M-Q07 query-retention risk,
but does not resolve result licensing, deployment behavior or downstream source fan-out.

### Current upstream source

Current upstream `mwmbl/mwmbl` main observed at
`86376eb363fb458f660223cb1111db8ea8f8bf6a` (2026-10-31). Relevant pinned source:

- [v2 search registration / request path](https://github.com/mwmbl/mwmbl/blob/86376eb363fb458f660223cb1111db8ea8f8bf6a/mwmbl/tinysearchengine/search.py)
- [standard and combined ranker setup](https://github.com/mwmbl/mwmbl/blob/86376eb363fb458f660223cb1111db8ea8f8bf6a/mwmbl/search_setup.py)

The v2 handler remains `auth=None`; when no `X-API-Key` is supplied, the shown handler leaves
`monthly_usage` and `monthly_limit` null and executes `ranker.search(q, [])`. Keyed requests enter
separate quota/rate-limit logic. This supports the existence of an anonymous source path but does
not prove how the hosted edge enforces the published anonymous 1,000/month and 1 req/s limits.
The policy/source difference is an explicit deployment-qualification item.

Standard search still constructs `MMRRanker(LTRRanker(..., include_wiki=True))`; combined search is
a separate `CombinedLTRRanker` path. Therefore a successful standard API response can still contain
Wikipedia-origin results. A Mwmbl route can receive native-route credit only when its own-index
provenance is preserved and usable general-web results are demonstrated; Wikipedia-only yield does
not create a second independent generic route.

### Mwmbl row movement

| Row | 2026-11-01 status |
|---|---|
| M-Q01 valid anonymous request | **Pending.** Published API and source support the route, but no valid-query response was acquired in this execution environment. |
| M-Q02..M-Q05 parser/provenance/shape | Partial offline evidence remains accepted only as the existing #693 experiment; no production parser is authorized. |
| M-Q06 transport/failure semantics | **Pending.** No provider HTTP exchange occurred in this refresh. |
| M-Q07 service/data/privacy conditions | **Advanced, not closed.** API use and query-retention policy are materially clearer; result-use licensing and deployed-policy consistency remain unresolved. |
| M-Q08 regional/failure-domain evidence | **Pending.** No controlled-region observation, including mainland China, was obtained. |

## Purili — current public API and privacy refresh

Sources retrieved 2026-11-01:

- [Purili developer page](https://puri.li/developer)
- [Purili Search API documentation](https://puri.li/developer/documentation), marked last updated August 2026
- [Purili integrations](https://puri.li/developer/integrations)
- [Purili hosted MCP integration](https://puri.li/developer/mcp)
- [Purili privacy policy](https://puri.li/privacy), marked last updated July 2026

The developer/API documentation explicitly describes public `/api/search` and `/api/context`
interfaces with no account, no API key and no fixed request quota. It names applications, AI agents,
retrieval pipelines and RAG workflows as intended consumers. `/api/search` is documented as ranked
organic results from Purili's crawler-built index; the response `source` is normally `crawled`.
The hosted read-only MCP surface is likewise documented as public and credential-free.

This is stronger integration-intent evidence than the 2026-10-09 packet. It does not establish a
blanket product-use licence. The previous evidence register cited a general Purili terms page with
personal/noncommercial wording; a current authoritative general terms/use page was not independently
recovered in this refresh. Until that discrepancy is reconciled, distributed-client/result-use
permission remains an explicit admission gate rather than being inferred from developer marketing.

Purili's current privacy policy says it does not retain search history tied to users, does not use
external analytics/profiling, and keeps standard access logs containing IP/path/timestamp for up to
seven days for security before deletion. That gives a concrete query-endpoint exposure bound for
D/privacy review. It is not equivalent to zero exposure, and it is not evidence about all hosting
subsystems or regional accessibility.

### Purili qualification rows to create/complete

A production adapter is still out of scope. Before admission, a Purili packet must cover at least:

1. one permitted bounded valid-query observation with status/content-type/body-size and no raw query
   or result body retained;
2. independently authored fixtures for success, empty, malformed, wrong-type, invalid URL and 5xx
   semantics based on the documented JSON contract;
3. parser/output bounds and URL normalization under the existing SEARCH-001-B/C safety contracts;
4. current applicable service/result-use terms reconciled with the explicit developer/agent
   integration invitation;
5. endpoint/hosting and downstream failure-domain comparison against Mwmbl and the compatibility
   DDG/Bing family;
6. paired controlled real-network observations, including genuine mainland-China provenance.

Purili is therefore the preferred second independent-route research target, but it is **not admitted**.

## Bounded valid-query attempt from this execution environment

A single non-sensitive fixed query was prepared for each public endpoint. The repository records only
its SHA-256 (`0c656b2a3d56f42acf99c1ca7fb2f1663d63f0577858b02904ac240e8e473f03`), not the query text or
any result content. TLS verification and a 15-second client deadline were configured; no credentials,
retries, cookies or bypass mechanisms were used.

Both attempts stopped before connection because this execution environment could not resolve the
provider hostname (`curl` error 6):

- `api.mwmbl.org` — DNS resolution failed before HTTP/TLS.
- `puri.li` — DNS resolution failed before HTTP/TLS.

Because two unrelated provider names failed at the same DNS boundary, this is recorded only as an
**execution-environment DNS limitation**. It is not provider-outage evidence, not a regional
reachability result and not M-Q01/M-Q08 acceptance. No response body was acquired or retained.

## Remaining shortest path for D

1. Reconcile current Purili applicable-use/result-use terms and prepare an offline parser/fixture
   packet comparable to Mwmbl's M-Q02..M-Q05 evidence.
2. Obtain permitted valid-query observations for Mwmbl and Purili from a network where DNS/HTTPS
   execution is actually available; preserve only bounded metadata/provenance, not query text.
3. Compare endpoint/downstream failure domains and prove native-origin yield rather than merely
   different hostnames.
4. Acquire paired controlled regional observations, including genuine mainland-China evidence.
5. Only then make the two-route admission/block decision and decide whether E/F can be activated.

Until these gates close, SEARCH-001-D / I294 remains Active / Claimed, #644 and #624 remain open,
and SEARCH-001-E/F remain unauthorized.
