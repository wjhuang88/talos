# SEARCH-001-D Failure-Domain Checkpoint — 2026-11-01

> Status: Research evidence only; no provider is admitted by this document.
> Owner: SEARCH-001-D / I294 / #644.
> Baseline: post-#828 `main` at `3ccce7eac00cca65e31750211001fa888c3530a3`.

This checkpoint separates several meanings of “independent” that must not be collapsed into one
claim. Different domains, projects, implementations, or indexes are useful evidence, but none alone
proves that two production routes have independent runtime failure domains or regional reachability.

## Independence dimensions

| Dimension | What must be shown |
|---|---|
| Project / implementation | Distinct implementation and operating project rather than a wrapper around the same provider. |
| Index / data plane | Generic web results can originate from independently built index data. |
| Query-time upstream | A route does not silently depend on the same third-party search backend for the credited native result. |
| Hosting / network | DNS, edge, compute, storage and other runtime dependencies are sufficiently independent for the intended fallback claim. |
| Regional reachability | Controlled observations show both routes can actually be reached from the relevant network/region. |

Only the first three dimensions can be materially advanced from current source and published
service documentation. Hosting/network independence and regional reachability remain live-evidence
gates.

## Mwmbl

Sources rechecked 2026-11-01:

- <https://github.com/mwmbl/mwmbl>
- <https://book.mwmbl.org/page/architecture/>
- pinned standard-ranker source at
  <https://github.com/mwmbl/mwmbl/blob/86376eb363fb458f660223cb1111db8ea8f8bf6a/mwmbl/search_setup.py>
- accepted provider refresh in
  [SEARCH-001-D Provider Evidence Refresh — 2026-11-01](SEARCH-001-D-PROVIDER-REFRESH-2026-11-01.md)

Mwmbl is an open-source search project with its own centralized index; crawling is distributed among
community-run crawlers. This is strong project/implementation and index-origin evidence, not hosted
failure-domain proof.

The pinned standard-search setup constructs `LTRRanker(..., include_wiki=True)` inside the standard
ranker. Therefore standard Mwmbl responses can include Wikipedia-origin results alongside Mwmbl
index results. A successful response only earns native-route credit when provenance demonstrates
usable Mwmbl-origin generic web results. Wikipedia/reference yield is not a second independent native
web route and also introduces a query-time dependency that can fail separately from Mwmbl's own
index.

Published architecture material distinguishes the API/front-end domains and describes Mwmbl's own
service architecture, but this checkpoint does not have authoritative current production evidence
for DNS/edge/compute/storage provider topology. Historical deployment instructions are useful
implementation context, not proof of the current hosted failure domain. M-Q08 hosting/network and
regional acceptance therefore remain pending.

## Purili

Sources rechecked 2026-11-01:

- <https://puri.li/developer/documentation>
- <https://puri.li/developer>
- <https://puri.li/why>
- <https://puri.li/privacy>
- <https://puri.li/stats>

Purili's current public documentation says core Search results come from its own crawler-built index
and that no third-party search API touches the core web query. The Search API documents `source` as
normally `crawled`, while the developer pages expose the same index directly to applications, AI
agents and RAG without credentials. This is strong published project/index/query-path evidence for
an independent core Search route.

Other Purili features must not be confused with the core Search failure domain. The documented
Images endpoint uses Wikimedia Commons and infocard/entity features can use Wikipedia-derived data;
those endpoints do not establish independence for `/api/search`, nor should their provenance be
credited as Purili native web-index yield.

Purili also describes dedicated server infrastructure, no third-party search API for core Search,
and no request-logging CDN. Those are useful operator assertions, but this checkpoint has no
independent current evidence for the actual DNS/edge/compute/storage provider topology, ASN,
physical placement, or correlated provider dependencies. P-Q08 hosting/network and regional
acceptance therefore remain pending.

The public privacy pages currently disagree on access-log lifetime: `/why` says IP logs rotate after
24 hours, while the formal `/privacy` policy says standard IP/path/timestamp access logs may be kept
up to seven days. Evidence handling must use the more conservative seven-day bound until the
published material converges.

## Cross-provider disposition

| Question | Current disposition |
|---|---|
| Distinct projects / implementations? | **Supported.** Mwmbl and Purili are materially different search projects and implementations. |
| Independently built generic web indexes? | **Supported with provenance caveat.** Both describe their own indexes; Mwmbl native credit must exclude Wikipedia/reference yield. |
| Same commercial search backend? | **No evidence of one for the credited core routes.** Purili says core Search uses no third-party search API; Mwmbl's native index is its own, but its standard route can fan out to Wikipedia. |
| Independent hosted failure domains? | **Not proved.** Different domains, codebases and operator descriptions are insufficient. Current production topology/correlation evidence is missing. |
| Independent regional availability, including mainland China? | **Not proved.** Requires paired controlled observations with genuine network provenance. |

The useful conclusion is therefore narrower than “the providers are independent”: the two
candidates have distinct implementation and native index origins, and no shared commercial search
backend is currently identified for the credited core results. Runtime hosting/network independence
and real regional independence remain open acceptance gates.

## Remaining evidence needed

1. Run the merged #827 controlled observation harness from permitted networks with working DNS/HTTPS,
   preserving only bounded metadata and result provenance.
2. Demonstrate native-origin useful yield for both routes; do not count Mwmbl Wikipedia/reference
   results or Purili auxiliary external-source endpoints as independent generic web routes.
3. Record current DNS/edge/compute/storage evidence only where it is authoritative and reproducible;
   do not infer failure-domain independence from hostnames or ASN snapshots alone.
4. Obtain paired genuine mainland-China and other target-region observations before any regional
   fallback claim.
5. Keep service/result-use conditions as a separate admission gate. Purili's strong public API/agent
   integration intent still does not by itself establish all distributed-client or result-use rights.

Until those steps close, SEARCH-001-D / I294 remains Active / Claimed and incomplete; #644/#624
remain open and SEARCH-001-E/F remain unauthorized.
