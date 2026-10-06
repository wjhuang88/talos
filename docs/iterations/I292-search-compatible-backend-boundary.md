# Iteration I292: Search Compatible Backend Boundary

> Document status: Review
> Published plan date: 2026-10-04
> Planned objective: Implement SEARCH-001-B's private SearchBackend compatibility boundary without changing production search behavior.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: A testable internal backend seam in `talos-tools` with current search adapters characterized and output-compatible.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | GPT-6 Sol / talos开发 session |
| Work Slice | SEARCH-001-B compatibility boundary only; no router behavior change |
| Claimed At | 2026-10-04 |
| Source Issue | #642 |
| Governance Claim PR | #651 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Governance-only claim+activation PR #651; exact-head CI, both governance validators and merge-time CAS required. |
| Implementation PR | #653 and #656 merged |
| Last Updated | 2026-10-04 |
| Handoff / Release Condition | Atomic claim+activation before implementation |

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| SEARCH-001-B | SEARCH-001 / #624 | Proposed / Unclaimed | Accepted ADR-085; SEARCH-001-A / #625 merged as `4567bf85` | Private compatibility seams and adapter characterization |

### Scope

- Extract private request/result/error/backend seams under `talos-tools`.
- Wrap existing rust-websearch, Tavily, SearXNG and Wikipedia paths as compatibility adapters.
- Add deterministic adapter/normalization fixtures without network calls.
- Preserve the existing `web_search` contract, permissions, provider activation and output.

### Non-Goals

- No SearchRouter behavior change, native provider, startup probe, GeoIP routing or premium-consent change.
- No generic retry/circuit breaker, public crate/API, dependency change or persisted config migration.

### Acceptance

- Given the existing search inputs, when the compatibility adapters execute, then the current
  model-facing schema/output and activation semantics remain unchanged.
- Given deterministic fixtures, when adapter responses are normalized, then typed seams preserve
  source, parse and failure information without issuing network traffic.
- Given the staged diff, when reviewed, then no production routing policy has changed.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo test -p talos-tools --features network --locked`
- `scripts/validate_project_governance.sh .`
- `bash scripts/validate_collaboration_claims.sh .`
- `git diff --check`

### Documentation To Update

- `docs/backlog/active/SEARCH-001-B-compatible-search-backends.md`
- `docs/backlog/active/SEARCH-001-zero-config-global-search.md`
- `docs/BOARD.md` and `docs/backlog/PRODUCT-BACKLOG.md`

### Risks And Rollback

- Risk: extraction accidentally changes fallback, provider activation or output formatting.
- Rollback: revert the structural extraction and retain the pre-B `WebSearchTool` path.

## Selection Inventory — 2026-10-04

Target baseline: `510dd646`. No other Active iteration exists on this baseline.

| Owner | Current State | Disposition For This Selection |
|---|---|---|
| I249 dependency upgrade pilot | Planned | Retain separately planned; no dependency changes in B. |
| I277 Desktop mock visual/i18n | Review; human acceptance Deferred | Retain Review/Deferred; no Desktop scope in B. |
| I290 Auto Review locale | Review | Retain Review; no locale or permission scope in B. |
| I291 locale product follow-up | Planned / Unclaimed | Retain its published plan and owner; B uses I292 and touches only search. |

The temporary duplicate B registration `SEARCH-001-B-search-backend-compatible-adapters.md`
remains an unclaimed inventory pointer; the full B owner linked above is authoritative for #642.
C waits for B delivery. D remains separately unclaimed. No release is authorized.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-10-04 | Atomic claim+activation effective | PR #651 merged as `6490118fed913521ac9d1875ccd63fb5e3914af1`; I292 and the B claim are active on `main`. |

## Verification Evidence

- Implementation PR #653 merged as `65066376cc6622b84e11d1f6c45facd27412b92a`, adding the private typed boundary and deterministic fixtures.
- Follow-up PR #656 merged as `44cdab8a1f2f3f367b3603bf0e294e39f854dd0b`, routing the existing rust-websearch, Tavily, SearXNG and Wikipedia paths through compatibility adapters and preserving the existing formatter and activation policy.

## Completion Evidence

- Completion Commit: pending follow-up acceptance

## Variance And Residuals

- SEARCH-001-C and SEARCH-001-D are now dependency-ready, with their own governance claims still required before implementation.

## Retrospective

- Outcome: Review; the private boundary and compatibility adapters landed, with deterministic parser/normalization characterization pending in the follow-up candidate.
- Documentation: completion evidence recorded here and in SEARCH-001-B.
- Lessons: keep the private seam separate from platform Provider terminology and retain the existing optional-provider and Wikipedia fallback policy until router work is separately claimed.
