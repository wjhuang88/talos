# SEARCH-001-D2: Hosted Keyless MCP Search Admission Triage

**Status**: Complete / Closed for **I331 D2.2 source-documentary evidence only** — Exa/Parallel **BLOCKED / NOT ADMITTED**; #848 remains Open for external admission/rights/region gates

Completion Commit: `74caf947521bca51f7e3ff0a71509da78a237365` (accepted official-source evidence PR #981 squash merge to main; **not** this status-only closeout)

| Field | Value |
|---|---|
| Story ID | SEARCH-001-D2 |
| Type | Research Spike / evidence gate |
| Parent | [SEARCH-001 / #624](https://github.com/wjhuang88/talos/issues/624) |
| Issue | [#848](https://github.com/wjhuang88/talos/issues/848) |
| Historical predecessor | SEARCH-001-D / #644, I294: **Complete / Closed — BLOCKED / ZERO-ADMISSION** |
| Selected iteration | [I331](../../iterations/I331-search-keyless-mcp-rights-evidence.md) — **effective** documentary-only claim via merged #980 (`e290e8882e5a8be23e90f962f444d71861c6eca6`) |
| Intake evidence | [D2 hosted MCP triage — 2026-11-04](../../reference/SEARCH-001-D2-MCP-TRIAGE-2026-11-04.md) |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | D2.2 / #848: published-source-only Exa/Parallel current MCP contract delta and anonymous/default-distribution rights applicability review; dated compliance/independence matrix, supplier clarification questions and **future** bounded MCP fixture/region acquisition preconditions. No live MCP, API, SERP requests, harness/provider code, default Auto or Network authority. |
| Claimed At | 2026-11-18 |
| Source Issue | #848 |
| Governance Claim PR | #980 |
| Authorization Mode | Independent review |
| Authorization Evidence | I331 claim **activated** by merged PR #980 at `e290e8882e5a8be23e90f962f444d71861c6eca6` after [exact-head CI #4387](https://github.com/wjhuang88/talos/actions/runs/44255790659), [independent read-only APPROVE](https://github.com/wjhuang88/talos/pull/980#issuecomment-6971707928), and merge-time CAS. This authorizes published-source research only, not MCP acquisition or production admission. |
| Implementation PR | [#981](https://github.com/wjhuang88/talos/pull/981), accepted source-only evidence **merged** as Completion Commit `74caf947521bca51f7e3ff0a71509da78a237365` |
| Last Updated | 2026-11-18 |
| Handoff / Release Condition | I331 D2.2 published its approved source-only delta (#981, Completion Commit `74caf947521bca51f7e3ff0a71509da78a237365`); claim Closed for this bounded slice upon owner-first closeout merge. **#848 remains Open** for external service/output rights, live MCP fixture and real-region evidence under separately claimed work. Exa/Parallel production remain BLOCKED; no E/F/G/I admission, Network/Auto changes or provider-count credit. |

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

This paragraph originally described the **D2.1 intake** before selection. D2.2/I331 now has an **effective, narrowly scoped documentary-only claim** through merged PR #980 (`e290e8882e5a8be23e90f962f444d71861c6eca6`). That claim authorizes the separate source-only PR #981, not an MCP fixture harness, external requests, Network permission, production integration or default-route admission. Any new behavioral/live acquisition still requires its own governed owner/iteration and accepted authorization.

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

## I331 D2.2 authorized documentary Work Slice — claim effective 2026-11-18

The 2026-11-10 D2.1 publication already documents official hosted no-key MCP surfaces, a source matrix and **BLOCKED / NOT ADMITTED** decisions. Do not repeat that research or treat publication as an effective execution claim. The non-overlapping D2.2 source-only follow-up was **selected and activated** in merged governance-only claim PR #980. Its effective owner/iteration state is **Review / Claimed** while independent source-documentary PR #981 is under review; this is not acceptance or a Completion Commit. No live MCP requests or provider implementation are authorized.

Scope: pin current vendor MCP 2.0/Exa official tool-contract deltas, source-date and applicable anonymous/paid service and output-use terms; produce a vendor-specific published-source versus unverified-wire/rights decision table, independent failure-domain limitations, a concrete vendor clarification questionnaire, and a gated future bounded MCP observer/region test plan. Do **not** send MCP `initialize/tools/list/tools/call` requests or any search query, capture live results, implement a harness/adapter, add Network authority or modify Auto. Distinguish anonymous personal/manual MCP use from default distributed embedding. The D2.1 evidence packet is historical baseline; D2.2's separate dated official-source assessment was subsequently **accepted and merged via PR #981** after its own exact-head CI, independent source review and final CAS; the pre-existing accepted source Completion Commit is `74caf947521bca51f7e3ff0a71509da78a237365`.

## I331 D2.2 source-documentary stage — 2026-11-18 (historical Review checkpoint)

Effective I331 governance claim [PR #980](https://github.com/wjhuang88/talos/pull/980) **merged** on target main at `e290e8882e5a8be23e90f962f444d71861c6eca6` after exact-head [CI #4387](https://github.com/wjhuang88/talos/actions/runs/44255790659) 4 applicable SUCCESS/4 SKIPPED, independent [APPROVE](https://github.com/wjhuang88/talos/pull/980#issuecomment-6971707928) and merge-time CAS. The earlier invalid I330/Search claim [PR #977](https://github.com/wjhuang88/talos/pull/977) closed UNMERGED; I330 belongs to ARCH-035 E2, I331 only to D2.2.

A separate **source-documentary** [PR #981](https://github.com/wjhuang88/talos/pull/981) proposed (and was subsequently accepted/merged as Completion Commit `74caf947521bca51f7e3ff0a71509da78a237365`) one dated [official-source contract/rights delta](../../reference/SEARCH-001-D2-HOSTED-MCP-RIGHTS-DELTA-2026-11-18.md). Distinct from completed D2.1/#849, it documents the current Exa public default/optional tools and auth; Parallel's 2026-11-05 MCP 2.0 launch versus 2026-11-18 *six-anonymous-tool* official docs; November 10 contracted terms vs anonymous distributed-default rights applicability UNKNOWN; result/LLM-context/cache/privacy/independence matrix, supplier clarification **questions only**, and separately governed future MCP fixture and genuine China/comparator evidence criteria.

**Historical source-stage status** was Review / Claimed without an accepted Completion Commit. That evidence is now accepted: [CI #4400](https://github.com/wjhuang88/talos/actions/runs/44268853702) SUCCESS on exact head `cd65e88e736229fcd225d16b47e4ed93f91edf45`, independent [APPROVE](https://github.com/wjhuang88/talos/pull/981#issuecomment-6973577247), final CAS and actual source merge `74caf947521bca51f7e3ff0a71509da78a237365`. The I331 documentary-only claim closes in this later owner-first closeout PR, not retroactively at the source stage. Neither vendor is admitted, no vendor contact/live MCP/HTTP query occurred, no Network/Auto/provider/Browser change is authorized. Exa and Parallel **BLOCKED / NOT ADMITTED**; parent #624 and child #848 remain Open. D/#644 terminal ZERO-ADMISSION, I329 T3a/T3b and E/F/G/I unchanged.

## I331 D2.2 owner-first closeout — 2026-11-18

**Completion Commit: `74caf947521bca51f7e3ff0a71509da78a237365`** — pre-existing accepted source/documentary implementation from [PR #981](https://github.com/wjhuang88/talos/pull/981), already merged to `main` before this status-only closeout. This document commit is **not** its own completion evidence. The claim/iteration for **only this I331 published-source D2.2 scope** is **Complete / Closed** upon this governance closeout's merge.

- **Claim/authorization:** I331 atomic governance [PR #980](https://github.com/wjhuang88/talos/pull/980) accepted at `e290e8882e5a8be23e90f962f444d71861c6eca6`, after [CI #4387](https://github.com/wjhuang88/talos/actions/runs/44255790659) 4 applicable successful jobs/4 scoped skips, independent [claim review](https://github.com/wjhuang88/talos/pull/980#issuecomment-6971707928), merge CAS. I330 was already assigned to independent ARCH-035 E2; no overlap.
- **Source evidence:** dated [official-source rights and MCP tool-surface delta](../../reference/SEARCH-001-D2-HOSTED-MCP-RIGHTS-DELTA-2026-11-18.md). Exa published-source default/optional tools and auth; Parallel Nov 5 launch vs official Nov 18 six-anonymous-tool docs, authenticated `Authorization`/Bearer and `?login` OAuth differences; applicable distributed-default access, snippets/LLM/context, cache, privacy and failure-domain independence remain unproven. Supplier questions are **not** vendor contact; MCP acquisition and true-China/comparator are future separately authorized gates.
- **Source validation:** accepted PR #981 candidate exact head `cd65e88e736229fcd225d16b47e4ed93f91edf45`, [CI #4400/run 44268853702](https://github.com/wjhuang88/talos/actions/runs/44268853702) **SUCCESS**: remote Issue/owner reconciliation, docs governance validation, scope classification, Windows installer fixture succeeded, four platform-specific jobs correctly SKIPPED for documentation-only scope. The original [CI #4393](https://github.com/wjhuang88/talos/actions/runs/44257297172) passed on the earlier head, while #4398 was cancelled and superseded; no old CI was used for final acceptance.
- **Independent exact-head content/security review:** [Copilot APPROVE](https://github.com/wjhuang88/talos/pull/981#issuecomment-6973577247), explicitly bound to `cd65e88e736229fcd225d16b47e4ed93f91edf45` and `main@e290e8882e5a8be23e90f962f444d71861c6eca6`, no blocking documentary findings; reviewer is technical/read-only, **not** a vendor/legal license grant or live transport/region verifier. Verified published-source-only scope and correct Review/Claimed source-stage evidence; source merge passed exact-head CAS.
- **Accepted bounded outcome:** **Exa BLOCKED / NOT ADMITTED; Parallel BLOCKED / NOT ADMITTED**. No default search provider or independent-source count added, no live MCP `initialize/tools/list/tools/call`/query, no credential, supplier outreach, actual regional test, SearchBackend/Auto, Network/permission, browser, dependency or payment change. Historical D2.1/#849 and original D/#644 ZERO-ADMISSION remain unchanged.
- **Residuals:** the originating [Issue #848](https://github.com/wjhuang88/talos/issues/848) and [Epic #624](https://github.com/wjhuang88/talos/issues/624) remain **OPEN**. Separate effective ownership is required for vendor-specific distributed-default/output-use legal/rightsholder qualification and authorization, bounded MCP live fixture harness/true mainland-China and comparator reachability, and any later SearchBackend implementation. Do **not** close #848 merely because the I331 source-only slice reached its honest BLOCKED disposition; do not activate E/F/G/I.
