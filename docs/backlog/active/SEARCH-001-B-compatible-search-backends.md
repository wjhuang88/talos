# SEARCH-001-B: Compatible SearchBackend Boundary

**Status**: Review / Claimed
**Type**: Implementation / Compatibility
**Parent Epic**: [SEARCH-001](SEARCH-001-zero-config-global-search.md) / [Issue #624](https://github.com/wjhuang88/talos/issues/624)

| Field | Value |
|---|---|
| Story ID | SEARCH-001-B |
| Source Issue | [#642](https://github.com/wjhuang88/talos/issues/642) |
| Depends on | Accepted ADR-085; SEARCH-001-A / #625 merged as `4567bf85` |
| Selected Iteration | [I292](../../iterations/I292-search-compatible-backend-boundary.md) |
| Implementation PR | #653 and #656 merged |
| Last Updated | 2026-10-04 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | GPT-6 Sol / talos开发 session |
| Work Slice | SEARCH-001-B compatibility boundary and adapter wiring |
| Claimed At | 2026-10-04 |
| Source Issue | #642 |
| Governance Claim PR | #651 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | #651 merged as `6490118fed913521ac9d1875ccd63fb5e3914af1`; #653 and #656 exact-head CI passed before merge |
| Handoff / Release Condition | B complete; C and D require their own claims and bounded iterations |

Completion Commit: pending follow-up acceptance

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
