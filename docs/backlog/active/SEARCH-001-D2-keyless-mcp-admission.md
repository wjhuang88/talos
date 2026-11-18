# SEARCH-001-D2: Hosted Keyless MCP Search Admission Triage

**Status**: Review / Claimed — I331 D2.2 documentary evidence PR #981 under review; **Exa/Parallel production BLOCKED / NOT ADMITTED**

| Field | Value |
|---|---|
| Story ID | SEARCH-001-D2 |
| Type | Research Spike / evidence gate |
| Parent | [SEARCH-001 / #624](https://github.com/wjhuang88/talos/issues/624) |
| Issue | [#848](https://github.com/wjhuang88/talos/issues/848) |
| Historical predecessor | SEARCH-001-D / #644, I294: **Complete / Closed — BLOCKED / ZERO-ADMISSION** |
| Selected iteration | [I331](../../iterations/I331-search-keyless-mcp-rights-evidence.md) — claim proposal #980, ineffective before merge |
| Intake evidence | [D2 hosted MCP triage — 2026-11-04](../../reference/SEARCH-001-D2-MCP-TRIAGE-2026-11-04.md) |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | D2.2 / #848: published-source-only Exa/Parallel current MCP contract delta and anonymous/default-distribution rights applicability review; dated compliance/independence matrix, supplier clarification questions and **future** bounded MCP fixture/region acquisition preconditions. No live MCP, API, SERP requests, harness/provider code, default Auto or Network authority. |
| Claimed At | 2026-11-18 |
| Source Issue | #848 |
| Governance Claim PR | #980 |
| Authorization Mode | Independent review |
| Authorization Evidence | Proposed atomic I331 claim in PR #980; exact-head native documentation CI, independent read-only review and merge-time expected-head CAS required BEFORE activation. No acceptance asserted while this PR is open. |
| Implementation PR | [#981](https://github.com/wjhuang88/talos/pull/981) — Draft source-documentary evidence candidate; NOT YET MERGED or accepted |
| Last Updated | 2026-11-18 |
| Handoff / Release Condition | I331 D2.2 may publish reviewed documentary gate evidence only after #980 merge; close its claimed slice owner-first with actual prior evidence Completion Commit. Exa/Parallel production stay BLOCKED until independent rights, protocol/region and Security/Network gates; #848 remains Open for residuals. No E/F admission. |

## Objective

Determine whether officially hosted, anonymous/no-key Exa and Parallel MCP search services can lawfully,
operationally, and technically support a **distributed Talos default Auto SearchBackend** under
SEARCH-001's existing definition. A user manually attaching an optional MCP endpoint is not proof
that a product may preconfigure that endpoint for all users.

## Increment D2.1 — scope and acceptance

1. Pin official provider MCP endpoints, default tool contracts, authentication and rate-limit
   policy. Distinguish operator claims from live protocol observations and non-binding marketing.
2. Reconcile service terms, output rights, anonymous access conditions and allowable use by
   distributed local clients; block admission on unresolved material policy conflicts.
3. Record hosting, upstream index provenance and correlated-failure risks; two different DNS names
   alone do not satisfy ADR-085 independence.
4. Define bounded MCP live/fixture and genuine mainland-China evidence requirements before
   any behavioral child. The existing I294/#827 `scripts/search_provider_observation.py`
   observes fixed Mwmbl/Purili HTTPS GET metadata only: it cannot negotiate MCP or
   retain replayable search fixtures. A **separately claimed and reviewed MCP-capable
   harness extension** is a hard prerequisite to approved MCP acquisition, reusing
   #827's bounds, source provenance, privacy, and regional-truth safeguards.
   This intake authorizes no harness implementation or live MCP calls.
5. Publish a binary evidence-gate outcome (per route: admissible only with complete evidence;
   otherwise explicitly blocked) and any separately owned next acquisition slice.

## Out of scope

No default-route change, production code, dependency, model-facing tool schema, new account/API
credentials, unsolicited network probes, persistent MCP connections, stealth circumvention,
paid fallback, or changes to already completed H/#832. Do not alter or reinterpret the historical
D/#644 decision. E/F/G/I remain unauthorized or blocked until their own documented gates pass.

## Dependencies and authorization

This is a **proposed unclaimed** evidence-only increment. Read-only investigation and the linked
intake proposal are not a claim; effective claim and iteration activation require a separate
governed atomic PR before new committed implementation work. Current other active iterations
must be inventoried before selection.

Any future adapter must remain an **internal SearchBackend** of the existing built-in Search
Provider under ADR-072/085 and honor ADR-090 endpoint authority, Network permission, proxy,
redirect, cancellation and deadlines. External MCP tools must not be blindly exposed as new
model-visible Talos tools.

## Initial disposition

**Exa: BLOCKED / NOT ADMITTED** pending default-distribution and output-use permission,
representative valid-query evidence, and independent/regional qualification.

**Parallel: BLOCKED / NOT ADMITTED** pending applicability of terms to anonymous MCP embedded in
a distributable default client, free-versus-production limits, and independent/regional qualification.

No evidence from this intake changes the parent #624 acceptance or authorizes E/F.

## Documentary source refresh — 2026-11-10

Parallel's official 2026-11-05 MCP 2.0 launch changed the recommended
anonymous endpoint to `https://mcp.parallel.ai/mcp` and expanded its published
free tool set (including `web_decisions`). The prior `https://search.parallel.ai/mcp`
remains a legacy endpoint; see the [dated source supplement](../../reference/SEARCH-001-D2-MCP-TRIAGE-2026-11-04.md#8-dated-follow-up-parallel-mcp-20-2026-11-10).
This is source-only triage: no live request, no applicability-of-terms approval,
no provider admission and no change to default Auto or claimed ownership.
**Exa and Parallel both remain BLOCKED / NOT ADMITTED.**

## I331 D2.2 proposed claim and bounded Work Slice — 2026-11-18

The 2026-11-10 D2.1 publication already documents official hosted no-key MCP surfaces, a source matrix and **BLOCKED / NOT ADMITTED** decisions. Do not repeat that research or treat publication as an effective execution claim. A non-overlapping D2.2 source-only follow-up is proposed for `I331` in **governance-only claim PR #980**. Its Claimed row above and Active delivery status are **only branch proposals until #980 merges**; no research-output or implementation commit is authorized before that.

Scope: pin current vendor MCP 2.0/Exa official tool-contract deltas, source-date and applicable anonymous/paid service and output-use terms; produce a vendor-specific published-source versus unverified-wire/rights decision table, independent failure-domain limitations, a concrete vendor clarification questionnaire, and a gated future bounded MCP observer/region test plan. Do **not** send MCP `initialize/tools/list/tools/call` requests or any search query, capture live results, implement a harness/adapter, add Network authority or modify Auto. Distinguish anonymous personal/manual MCP use from default distributed embedding. The D2.1 evidence packet is historical baseline; D2.2 will publish a separate dated assessment only after this claim is effectively merged.

## I331 D2.2 source-documentary checkpoint — 2026-11-18 (Review only)

Effective I331 governance claim [PR #980](https://github.com/wjhuang88/talos/pull/980) **merged** on target main at `e290e8882e5a8be23e90f962f444d71861c6eca6` after exact-head [CI #4387](https://github.com/wjhuang88/talos/actions/runs/44255790659) 4 applicable SUCCESS/4 SKIPPED, independent [APPROVE](https://github.com/wjhuang88/talos/pull/980#issuecomment-6971707928) and merge-time CAS. The earlier invalid I330/Search claim [PR #977](https://github.com/wjhuang88/talos/pull/977) closed UNMERGED; I330 belongs to ARCH-035 E2, I331 only to D2.2.

A separate **source-documentary** [PR #981](https://github.com/wjhuang88/talos/pull/981) now proposes one dated [official-source contract/rights delta](../../reference/SEARCH-001-D2-HOSTED-MCP-RIGHTS-DELTA-2026-11-18.md). Distinct from completed D2.1/#849, it documents the current Exa public default/optional tools and auth; Parallel's 2026-11-05 MCP 2.0 launch versus 2026-11-18 *six-anonymous-tool* official docs; November 10 contracted terms vs anonymous distributed-default rights applicability UNKNOWN; result/LLM-context/cache/privacy/independence matrix, supplier clarification **questions only**, and separately governed future MCP fixture and genuine China/comparator evidence criteria.

**Current state: Review / Claimed; no accepted Completion Commit.** PR #981 still needs exact-head docs CI, independent review, final CAS and source merge; only after actual merge may an owner-first closeout record the prior source Completion Commit. Neither vendor is admitted, no vendor contact/live MCP/HTTP query occurred, no Network/Auto/provider/Browser change is authorized. Exa and Parallel **BLOCKED / NOT ADMITTED**; parent #624 and child #848 remain Open. D/#644 terminal ZERO-ADMISSION, I329 T3a/T3b and E/F/G/I unchanged.
