# SEARCH-001-B: Compatible SearchBackend Boundary

**Status**: Active / Claimed
**Type**: Implementation / Compatibility
**Parent Epic**: [SEARCH-001](SEARCH-001-zero-config-global-search.md) / [Issue #624](https://github.com/wjhuang88/talos/issues/624)

| Field | Value |
|---|---|
| Story ID | SEARCH-001-B |
| Source Issue | [#642](https://github.com/wjhuang88/talos/issues/642) |
| Depends on | Accepted ADR-085; SEARCH-001-A / #625 merged as `4567bf85` |
| Selected Iteration | [I291](../../iterations/I291-search-compatible-backend-boundary.md) |
| Implementation PR | None |
| Last Updated | 2026-10-03 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | GPT-6 Sol / talos开发 session |
| Work Slice | SEARCH-001-B compatibility boundary and adapter characterization; no router behavior change |
| Claimed At | 2026-10-03 |
| Source Issue | #642 |
| Governance Claim PR | #651 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Governance-only claim+activation PR #651; exact-head CI, both governance validators and merge-time CAS required. |
| Handoff / Release Condition | Claim and I291 activation become effective only after #651 merges; implementation PR follows from that merge. |

## Goal

Create the private `SearchBackend` request/result/error seams inside `talos-tools` and wrap the
current rust-websearch, Tavily, SearXNG and Wikipedia compatibility paths. This is a structural
boundary with a compatibility target; C owns the later Talos-router behavior change.

## Constraints

- Preserve `web_search`, its schema, Network permission, contribution group and model-facing output.
- Preserve current provider activation and fallback behavior during B.
- Keep `rust-websearch` as a replaceable compatibility adapter.
- Do not add native providers, startup probes, GeoIP routing, paid-provider consent changes,
  generic retry/circuit-breaker logic, public crates or persisted config migration.

## Acceptance

- Private typed backend/request/result/error contracts exist under `talos-tools`.
- Existing adapters are covered by deterministic fixtures and remain behavior-compatible.
- Production search behavior has no intentional diff beyond the internal extraction.
- Locked focused tests, governance validators and owner/Issue synchronization pass.

## User-Facing Documentation

No user-facing behavior or configuration changes are advertised by B. Update compatibility notes
only if the internal boundary changes diagnostics visible to existing users.

## Rollback

Revert the structural extraction and retain the pre-B `WebSearchTool` path; no provider or config
migration is allowed to depend on B alone.
