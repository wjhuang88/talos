# SEARCH-001-D2 Hosted No-Key MCP Search Triage — 2026-11-04

> Research-only evidence packet for [SEARCH-001-D2 / #848](https://github.com/wjhuang88/talos/issues/848).
> **Outcome: BOTH ROUTES BLOCKED / NOT ADMITTED.** This is a dated *documentary triage*,
> not a live MCP handshake, valid-query observation, provider authorization, legal opinion,
> independent failure-domain proof, or mainland-China reachability test.
> The earlier #644/I294 **BLOCKED / ZERO-ADMISSION** disposition remains effective.
> Base architecture: ADR-072 / ADR-085 / ADR-090; SEARCH-001-C router and H/#832 explicit config.

## 1. Why this specific follow-up

SEARCH-001-D examined Mwmbl/Purili and did not admit either. Two more official
AI-agent-facing, hosted, **keyless** routes may reduce native HTML parser maintenance:

- **Exa** officially advertises `https://mcp.exa.ai/mcp`, keyless hosted MCP with
  default `web_search_exa`/`web_fetch_exa` and limited anonymous usage.
- **Parallel Search** officially advertises `https://search.parallel.ai/mcp`, Streamable HTTP
  without a key/account and tools `web_search`/`web_fetch`. Its free service is presented for
  personal/hobby agents; paid/keyed service is recommended for higher limits or production at scale.

**Important:** An unauthenticated endpoint is not automatically a permission for Talos to distribute
a preselected *default* third-party service, nor proof of a stable capacity/SLA. A user adding a
remote MCP server voluntarily is not the same integration posture as built-in Auto routing.

## 2. Source evidence (verified 2026-11-04; publication and runtime may change)

| ID | Provider | Official / primary source | Fact supported / boundary |
|---|---|---|---|
| D2-01 | Exa | https://exa.ai/mcp | Hosted endpoint and no-key setup are published. |
| D2-02 | Exa | https://github.com/exa-labs/exa-mcp-server/blob/main/README.md | `web_search_exa`, `web_fetch_exa`, auth optional, anonymous throttling; upstream README is mutable. |
| D2-03 | Exa | https://github.com/exa-labs/exa-mcp-server/blob/main/api/mcp.ts | IP-based free rate limiting is implemented in public code; example default values are deployment-configurable and **not guaranteed hosted quotas**. |
| D2-04 | Exa | https://exa.ai/assets/Exa_Labs_Terms_of_Service.pdf | §1.1 constrains API use to documentation; §4.2 covers content/output redistribution, resale, unauthorized extraction and service restrictions; applicability/exceptions need verification for shipped default client. |
| D2-05 | Exa | https://exa.ai/pricing | Pricing and free account credits differ from anonymous keyless MCP terms; **do not** assume credits, account quotas or commercial rights transfer to the anonymous endpoint. |
| D2-06 | Parallel | https://parallel.ai/mcp | Current hosted no-key endpoint, default tools, free but throttled access; production use and higher limits reference keyed access. |
| D2-07 | Parallel | https://parallel.ai/blog/free-web-search-mcp | 2026-04-23 announcement: Streamable HTTP, `web_search`/`web_fetch`; free for agents, limits targeted at personal/hobby use and paid recommendation for production scale. |
| D2-08 | Parallel | https://parallel.ai/customer-terms | Customer terms §2 permit *customer* API incorporation into *customer applications* under their agreement, with restrictions on output handling. **Not evidence that anonymous MCP has identical rights.** |
| D2-09 | Parallel | https://parallel.ai/acceptable-use-policy | AUP expressly covers agentic/agent use, abuse/rate circumvention, and other limitations. |
| D2-10 | Parallel | https://parallel.ai/terms-of-service | Website terms, not sufficient to resolve which specific terms govern anonymous MCP. |

No source above proves legal clearance for silently preconfigured third-party default search.
Conflicts/ambiguity must be resolved via applicable current developer terms or written provider
clarification before an E/F candidate can be marked admitted. This is a technical risk flag, not
legal advice.

## 3. Provider qualification matrix

| Gate | Exa | Parallel | Admission implication |
|---|---|---|---|
| Official hosted URL | DOCUMENTED | DOCUMENTED | Candidate only |
| Public no-key access | DOCUMENTED / RATE-LIMITED | DOCUMENTED / RATE-LIMITED | Candidate only |
| Streamable HTTP request/response shape | DOCUMENTED; live unobserved | DOCUMENTED; live unobserved | PENDING |
| Permission to preconfigure a distributed default Auto backend | UNKNOWN | UNKNOWN | **BLOCKING** |
| Terms governing output snippets, attribution, LLM-context use, retention | AMBIGUOUS; §4.2 relevant | AMBIGUOUS; customer versus anonymous grant unresolved | **BLOCKING** |
| Anonymity, abuse controls, shared NAT limits / throttling semantics | Reported; quotas not verified | Reported; quotas not verified | PENDING |
| Own web index, native provenance | Vendor-described; independently unverified | Vendor-described; independently unverified | PENDING |
| Failure-domain independence of both hosted services | UNVERIFIED | UNVERIFIED | **BLOCKING** |
| Valid real-result wire fixture and parsing | NOT CAPTURED | NOT CAPTURED | **BLOCKING** |
| DNS/TLS/HTTP and permission/cancel/deadline evidence | NOT CAPTURED | NOT CAPTURED | **BLOCKING** |
| Genuine mainland-China vs comparison-region paired observation | NOT CAPTURED | NOT CAPTURED | **BLOCKING** |

## 4. Questions requiring supplier-grade clarity

For **each** service obtain a source-specific decision: (a) may an independently distributed,
locally installed OSS client ship the anonymous MCP endpoint as its default search destination;
(b) may thousands of independent end users submit queries via that default without obtaining an
individual API key/account; (c) what rate/share limits and 429/Retry-After commitments apply,
especially to users behind a shared egress IP; (d) what attribution, snippet/results display,
model-context inclusion, storage/caching and privacy rules apply; and (e) are there restrictions
for commercial use, embedding or competing/search-product distribution.

For Parallel, explicitly discriminate **public anonymous MCP use** from rights acquired under
a separate, authenticated customer agreement. For Exa, explicitly reconcile the published
MCP integration documentation with Exa Labs Terms of Service §4.2 and any exception granted
by documentation or written permission.

If the right to ship a default integrated anonymous client is not established, classify the
provider as **explicit user-connected MCP only** (a different user choice), not a valid E/F default
route. Do not infer provider permission from other agents' usage.

## 5. Engineering fit without premature implementation

Potential architecture (not a committed design):

```text
web_search (existing model tool; Network permission)
    -> SearchRouter (existing first-valid-success and bounded fallback)
       -> SearchBackend: Exa MCP adapter [UNADMITTED]
       -> SearchBackend: Parallel MCP adapter [UNADMITTED]
       -> existing rust-websearch compatible Auto path
```

Native MCP protocol adapters would need strict endpoint allowlisting and invocation-bound
authority (ADR-090), outbound proxy integration, redirect/SSRF protections, deadlines/cancellation,
no unsolicited startup handshake, normalized search result provenance, limits on excerpts,
MCP content-as-untrusted-data handling, and error classification for JSON-RPC/tool-level errors
and HTTP 429. Do not advertise vendor-added tools to the model or broaden web_fetch permissions.

A backend wrapping MCP is still Talos-authored **SearchBackend** logic; a remote hosted MCP name
does not grant an independent architectural Provider or prove upstream independence.

## 6. Next possible increment: evidence acquisition — not authorized by this packet

1. Finish applicable-use/default-client permission decision with authoritative source or
   vendor clarification. If unresolved, **stop** default-route research; optional user MCP
   connection can be evaluated as separate feature, not E/F substitution.
2. Before collecting MCP evidence, **separately claim, implement, review, and validate an
   MCP-capable extension** to the I294/#827 observation harness. The existing
   `scripts/search_provider_observation.py` supports fixed Mwmbl/Purili HTTPS GET
   observations only; it does not implement MCP initialize/tools/list/tools/call,
   and its metadata-only output omits response bodies and replayable fixtures.
   Reuse its bounded DNS/TLS/HTTP, timeout, privacy, regional provenance and no-
   secret-retention safeguards; add separately authorized search-only MCP protocol
   acquisition and sanitized deterministic fixtures only within that future scope.
   No such extension, live MCP query, or fixture capture is delivered by this intake.
3. Compare controlled-region performance and success from genuine mainland-China and other
   locations; record operator-supplied coarse region/proxy provenance. A VPN, arbitrary public IP,
   offshore CI/VM or guessed location is not mainland-China proof.
4. Measure failure-domain independence, result utility, 429 and cancellation; enforce existing
   router limits and external-data sanitization.
5. Only then produce a new independently reviewed D2 admission decision. E/F and I remain
   blocked until their existing gate conditions are **actually satisfied**.

## 7. Current decision / non-goals

- **Exa: BLOCKED / NOT ADMITTED.**
- **Parallel: BLOCKED / NOT ADMITTED.**
- **No provider should be placed in Auto, nor any provider E/F work started.**
- No production code, dependency, endpoint activation, test network call or architecture
  replacement is authorized by the issue or this draft alone.
- SEARCH-001-H / #832 is complete and must not be reopened as a side effect.
- SEARCH-001-D / #644 is closed with its prior blocked outcome, not reopened here.

Links: [Parent #624](https://github.com/wjhuang88/talos/issues/624) ·
[Historical D #644](https://github.com/wjhuang88/talos/issues/644) ·
[New D2 #848](https://github.com/wjhuang88/talos/issues/848).

## 8. Dated follow-up: Parallel MCP 2.0 (2026-11-10)

**Source-only correction to the 2026-11-04 snapshot; no MCP wire call or production admission.**
Parallel announced MCP 2.0 on **2026-11-05**, after this packet's initial
source checkpoint. Official launch: https://parallel.ai/blog/parallel-mcp-2 ;
current product setup: https://parallel.ai/mcp ; official compatibility note:
https://parallel.ai/articles/hermes-agent-web-search .

- Recommended new no-key endpoint: `https://mcp.parallel.ai/mcp`.
  Earlier `https://search.parallel.ai/mcp` remains a *legacy* Search MCP endpoint
  reported as operational but no longer receiving updates. It must not be
  treated as the canonical current endpoint without fresh verification.
- Public 2.0 tool descriptions: anonymous `web_search`, `web_fetch` and
  `web_decisions`; `web_research` requires sign-in/billed usage. The search
  candidate concerns **only** `web_search` and must not silently expose or
  dispatch the other tools. This replaces the earlier two-tool assumption for
  new 2.0 connections, but does not retroactively change the historical D2-06/07
  evidence recorded above.
- The official announcement indicates product-level configurability and
  organizational controls for signed-in users. **This does not establish a
  default anonymous redistribution/commercial embedding license**, free-scale
  guarantee, independence from Exa, valid real-result wire, or target-region
  reachability. Separate current applicable terms or explicit vendor permission
  remain necessary before default-search admission.
- Do not infer an observed 2.0 `tools/list` or query schema from marketing
  tables. A future separately governed acquisition must pin the new URL,
  negotiate the protocol, capture actual bounded search-only fixtures, and
  keep normal Network/permission/endpoint-manifest limits.

**Updated decision remains: Exa BLOCKED / NOT ADMITTED; Parallel BLOCKED / NOT
ADMITTED.** Existing E/F/G/I gates and the unrelated browser feasibility #891
are unchanged. This supplement is read-only documentary evidence, not an
implementation authorization or final legal clearance.
