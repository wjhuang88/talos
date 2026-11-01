# SEARCH-001-D: Purili Search Route Qualification

> Status: Published-interface trace plus synthetic offline evidence; no backend admission.
> Owner: I294 / SEARCH-001-D / #644; effective claim #665.
> Talos baseline: `e542b0276acf863c37b890d1e8155846bae2e5b1`.
> Evidence date: 2026-11-01.

This packet qualifies only the documented Purili core Search wire shape far enough to support
future admission review. It does not introduce a production adapter, HTTP request path, provider
registration, routing, credentials, relay, account, paid use or regional claim. The executable
fixture is independently authored from the public field contract; it contains no captured Purili
response or copied upstream implementation.

## Published Core Search Interface

Primary sources retrieved 2026-11-01:

- [Purili developer page](https://puri.li/developer)
- [Purili API documentation](https://puri.li/developer/documentation), marked last updated August 2026
- [Purili web-search SDK guide](https://puri.li/developer/web-search)
- [Purili integrations](https://puri.li/developer/integrations)
- [Purili hosted MCP guide](https://puri.li/developer/mcp)
- [Purili privacy policy](https://puri.li/privacy), marked last updated July 2026

The public documentation presents `/api/search` and `/api/context` as credential-free JSON APIs for
applications, agents, retrieval pipelines and RAG. Search is documented as ranked organic output
from Purili's crawler-built index. The HTTP API accepts GET/POST, requires `q` for search, and uses
HTTP status plus a small JSON `error` object for failures. A valid query with no visible matches is
instead documented as HTTP 200 with an empty `results` array.

The documented Search success envelope contains:

- `results: SearchResult[]`;
- numeric `total`, `totalPages`, and `page` pagination metadata;
- string `timeMs`;
- boolean `hasNext`;
- optional/null `correction` and `queryAnalysis` metadata.

A SearchResult contains `id`, `title`, `url`, `displayUrl`, `description`, optional favicon and
`source`. The docs say `source` is usually `crawled` for the independent index. That source label is
provenance metadata only; Talos must not promote it into hosting/failure-domain proof without live
qualification.

The Context API is intentionally outside this offline Search parser experiment. It is separately
documented as stored indexed text with `source: "purili-index"` and `liveFetch: false`. Future use
of stored page content would require its own untrusted-content, size/licence and model-context review;
Search-route admission does not silently authorize Context ingestion.

## Privacy And Use Boundary

The current privacy policy states that Purili does not retain search history tied to users or use
external analytics/profiling, while standard access logs containing IP/path/timestamp may be kept
for up to seven days for security before deletion. This is concrete query-endpoint exposure evidence,
not zero exposure and not proof about every hosting subsystem.

The developer/integration pages strongly invite use by applications and AI agents and expose public
credential-free packages/endpoints. The prior SEARCH-001-D register recorded a general Purili terms
page with personal/noncommercial wording. A current authoritative general service/result-use terms
page was not independently recovered during the 2026-11-01 refresh. The apparent policy history
therefore remains unresolved. Do not infer blanket distributed-client, redistribution or
model-context permission from technical integration documentation alone.

## Disposable Offline Wire Experiment

Implementation:

- `crates/talos-tools/tests/i294_purili_wire_characterization.rs`
- `crates/talos-tools/tests/fixtures/i294_purili_synthetic.json`

The test is compiled only into the `talos-tools` integration-test binary when the existing `network`
feature is enabled. It uses existing `serde`/`serde_json` and `reqwest::Url` facilities only. It does
not construct a client or issue a request. The fixture uses invented text and `example.invalid`
URLs. Field names follow the public documentation; values are not acquired provider data.

The experiment deliberately keeps production decisions open:

- integer widths and strict field rejection are test hypotheses, not public adapter guarantees;
- URL validation accepts only credential-free HTTP(S) result URLs and never fetches them;
- result limits are applied after URL filtering;
- advertised `total`/`totalPages` never control allocation or accepted-result count;
- unknown `source` values are retained rather than rejected or treated as native-route identity;
- unknown extra response/result fields are tolerated;
- already-received body size and normalized output count are bounded experiment parameters;
- 4xx and 5xx JSON error envelopes are distinguished from HTTP-200 empty search yield;
- redirects are classified as unexpected in this offline helper, but no runtime redirect policy is
  established by doing so.

These tests do not prove transport buffering, DNS/TLS, redirect handling, timeout/cancellation,
retry policy, endpoint safety, provider uptime, source truth, result quality or regional reachability.

## Qualification Rows

| Row | Required evidence | Current state |
|---|---|---|
| P-Q01 | Permitted anonymous valid-query request to documented `/api/search` | **Pending.** Published contract supports it; no response was acquired in this execution environment. |
| P-Q02 | Success envelope and SearchResult shape, optional metadata and unknown-field drift | **Partial offline evidence.** Independently authored fixture/parser only. |
| P-Q03 | Empty, malformed, missing/wrong-type and invalid-only outcomes | **Partial offline evidence.** Distinct offline outcomes; no deployed malformed/challenge capture. |
| P-Q04 | Unicode/plain-text normalization, wire order and provenance labels | **Partial offline evidence.** No renderer or relevance claim. |
| P-Q05 | Invalid/credential/non-HTTP URL filtering and body/output bounds | **Partial offline evidence.** No result fetch, transport or production numeric policy. |
| P-Q06 | HTTP 4xx/5xx versus 200-empty semantics and bounded retry/deadline behavior | **Partial offline status evidence only.** Transport/retry/deadline/cancellation remain pending. |
| P-Q07 | Current service/result-use/privacy conditions for distributed Talos use | **Pending.** Privacy is clearer; current general use/result terms remain unresolved. |
| P-Q08 | Endpoint/downstream independence and paired real-network regional measurements | **Pending.** No failure-domain or genuine mainland-China evidence. |

## Offline Cases

The disposable tests cover:

1. `crawled` and unknown source labels retained without invented backend identity;
2. Unicode/plain text and wire order preservation;
3. HTTP-200 empty, invalid-only and malformed-success distinction;
4. relative, credential-bearing and non-HTTP(S) URL rejection before output truncation;
5. required top-level/result fields and wrong-type detection;
6. optional correction/query-analysis metadata plus unknown-field tolerance;
7. huge/negative advertised counts not controlling allocation or result count;
8. documented 400/5xx JSON error classes distinct from 200-empty and unexpected redirect status;
9. explicit already-received-body and output-count bounds.

No captured error strings are needed for the experiment: error text is treated only as a shape
requirement and not retained in the normalized outcome.

## Relationship To Mwmbl And Independence

Purili is useful to SEARCH-001-D only if it provides a genuinely independent second native route,
not merely a different hostname. Its docs state core Search uses its own crawler and independent
index rather than a third-party search API. That is promising source evidence, but admission still
requires controlled endpoint/hosting observations and comparison against Mwmbl and the DDG/Bing
compatibility family.

Mwmbl's standard route has documented Wikipedia fan-out, so paired tests must preserve native hit
provenance and avoid counting reference-only output as independent generic search yield. Purili's
separate Wikipedia-derived infocard/image features are not part of the core `/api/search` contract
qualified here and must not be mixed into native-route credit.

## Remaining Admission Path

Before this route can count toward SEARCH-001-D acceptance:

1. reconcile current applicable Purili service/result-use terms;
2. acquire one permitted bounded anonymous valid-query observation from a network with working
   DNS/HTTPS, retaining only bounded metadata rather than query/result content;
3. define reviewed production parser/transport bounds only if a later E/F implementation is
   authorized;
4. compare actual endpoint/downstream failure domains against Mwmbl and DDG/Bing;
5. obtain paired controlled real-network observations including genuine mainland-China provenance;
6. make an explicit admission-or-block decision in D before E/F activation.

Until then, Purili remains a research candidate. SEARCH-001-D / I294 remains Active / Claimed,
#644 and #624 remain open, and E/F remain unauthorized.
