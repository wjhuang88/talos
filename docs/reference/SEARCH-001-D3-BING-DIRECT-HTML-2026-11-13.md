# SEARCH-001-D3: Bing Direct HTML, Endpoint and Rights Evidence — 2026-11-13

> Owner: [SEARCH-001-D3 / #937](../backlog/active/SEARCH-001-D3-bing-direct-html-feasibility.md).
> **Decision: OFFLINE TECHNICAL RESEARCH POSSIBLE; NEW DEFAULT/AUTOMATED
> LIVE PROVIDER ADMISSION BLOCKED.** `www.bing.com` and `bing.cn` are
> hypotheses, not observed canonical endpoint results or regional reachability.
> No live automated Bing SERP retrieval was performed for this packet.

## 1. Evidence strata and current status

| ID | Claim | Evidence | Disposition |
|---|---|---|---|
| B-01 | Bing SERPs are server-rendered and HTML is delivered in chunks | [Microsoft Bing engineering blog, 2022-08-25](https://blogs.bing.com/search-quality-insights/2022/8/Fast-Front-End-Performance-for-Microsoft-Bing/) | **Supported historical architecture**, not a live 2026 HTML contract |
| B-02 | Talos already prepares a Bing query | [Fixed compatibility requests](SEARCH-001-C-COMPATIBILITY-REQUESTS.md): `GET https://cn.bing.com/search`, `q` and `mkt=zh-CN`; [chain](../backlog/active/SEARCH-001-C-search-router.md) | **PASS code truth**, already-owned compatibility, not an admitted new provider |
| B-03 | Talos already parses bounded Bing SERP HTML | `crates/talos-tools/src/search_backend/bing_parser.rs`: `li.b_algo` / `h2 a` / `.b_caption p`, typed empty/challenge/status/rate errors, bounded candidates | **PASS code truth**, current parser only; no demonstrated international DOM parity |
| B-04 | `https://www.bing.com/search` can be a separately selected Talos endpoint | No approved prepared destination, no independent authorized live acquisition in this packet | **UNKNOWN**; do not add to network manifest or URL allowlist |
| B-05 | Bare `bing.cn` resolves or redirects to a stable canonical SERP/query authority | User observation, not a recorded controlled redirect trace | **UNKNOWN**; distinguish `bing.cn` from existing `cn.bing.com` |
| B-06 | `www.bing.com` and `cn.bing.com` are independent web-search providers | Both are Bing-branded entry surfaces; the code's existing Bing source identity is embedded in compound compatibility | **NO**; never count as two native independent routes |
| B-07 | A direct request can yield licensed/default-distributable results for Talos | [Microsoft Services Agreement](https://www.microsoft.com/en/servicesagreement), published 2026-07-30, effective 2026-09-30 | **BLOCKED / NOT ESTABLISHED**; new product-building and redistribution rights require independent clearance |
| B-08 | The retired Bing Search API supplies a supported anonymous direct-search license | [Microsoft retirement notice](https://learn.microsoft.com/lifecycle/announcements/bing-search-api-retirement): APIs retired 2025-08-11 | **NO** |
| B-09 | This endpoint works reliably on genuine mainland-China and another independent network | No live region-tagged observation for D3 | **UNKNOWN / NOT QUALIFIED** |
| B-10 | BrowserPageRecord selected links are equivalent to organic search results | [Browser feasibility #891](https://github.com/wjhuang88/talos/issues/891): negative counterexample; real browser #452 not complete | **NO**; a dedicated HTML parser is a different pathway |

### Rights are two separate gates

Microsoft Services Agreement section **Service-Specific Terms → Bing and MSN**
states Bing/MSN materials are for noncommercial personal use and that other uses
such as copying, redistribution or building a product require authorization by
Microsoft/rightsholders or applicable law. Section **Code of Conduct** also
disallows circumvention of usage/access restrictions, including impermissible
scraping. The agreement was published **2026-07-30** and is effective
**2026-09-30**, so this is the currently applicable source checkpoint.

This establishes **neither** that a one-off manual search is prohibited **nor**
that Talos has permission to automate a distributed default search client.
A proper authorization review must separate:

1. **Automated service access** (query-bearing requests, volume, headers,
   rate-limits, interactive/browser vs embedded agent).
2. **Output use** (titles, snippets, result URLs, context provided to models,
   display, attribution, retention/cache and distribution).

No permission is inferred from a browser-accessible endpoint, HTTP 200,
robots.txt or a library already containing a Bing parser. Provider clarification
or a scoped legal determination is needed before new production admission.
No legal compliance conclusion is made by this technical packet.

## 2. Protected code path and invariants

| Area | Talos source | Baseline / do not change in D3.1 |
|---|---|---|
| Query-bearing authority | `docs/reference/SEARCH-001-C-COMPATIBILITY-REQUESTS.md` | `https://cn.bing.com/search?q=<encoded>&mkt=zh-CN` only; a new host is a **new request authority** |
| Compatibility order | `search_backend/transport/compatibility/chain.rs` | Lite → Instant Answer → HTML → Bing CN; no new source or priority |
| HTML parser | `search_backend/bing_parser.rs` | `li.b_algo`, first `h2 a`, first `.b_caption p`, bounded text/body/candidates; do not conflate ads with organic |
| Legacy public tool | `search_backend/legacy_owned.rs` | Separate standalone `WebSearchTool` compatibility, not a protected prepared fallback |
| Safety | ADR-085 / ADR-090 / NET-001 | Prepared authority and Network grant before query, bounded cancellation, SSRF/redirect admission, privacy/proxy, retry/deadline budget |
| Ownership | #901, #644, #848, #891 | Retired dependency complete; prior provider admission blocked; MCP and Browser are separate owners |

A new `www.bing.com` request must be declared in the **same invocation's
prepared endpoint manifest before permission is granted**, including redirects,
possible query recipients and explicit proxy behavior. It must not be assembled
inside a parser or used as a silent fallback from the exported standalone tool.

## 3. Synthetic-only test matrix for a later *claimed* implementation

All fixture HTML is generated locally and manually; no genuine Bing response
body or live query must be collected by this intake. Fixture row names are
acceptance candidates, **not completed executable tests**.

| Fixture ID | Input | Required assertion |
|---|---|---|
| BF-01 | HTML with one `li.b_algo` organic entry containing `h2 a` and `.b_caption p` | One bounded title/snippet/URL, correct source; no inherited browser markup metadata |
| BF-02 | Ads-only HTML using non-organic ad container, including a visually similar `h2 a` | No fabricated organic hit; distinguish valid zero from parser/structure failure |
| BF-03 | `200` challenge or captcha form with no organic entries | Typed challenge, not false valid empty or health success |
| BF-04 | `403`, `429`, `503` and HTML with empty body | Typed status/rate/temporary failure, no retry storm or query leak |
| BF-05 | Malformed UTF-8, BOM and large nested/overlong fields | Existing deterministic encoding/body/field bounds and closed errors |
| BF-06 | `200` with alternate locale/market text and changed result DOM selectors | Detection of drift and false-positives; no claim of actual Bing intl parity |
| BF-07 | Redirect `www.bing.com`→`cn.bing.com`; `bing.cn`→unknown; external `evil.invalid` | Explicitly prepared authority and per-hop admission required; unknown/cross-origin fail closed |
| BF-08 | URL variants with `q`, `mkt`, encoded query, double-encoding and control chars | Preserve one encoding, parameter ordering and query secrecy; never log raw query |
| BF-09 | 200 root HEAD/GET but query path returns 403 or challenge | Root reachability alone **cannot** mark search healthy |
| BF-10 | 200 valid organic entries vs 200 empty results vs 200 ads-only | Health samples record *valid query outcome*, not status alone; do not imply distinct index |
| BF-11 | Two endpoints with alternating observed failures and changed proxy | Bounded cool-down/recovery concept without GeoIP/country routing, stale state or global new retry layer |
| BF-12 | Cancellation during redirect/body/parse and permission denial before send | No late outbound query, no permission bypass, bounded deadline and cleanup |
| BF-13 | Existing original Bing CN-only fixture path | No source order, API/output/schema/permission regression from an **offline-only** characterization |

Any fixture that actually compares live Bing HTML must be separately
authorized; synthetic markup alone cannot establish real-world parser quality.

## 4. Future conditional endpoint design — NOT APPROVED

```text
SearchRouter (unchanged public web_search)
  └─ PreparedCompatibilitySearch (existing Network grant, budget, URI authority)
        └─ Bing logical family (candidate only)
             ├─ variant: cn.bing.com/search   [already present]
             └─ variant: www.bing.com/search  [unverified, not admitted]
                     ↓
              owned bounded Bing HTML parser / result admission
```

- Maintain a *single logical Bing provider family* and optional endpoint-variant
  observations. Endpoint health never substitutes for independent-provider
  admission, legal right or end-user Network permission.
- Future selection must rely on observed query-specific valid organic yield,
  redirect/challenge/429 behavior, latency and recent failures; **not**
  `GeoIP`, country-locale assumptions or an arbitrary `bing.cn` alias.
- A prior query-free HEAD/GET root availability is useful only as weak
  connectivity evidence. Mark real search health based on permitted query
  outcomes, without startup probes or covert sample queries.
- Bound attempts by the existing invocation budget and cancellation rules.
  Every candidate host/redirect must be declared before a request and subject
  to per-hop SSRF URL admission and privacy. No general transport changes.
- Retain current protected and legacy fallback behavior as rollback until
  separately governed real acceptance; do not count Bing and DDG as two
  independently sourced providers merely because the destinations differ.

## 5. Future authorized network-evidence plan

**Prerequisites**: scoped legal/product permission to use and retain Bing
search responses for this exact Talos design, selected D3 work slice + effective
claim, acceptance of new prepared-authority manifest and privacy controls.

Only then acquire a bounded record from a genuinely operated mainland-China
network and a separately identified external comparator. Each capture would
record local date, operator-attested network provenance/coarse region, exact
requested host and effective redirect *host path* (redact query and cookies),
HTTP status, observed challenge, parsed eligible organic count, response timing,
TLS/DNS/network error classification and result age. No IP-based inference,
VPN exit or foreign cloud host substitutes for mainland-China evidence.
Minimize retained content, respect rate/policy, no scraping around challenges,
no telemetry of query text. Record UNKNOWN if environment unavailable.

## 6. Independent review questions and decision

1. Which exact Microsoft authorization/right applies to a **distributed
   zero-key, automatically selected, query-bearing Talos search adapter** and
   model-visible titles/snippets? Without affirmative scope, **BLOCKED**.
2. Are `www.bing.com` and `cn.bing.com` distinct response templates or redirects,
   and does `bing.cn` resolve to either? **UNKNOWN** without approved observation.
3. Can existing parser reliably distinguish organic results from ads/empty and
   challenges on both templates without violating rights? **UNKNOWN**.
4. Would any future expanded host manifest preserve ADR-090 endpoint and
   redirect permission, proxy, cancellation, retry and query disclosure?
   Requires **separate design review**, not authorized by D3.1.
5. Can controlled mainland-China and comparison network evidence be obtained
   without pretending that CI/geo-IP proves access? **UNKNOWN**.

**D3.1 conclusion**: existing Talos HTML parser + Bing SSR make an
offline-design follow-up useful, but *no additional endpoint or provider
is admitted*. New production and live automated acquisitions remain
**BLOCKED** pending appropriate use rights and a new effective claim.
D/#644 stays closed BLOCKED/ZERO-ADMISSION; D2/#848 remains unclaimed;
E/F/G/I and parent #624 are unchanged.

## 7. Rights and next-action recheck — 2026-11-15

**Provenance:** This is a current review of *published Microsoft
documentation* and accepted Talos GitHub source, **not** a live Bing query,
written Microsoft approval, new commercial license or Chinese-network test.
The 2026-11-13 original source matrix and its BLOCKED/UNKNOWN ratings stand.

| Evidence surface | Official source | Rechecked decision |
|---|---|---|
| Bing/MSN user content and product reuse | [Microsoft Services Agreement](https://www.microsoft.com/en-us/servicesagreement), published 2026-07-30, effective 2026-09-30, Bing and MSN Materials | **BLOCKED for unapproved product use**. The personal/noncommercial grant does not establish Talos's distributed zero-key embedded automated search or response reuse rights. Permission by Microsoft/rightsholders or applicable law must be assessed independently. |
| Historic paid Bing Search API | [Microsoft 2025-08-11 retirement notice](https://learn.microsoft.com/lifecycle/announcements/bing-search-api-retirement) | **NOT an available free default alternative**. Former paid API display and LLM-specific terms, where still posted, attach to those services; they are not a raw SERP scraping grant. |
| Current Microsoft-managed Foundry Web Search/Grounding | [Foundry web grounding overview](https://learn.microsoft.com/en-us/azure/foundry/agents/how-to/tools/web-overview), [Foundry Grounding with Bing](https://learn.microsoft.com/en-us/azure/foundry/agents/how-to/tools/bing-tools), [general Microsoft Azure Product Terms (MCA; no Grounding-specific clause established from this page)](https://www.microsoft.com/licensing/terms/en-US/product/MicrosoftAzure/MCA) and **[Grounding enterprise-integration terms](https://www.microsoft.com/en-us/bing/apis/grounding-legal-enterprise)** | Foundry is an **enterprise integration** subject to the linked *enterprise* terms and customer agreement. [The separate Approved Microsoft Application terms](https://www.microsoft.com/en-us/bing/apis/grounding-legal) expressly **exclude Azure Foundry/enterprise integrations**; do not apply those terms to Foundry. This is a distinct managed/billed output and display contract, **not** a grant for anonymous consumer SERP HTML; its citations/summaries are not `li.b_algo` transport fixtures. Any optional opt-in integration requires separate review. |
| Microsoft Web IQ | [Web IQ official FAQ](https://learn.microsoft.com/en-us/web-iq/resources/faq/) | Distinct commercial/search data product with restrictions on cache, archival and output; no automatic entitlement to use consumer Bing HTML. Any future consideration needs a separate paid/service/privacy review. |
| Talos actual change | [D3-T2/#947](https://github.com/wjhuang88/talos/issues/947), [source PR #949](https://github.com/wjhuang88/talos/pull/949) and [owner-first #950](https://github.com/wjhuang88/talos/pull/950) | **Complete / Closed**, source Completion Commit `5b2fa4358b83698ca4c0e517ce24a6dbba021d99`, docs closeout `6c5505abdc8d964ec39c01d221e9e3297a3458cf`. Exact `b_ad` ancestor-only filtering and synthetic tests; NO real Bing DOM validation or host admission. |

### Supplier/rightsholder clarification request — pre-implementation gate

A documented affirmative answer needs to explicitly cover **both service
access and output rights** for Talos's contemplated use, rather than treating
a working browser page or HTTP 200 as authorization:

1. Does Microsoft (or applicable rights holder) authorize end-user initiated
   **automatic** queries made locally by a redistributed, uncredentialed
   open-source Talos agent, as opposed to individual browser searches or
   purchased Foundry tools? Clarify `cn.bing.com/search`,
   `www.bing.com/search`, and the **unverified** `bing.cn` alias separately.
2. Which access method, identity, agreement, rate/volume and distribution
   conditions apply? Are automated HTML SERP requests permitted at all, or
   must access use a documented, credentialed service?
3. May Talos extract/display titles, result URLs and snippets; pass snippets
   to a model for one user query; retain query/session traces or results; and
   display attribution? Which per-user terms or rights apply?
4. What geographic/locale and redirect restrictions, retention periods,
   attribution and user-notice requirements are binding? Is a written
   product-specific grant required and who can grant it?

**GO criteria:** written applicable rights (including output use); an
effective separately selected Issue/owner/iteration claim; ADR-090 prepared
endpoint + redirect authority review and Network/privacy/budget acceptance;
then bounded operator-attested mainland-China and separate comparator
**query-level** observations, with no raw query in logs and no CAPTCHA
workarounds. **Otherwise BLOCKED / no new host**. Do not infer legal
authorization from official commercial grounding product docs.

D3/#937 remains **OPEN / Intake / Unclaimed**; Epic/#624 remains OPEN.
#644 stays historical **Complete / Closed with BLOCKED / ZERO-ADMISSION**.
E/F/G/I and independent-provider counting remain gated. This evidence
recheck makes **no** code/CI/network permission/default changes.
