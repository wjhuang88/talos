# SEARCH-001-D Terms And Result-Use Checkpoint — 2026-11-01

> Status: Research / technical-governance evidence only; not legal advice and no provider is admitted.
> Owner: SEARCH-001-D / I294 / #644.
> Baseline: post-#829 `main` at `718e566e4ab847b994800036c7fe4dcc808cf37a`.

This checkpoint separates two questions that earlier evidence sometimes grouped together:

1. **interface / automation permission** — whether the provider publishes an interface intended for
   programmatic use by applications or agents; and
2. **result / content use** — what rights and restrictions apply to titles, snippets, stored page
   text, result sets or other material returned through that interface.

A public no-key API can have strong automation intent while result reuse remains restricted or
unclear. Conversely, an SDK licence governs the SDK code and does not by itself license the search
results returned by the service.

## Mwmbl

Current primary sources rechecked 2026-11-01:

- [Mwmbl API developer page](https://developer.mwmbl.org/)
- [Mwmbl API Terms and Conditions](https://developer.mwmbl.org/terms/), effective 2026-05-15
- [Mwmbl API Privacy Policy](https://developer.mwmbl.org/privacy/), effective 2026-05-15
- [Creative Commons FAQ](https://creativecommons.org/faq/) for the general meaning of an NC
  restriction; this is licence interpretation background, not Mwmbl-specific permission.

### Interface / automation permission

The current developer page explicitly documents an Anonymous plan with no API key or signup,
1,000 requests/month and a 1 request/second limit. The API terms likewise list an Anonymous plan,
but their access introduction also says API access requires registration and an API key. That
internal policy inconsistency remains a deployed-contract qualification item and must not be
silently resolved in Talos's favour.

Separately, the API terms' permitted-use section says the API may be used for lawful purposes,
including commercial applications, subject to the stated restrictions. This materially advances the
**API-use** side of M-Q07: the current API terms do not present programmatic/commercial use itself as
a prohibited category. It does not resolve anonymous-plan consistency or result-content rights.

The API privacy policy also materially advances query-exposure review: it says API search queries
are not logged or retained and request IP addresses are transient in memory rather than persisted.
This remains a published-policy statement rather than a runtime measurement.

### Result / content use

The same API terms explicitly say search results returned by the API are made available under
CC BY-NC-SA 4.0 and remain subject to rights held by the original content owners. This is distinct
from the terms' permission to **call** the API for commercial applications.

Creative Commons describes an NC restriction as depending on the purpose of the particular reuse,
not simply whether the user is a for-profit or nonprofit entity. Therefore Talos cannot safely
convert Mwmbl's broad API-call permission into a blanket claim that returned result material is
suitable for every default, hosted, enterprise or commercial Talos use. The exact application of
copyright, database rights, attribution, ShareAlike and original-source rights to a particular
Talos output is a legal question outside this research packet.

For SEARCH-001-D governance, that uncertainty is enough to keep Mwmbl's **unrestricted default
result-use** gate blocked. A route may remain a technical research candidate, but D must not state
that the current terms authorize arbitrary commercial redistribution or model-context reuse of
Mwmbl result material.

### Mwmbl disposition

| Sub-question | Disposition |
|---|---|
| Programmatic API use | **Advanced.** Current API terms expressly allow lawful use including commercial applications. |
| Anonymous/no-key contract | **Still inconsistent.** Developer page and plan table support it; access prose says registration/key required. Deployed observation remains needed. |
| Query/privacy policy | **Advanced.** Published API policy says queries are not retained and request IPs are not persisted. |
| Unrestricted result reuse | **Blocked / unresolved for default admission.** API terms attach CC BY-NC-SA 4.0 plus original-content-owner rights to returned search results. |

M-Q07 therefore advances but does not close.

## Purili

Current primary sources rechecked 2026-11-01:

- [Purili developer page](https://puri.li/developer)
- [Purili API documentation](https://puri.li/developer/documentation), marked last updated August
  2026
- [Purili integrations](https://puri.li/developer/integrations)
- [Purili TypeScript SDK guide](https://puri.li/developer/web-search)
- [Purili MCP guide](https://puri.li/developer/mcp)
- [Purili privacy policy](https://puri.li/privacy), marked last updated July 2026
- public `@purili/*` npm package metadata, including MIT-licensed integration packages

### Interface / automation permission

Purili's current official developer material is unusually explicit about programmatic integration.
It describes `/api/search` and `/api/context` as public, no-account/no-key APIs for applications,
AI agents, retrieval pipelines and RAG; the developer landing page says integrations can be shipped
without keys, quotas or billing setup; current examples show direct search, site-search and RAG
usage; and the hosted MCP endpoint is documented for arbitrary compatible MCP clients.

This is strong current evidence of **intended automated interface use**. It is stronger than merely
finding an undocumented public endpoint and materially advances the interface-use part of P-Q07.

Purili also publishes official integration packages whose package metadata is MIT-licensed. That
supports redistribution/use of the package code under its package licence, but the package licence
must not be treated as a licence for Purili's API service or for third-party page material returned
by Search/Context. Talos currently does not need to adopt those packages as dependencies to benefit
from this evidence.

### Current service terms and result / content use

The 2026-10-09 evidence register recorded a then-retrieved `https://puri.li/terms` page with
personal/noncommercial wording. During the 2026-11-01 refresh and this follow-up, a current
canonical Purili terms/use page could not be independently recovered from Purili's current public
site/search results. Current developer pages strongly invite application and agent integrations,
but they do not provide a separately identified contractual result-content licence in the material
recovered here.

The old restriction must therefore be treated as **historical evidence requiring reconciliation**,
not silently assumed to remain current and not silently declared superseded. Current integration
copy is relevant evidence of service intent, but marketing/developer documentation alone is not a
substitute for an authoritative terms record when the prior evidence conflicts with it.

Purili's current Search schema returns cleaned page titles and descriptions, and Context can return
stored summaries, extracted text and passages from indexed third-party pages. The documentation
calls indexed content untrusted source material and recommends preserving original source URLs for
citation. No current Purili-specific licence granting unrestricted reuse of that returned page
material was recovered in this checkpoint. Rights in source-page content therefore cannot be
inferred from the MIT licence of an SDK or from the API being free/no-key.

### Purili disposition

| Sub-question | Disposition |
|---|---|
| Intended automated/API/agent use | **Strongly advanced.** Current official docs explicitly target applications, agents and RAG and publish direct API/MCP/SDK integrations. |
| SDK/integration code licence | **Separate and clear for published packages.** Current package metadata identifies MIT licences; this says nothing by itself about API result rights. |
| Current canonical service/use terms | **Unresolved.** The prior personal/noncommercial `/terms` evidence was not independently recovered as a current authoritative page, and current integration docs do not explicitly reconcile it. |
| Result/page-content reuse | **Unresolved.** Search/Context return third-party-derived page material; no blanket Purili result-content licence was recovered. |

P-Q07 therefore advances on interface intent but does not close for default admission.

## Cross-provider admission effect

| Candidate | Interface / automation | Result / content use | D disposition |
|---|---|---|---|
| Mwmbl | Current API terms expressly permit lawful use including commercial applications, with an anonymous-access wording inconsistency | Current API terms explicitly attach CC BY-NC-SA 4.0 plus original-owner rights to returned results | **Not admitted.** Result-use/default-distribution gate remains explicit. |
| Purili | Current official docs strongly and repeatedly invite public application/agent/RAG/API/MCP use | No current canonical service terms or blanket result-content licence was recovered; historical personal/noncommercial evidence conflicts with current integration intent | **Not admitted.** Current terms/result-use reconciliation remains explicit. |

This checkpoint does **not** conclude that every Talos use of either provider would infringe rights or
breach terms. It records a narrower engineering-governance rule: SEARCH-001-D cannot authorize an
unrestricted native default route while the provider's own current evidence leaves material
result-use restrictions or conflicting/absent terms unresolved.

## What would close the terms gate

For Mwmbl, D needs an evidence-backed disposition for how the API's CC BY-NC-SA result clause applies
to the exact Talos route/output contract, or a provider/rightsholder clarification that removes the
relevant default-use ambiguity. Any narrower technical output contract proposed to avoid licensed
material would require its own review; this document does not pre-authorize one.

For Purili, D needs a current authoritative service/use record that reconciles the historical
personal/noncommercial evidence with the current public app/agent/RAG integration material and
states enough about result/content use for the intended Talos route. Absence of a discoverable terms
page is not treated as permission.

Live endpoint success, failure-domain independence and genuine regional observations remain separate
acceptance gates. Until all required gates close, SEARCH-001-D / I294 stays Active / Claimed,
#644/#624 remain open and SEARCH-001-E/F remain unauthorized.
