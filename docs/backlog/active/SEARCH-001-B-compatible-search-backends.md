# SEARCH-001-B: Compatible SearchBackend Boundary

**Status**: Complete / Closed — I292 compatibility-only delivery; #624 remains Open
**Type**: Implementation / Compatibility
**Parent Epic**: [SEARCH-001](SEARCH-001-zero-config-global-search.md) / [Issue #624](https://github.com/wjhuang88/talos/issues/624)

| Field | Value |
|---|---|
| Story ID | SEARCH-001-B |
| Source Issue | [#642](https://github.com/wjhuang88/talos/issues/642) |
| Depends on | Accepted ADR-085; SEARCH-001-A / #625 merged as `4567bf85` |
| Selected Iteration | [I292](../../iterations/I292-search-compatible-backend-boundary.md) |
| Implementation PR | #653 and #656 merged |
| Last Updated | 2026-11-19 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | GPT-6 Sol / talos开发 session |
| Work Slice | SEARCH-001-B compatibility boundary and adapter wiring |
| Claimed At | 2026-10-04 |
| Source Issue | #642 |
| Governance Claim PR | #651 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Historical effective claim #651 merged `6490118fed913521ac9d1875ccd63fb5e3914af1`; private seam #653, compatible adapter wiring #656, final deterministic fixture #660 and evidence closeout #663 merged. Exact source-head CI #3009/run 37409618084 succeeded for final source #660. |
| Implementation PR | #653 and #656 (seam/adapter) and #660 (final deterministic fixture; actual Completion Commit below) |
| Handoff / Release Condition | I292 compatibility scope accepted through #660 and prior documentation closeout #663. C was later independently delivered; D/#644 completed with ZERO-ADMISSION. Epic #624 Open; E/F/G/I native-provider gates remain unmet. |

Completion Commit: `7728f7681df11a9d715a0e479590c1002c53a26f` (PR #660 squash merge)

Exact-head CI run 37409618084.

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

## Post-delivery owner-first terminal reconciliation — 2026-11-19

**Completion Commit: `7728f7681df11a9d715a0e479590c1002c53a26f`**, accepted final source/fixture [PR #660](https://github.com/wjhuang88/talos/pull/660), already merged on `main`; **not** the earlier adapter-wiring PR #656 merge `44cdab8a1f2f3f367b3603bf0e294e39f854dd0b` and **not** this status-only PR. The private typed seam came via [#653](https://github.com/wjhuang88/talos/pull/653) `65066376cc6622b84e11d1f6c45facd27412b92a`; compatibility adapters via [#656](https://github.com/wjhuang88/talos/pull/656); deterministic no-network fixture gap closure via #660.

Exact final source-head [CI #3009/run 37409618084](https://github.com/wjhuang88/talos/actions/runs/37409618084) completed SUCCESS; previous source-documentary closeout [#663](https://github.com/wjhuang88/talos/pull/663) merged `e55d86cb1e7b045248e0cbec396a8b3388f8b2e1` but retained a stale `Claimed` marker. This is the bounded, owner-first terminal-claim reconciliation, not a second implementation.

The actual canonical owner is **this file**: `docs/backlog/active/SEARCH-001-B-compatible-search-backends.md`, bound to [I292](../../iterations/I292-search-compatible-backend-boundary.md) and [Issue #642](https://github.com/wjhuang88/talos/issues/642). The obsolete `SEARCH-001-B-search-backend-compatible-adapters.md` is only a historical pointer; it is not a competing live owner/claim. #642 may be closed **after** this claim-state correction reaches `main`. Preserved behavior: no new native provider, no default SearchRouter change in B, no new Network/tool/public API change. Parent #624 remains Open and E/F/G/I admission remains blocked after D/#644 ZERO-ADMISSION.
