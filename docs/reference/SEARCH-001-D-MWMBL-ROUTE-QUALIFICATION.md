# SEARCH-001-D: Mwmbl Standard Route Qualification

> Status: Partial source and query-free diagnostic evidence; no backend admission.
> Owner: I294 / SEARCH-001-D / #644; effective claim #665.
> Talos baseline: `fecc9583992ccf2c0678b58e902f4a3bdcf19d2b`.
> Inspected upstream: `ff482e225d99c694abbd3337a1b1be93b8c95dd8`.

This packet follows #686 with a concrete standard-route trace and two bounded, query-free
observations. It does not introduce an adapter, production routing, credentials or a startup
probe. No search query, result response, account, paid call or provider contact was acquired.

## Observed Public Interface — 2026-10-09

Each URL below received one GET from this execution environment, without query parameters,
credentials, cookies, client retries or redirect following. TLS verification stayed enabled;
request timeout was 25 seconds. These are explicit research diagnostics, not Talos runtime I/O.
No client IP or inferred geographic label was retained. Timestamps are UTC response-file completion
times. Response-body hashes identify the observed bytes; they are not deployment commit hashes.

| Observation | Time | Outcome | Limit |
|---|---|---|---|
| [Public v2 OpenAPI](https://mwmbl.org/api/v2/openapi.json) | 07:00:26Z | HTTP 200, application/json; 21,235 bytes; OpenAPI 3.1.0, API version 2.0.0 | Public contract document, not search execution or a fixed deployed source revision |
| [Standard route, missing q](https://mwmbl.org/api/v2/search/) | 07:01:12Z | HTTP 422, application/json; 81 bytes; validation reports missing query parameter q | Reached input validation; no search results, valid-query success, anonymous execution or regional availability proved |

OpenAPI body SHA-256: `acb35e1f0ccc4c1c8afd1cfd8fd8ab6164cf4d2c7b1eeeaf6cf990d891e65022`.
Missing-input body SHA-256: `7fd5c8e1fc9136f0b56d8b2563265d325edca4a1fec71be75c4cba7daf05db04`.
The missing-input body contains only a validation envelope, not a retained search fixture.
Raw OpenAPI descriptions/examples are not copied into the repository.

The public document advertises GET `/api/v2/search/`, required string `q`, `SearchResponse` and
optional key-based usage tracking. Its schema structurally agrees with the inspected source:
`query`, `number_of_results` and `results` are required; usage fields accept integer or null;
each hit has URL, title, content, highlight arrays, engine and numeric score. This agreement is
interface evidence, not proof the server runs the pinned commit. The document calls the ranker
heuristic while source setup selects LTR; that wording cannot establish deployed ranking.

## Standard Source Call Chain

All source links below use the fixed upstream commit. No upstream code or tests were executed.

| Boundary | Primary evidence | Finding / implication |
|---|---|---|
| Endpoint | [search.py](https://github.com/mwmbl/mwmbl/blob/ff482e225d99c694abbd3337a1b1be93b8c95dd8/mwmbl/tinysearchengine/search.py) | Standard v2 invokes ranker.search(q, []) and formats that list. No client-selectable index-only flag is exposed on this route. Optional key validation is separate; missing-q validation does not prove that later branch executes anonymously. |
| Setup | [search_setup.py](https://github.com/mwmbl/mwmbl/blob/ff482e225d99c694abbd3337a1b1be93b8c95dd8/mwmbl/search_setup.py) | Standard route uses MMR over LTR with Wikipedia enabled. Combined search has a separate ranker; do not classify every standard request as combined/EUSP/Jev processing. |
| Wrapper | [mmr_rank.py](https://github.com/mwmbl/mwmbl/blob/ff482e225d99c694abbd3337a1b1be93b8c95dd8/mwmbl/tinysearchengine/mmr_rank.py) | MMR forwards the default external-search flag to its wrapped ranker and reranks results. It does not disable Wikipedia. |
| Retrieval and ranking | [rank.py](https://github.com/mwmbl/mwmbl/blob/ff482e225d99c694abbd3337a1b1be93b8c95dd8/mwmbl/tinysearchengine/rank.py), [ltr_rank.py](https://github.com/mwmbl/mwmbl/blob/ff482e225d99c694abbd3337a1b1be93b8c95dd8/mwmbl/tinysearchengine/ltr_rank.py) | Ranker retrieves local index candidates, then external candidates, then ranks together. LTR external_search calls get_wiki_results. This path can return externally sourced reference results alongside index/curated results. |
| Wikipedia helper | [rank.py](https://github.com/mwmbl/mwmbl/blob/ff482e225d99c694abbd3337a1b1be93b8c95dd8/mwmbl/tinysearchengine/rank.py) | Helper truncates the external query to 100 characters, checks cache/circuit, then calls English Wikipedia search API with a 5-second request timeout. Retry total is four with server-delay handling; circuit cooldown is 60 seconds. These upstream settings are not a 5-second total Talos budget or a transport-cancellation guarantee. |
| Upstream cache | [external_cache.py](https://github.com/mwmbl/mwmbl/blob/ff482e225d99c694abbd3337a1b1be93b8c95dd8/mwmbl/indexer/external_cache.py) | When enabled, source-separated cached results use a keyed normalized-query digest and retain result content with freshness metadata. A cache hit can avoid a live Wikipedia call; hashed keys do not make retained result content anonymous by definition. Deployed settings remain unknown. |
| Provenance | [format.py](https://github.com/mwmbl/mwmbl/blob/ff482e225d99c694abbd3337a1b1be93b8c95dd8/mwmbl/format.py), [indexer.py](https://github.com/mwmbl/mwmbl/blob/ff482e225d99c694abbd3337a1b1be93b8c95dd8/mwmbl/tinysearchengine/indexer.py) | Formatter prefers explicit external source, otherwise maps document state. Wire labels include mwmbl, wikipedia, google, user and eusp. Google/user labels can identify stored curated provenance; a label alone does not prove a live query to that provider. v2 score is reciprocal result position, not comparable cross-backend relevance. |

The inspected standard call chain reaches local ranking plus Wikipedia, not the separate combined
ranker. This bounds a source claim only: middleware, deployment configuration, source drift and
published privacy disclosures still require qualification. It does not prove all hosted query
processing is confined to these calls. Wikipedia-only yield cannot count as an independently
qualified generic web route. Preserve backend identity separately from per-hit provenance.

## Remaining Qualification And Fixture Rows

These rows describe future evidence, not executed tests or an accepted adapter contract.

| Row | Required evidence | Qualification condition |
|---|---|---|
| M-Q01 | Permitted valid anonymous standard request | Normalized usable general-web results, null usage semantics, no credentials; a schema GET/422 cannot substitute |
| M-Q02 | Index, Wikipedia-only, mixed and curated provenance fixtures | Preserve source labels; distinguish index route identity from hit origins; no second-native credit for reference-only yield |
| M-Q03 | Unknown engine, missing required fields, wrong types, empty list and count mismatch | Record drift/invalid-result outcome; do not trust advertised count as the number of accepted results |
| M-Q04 | Unicode, highlights and positional score | Plain title/content normalization; no HTML interpretation or cross-provider score comparison |
| M-Q05 | Invalid/credential-bearing/non-HTTP URLs and payload limits | Offline validation and bounded output; never fetch result URLs to validate them |
| M-Q06 | Transport timeout/status, schema-valid empty response and external-source failure | Distinct outcomes; caller deadline bounds the entire request; do not assume upstream retries fit a per-request timeout |
| M-Q07 | Published service/data conditions and hosted query-processing evidence | Resolve distributed client/model-context and fixture reuse; reconcile privacy policy with actual standard route without assuming bespoke outreach is mandatory |
| M-Q08 | Paired controlled real-network measurements and failure-domain comparison | Verified environment provenance including genuine mainland China; client endpoint success does not prove independent downstreams or global coverage |

Synthetic fixtures can be authored independently from documented field shapes with invented text
and example-domain URLs. They prove parser behavior only after an admitted parser exists; do not
publish them as captured provider results. No bulk/raw index acquisition, autocomplete workaround,
paid combined route, relay or Wikipedia-only generic fallback is proposed by this packet.

## Disposition

The public host now has a dated schema and missing-input observation. Valid-query anonymous
execution, deployed fan-out, terms/result-use applicability, parser fixtures and independent
regional acceptance remain pending. The [privacy policy](https://mwmbl.org/privacy) describes
external query/result processing more broadly than this pinned standard trace; the
[service terms](https://mwmbl.org/terms) remain a separate source of use conditions. Neither a
source trace nor a query-free diagnostic resolves those questions. Mwmbl remains a research
candidate, not an admitted native backend. D/I294 stays Active / Claimed, E/F gated, #624 open.
