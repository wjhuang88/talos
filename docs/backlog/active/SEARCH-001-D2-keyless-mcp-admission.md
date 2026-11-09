# SEARCH-001-D2: Hosted Keyless MCP Search Admission Triage

**Status**: Intake / Proposed / Unclaimed — no production provider admitted

| Field | Value |
|---|---|
| Story ID | SEARCH-001-D2 |
| Type | Research Spike / evidence gate |
| Parent | [SEARCH-001 / #624](https://github.com/wjhuang88/talos/issues/624) |
| Issue | [#848](https://github.com/wjhuang88/talos/issues/848) |
| Historical predecessor | SEARCH-001-D / #644, I294: **Complete / Closed — BLOCKED / ZERO-ADMISSION** |
| Selected iteration | None |
| Intake evidence | [D2 hosted MCP triage — 2026-11-04](../../reference/SEARCH-001-D2-MCP-TRIAGE-2026-11-04.md) |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | None — research intake only |
| Claimed At | Not applicable |
| Source Issue | #848 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | None |
| Last Updated | 2026-11-10 |
| Handoff / Release Condition | Intake-only; remain Proposed / Unclaimed with both providers BLOCKED until policy/use rights, live protocol and regional evidence are verified. Any activation requires separately approved claim and iteration; no Auto or E/F admission. |

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
