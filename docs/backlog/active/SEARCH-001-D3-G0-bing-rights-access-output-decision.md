# SEARCH-001-D3-G0: Bing Access and Output Rights Decision

**Status**: Intake / Unclaimed — evidence and rights determination only; no code or live/default Bing authorization
**Type**: Supplier / Rightsholder Access and Result-Use Evidence Gate
**Parent Epic**: [SEARCH-001 / #624](SEARCH-001-zero-config-global-search.md)
**Source Issue**: [#1000](https://github.com/wjhuang88/talos/issues/1000)
**Predecessor**: [SEARCH-001-D3 / #937](SEARCH-001-D3-bing-direct-html-feasibility.md)
**Related**: [D3-T3 / #957](SEARCH-001-D3-T3-bing-dual-host-zero-key-route.md); [D3-T5 / #1001](SEARCH-001-D3-T5-bing-dual-runtime.md) implementation-first distinct work
**Selected Iteration**: None; no active claim selected
**Last Updated**: 2026-11-22

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | None; read-only supplier/rightsholder evidence intake |
| Claimed At | Not applicable |
| Source Issue | #1000 |
| Governance Claim PR | None — separate claimed research task not yet selected |
| Authorization Mode | Not applicable |
| Authorization Evidence | Microsoft 2026 Bing/MSN terms and distinct Foundry/Web IQ contracts do not establish applicable raw Bing HTML access + result/LLM-context use rights; see #1000 evidence matrix and unsent supplier inquiry |
| Implementation PR | None |
| Handoff / Release Condition | Written scoped applicable **service access AND output-use** rights, or documented NO-GO; this Intake owner does not permit new live WWW Bing calls/default registration |
| Last Updated | 2026-11-22 |

## Goal

Record the actual permission scope for distributed end-user-initiated `www.bing.com/search` and optional `cn.bing.com/search` HTML requests, parsing/display/LLM-context/attribution/cache, commercial and regional limits. Default **BLOCKED / UNPROVEN** for production admission until a documented legal/rightsholder determination is reached. This is a separate evidence gate: implementation D3-T5/#1001 may develop real protected transport and explicit opt-in tests, without claiming legal admission or changing shipped default.

## Acceptance (not yet satisfied)

- [ ] Identify scoped written automated-access authorization, applicable service, location, product distribution and rate/redirect/proxy limits.
- [ ] Identify separately applicable organic URL/title/snippet, model-context, cache/retention/attribution and third-party content rights.
- [ ] Record reviewer/source, effective date, governing agreement and explicit GO/NO-GO; if unverified, keep T3b/#957 production/default blocked.
- [ ] If GO, separately authorize real-region validation and production activation with ADR-090, protected independent review, CI and rollback.

## Evidence and non-goals

[Gate-0 #1000](https://github.com/wjhuang88/talos/issues/1000#issuecomment-7075498403) retains dated access/output matrix and **UNSENT** supplier inquiry. No live automated Bing acquisition, license acceptance, Microsoft contact, bypass, source code change, independent provider admission or claim activation is asserted by this record. Parent D3/#937, SEARCH-001/#624 remain Open, T3b/#957 blocked, historical #644 ZERO-ADMISSION unchanged.
