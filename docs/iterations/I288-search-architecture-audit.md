# Iteration I288: Search Architecture Audit And Migration Contract

> Document status: Review / Claimed
> Published plan date: 2026-09-30
> Planned objective: Establish the reviewed Talos-owned search architecture, compatibility baseline, provider-admission evidence contract, and rollbackable migration matrix for SEARCH-001-A without changing production search behavior.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: A reviewable SEARCH architecture ADR/proposal package that proves current behavior, defines provider/routing ownership and global evidence gates, and makes later SEARCH-001 children independently claimable.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | GPT-5.6 Sol / talos开发 session |
| Work Slice | SEARCH-001-A architecture audit, deterministic current-behavior characterization, provider-admission/multi-region evidence plan, ADR and B–I migration matrix only; no production search implementation |
| Claimed At | 2026-09-30 |
| Source Issue | #625 |
| Governance Claim PR | #627 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Governance claim #627 merged as `6f651a28fe901a894942ab1c56015a60491a054e` after exact-head reduced CI, both governance validators, remote Issue reconciliation and merge-time CAS. |
| Implementation PR | #629 |
| Last Updated | 2026-09-30 |
| Handoff / Release Condition | Accepted architecture/ADR and recorded compatibility/provider evidence gates; later children remain separately unclaimed |

Claim and activation became effective when governance PR #627 merged as `6f651a28fe901a894942ab1c56015a60491a054e`. I288 remains architecture/evidence-only: production search behavior, dependencies, configuration and provider implementation are still outside this iteration.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| SEARCH-001-A / #625 | SEARCH-001 / #624 | Ready / Unclaimed | SEARCH-001 intake #626 merged; no implementation dependency | Reviewed code-truth/behavior baseline, architecture ADR, provider admission plan and B–I migration contract |

### Existing Iteration / Work Inventory And Disposition

- I286 is Complete / Closed.
- I287 remains Planned / Unclaimed and is not activated or superseded.
- I285 remains Review / Partial for Desktop H1/H6 residual acceptance; this architecture slice does not consume or change those acceptance rows.
- DEPENDENCY-003-A remains Active / Claimed under #604. The maintainer explicitly requested SEARCH-001 progression on 2026-09-30; I288 is permitted as a parallel, non-overlapping **architecture/governance-only** slice. It must not edit dependency manifests, libc/native-boundary code, or #502 ownership.
- No other open PR may claim SEARCH-001-A's Work Slice. Re-check this at merge-time CAS.

### Scope

- Audit current `web_search` implementation, registration, configuration, permission/disclosure, dependency and URL-fetch boundaries against repository truth.
- Produce deterministic evidence for current first-completed vs first-valid-success behavior, cancellation/deadline, all-fail/empty/parse-drift and fallback semantics; tests or disposable harnesses must not alter production behavior.
- Define provider qualification: endpoint/failure-domain independence, maintenance ownership, terms/rate-limit/challenge risk, privacy/query exposure, parser fixtures, regional test provenance and real mainland-China evidence acquisition.
- Propose and review the Search architecture decision: request/result/error/provider/router/health ownership, no-startup-network default, auto-routing/hedging boundary, config/key policy, and explicit composition with ADR-072 plus NET-001/#199.
- Decide whether a `talos-search` crate is warranted or an existing module boundary is sufficient.
- Publish B–I compatibility, activation, validation and rollback matrix; create later child Issues only after the decision makes them dependency-ready.
- Synchronize SEARCH-001 / SEARCH-001-A owners, Issue status, backlog and Board with actual evidence.

### Non-Goals

- No production provider implementation, provider replacement, dependency addition/removal, crate/API migration, persisted configuration change, Tool schema/output behavior change, default route change, startup probe, relay, geolocation-based selection, CAPTCHA/access-control bypass, or automatic paid-provider use.
- No edits to DEPENDENCY-003-A/libc work or Desktop I285/I287 product scope.
- No claim that any candidate provider is globally available until provenance-backed controlled evidence exists.
- No activation of SEARCH-001-B through I; this iteration only makes later owners decidable.

### Acceptance

- Given current Talos main, when the audit is replayed, then the existing provider call graph, feature/dependency/configuration and permission/disclosure boundaries are documented from source truth.
- Given deterministic fast-failure/slow-success and related scenarios, when the current selection logic is exercised or modeled, then first-completed vs first-valid-success semantics and cancellation/fallback behavior are evidenced without production changes.
- Given candidate zero-key routes, when provider admission is assessed, then independent failure domains, parser/update ownership, privacy/terms risks, regional test provenance and the path for real mainland-China evidence are explicit; unknowns remain blockers rather than invented success.
- Given ADR-072 and NET-001/#199, when the architecture is proposed, then search-specific routing/provider ownership composes with those authorities and does not create a competing generic lifecycle/retry system.
- Given future migration stages, when the matrix is reviewed, then B–I have explicit compatibility, activation, test, rollback and retirement gates while remaining unclaimed.
- Given no behavior implementation in this slice, when validation completes, then default `web_search` behavior and dependencies are unchanged.

### Planned Validation

- `git diff --check`
- `scripts/validate_project_governance.sh .`
- `scripts/validate_collaboration_claims.sh .`
- focused deterministic tests or documented disposable experiments for existing `WebSearchTool` selection semantics if code-level evidence is needed
- `cargo test -p talos-tools --features network --locked` if architecture evidence adds/changes tests
- appropriate locked workspace validation for any non-document test-only changes
- exact-head GitHub CI and architecture/API review of the final stable candidate

### Documentation To Update

- `docs/backlog/active/SEARCH-001-A-architecture-migration-contract.md`
- `docs/backlog/active/SEARCH-001-zero-config-global-search.md`
- a new Search architecture ADR/proposal if accepted for review
- `docs/backlog/PRODUCT-BACKLOG.md`, `docs/BOARD.md`, requirement convergence and Issue #624/#625 status as appropriate

### Risks And Rollback

- Risk: architecture duplicates ADR-072 Provider or NET-001 resilience ownership.
  Rollback: retain current `WebSearchTool` architecture and reject/revise the proposal; no production behavior changed.
- Risk: candidate zero-key providers cannot be verified or are operationally/terms-fragile.
  Rollback: keep them out of native-provider admission and split a bounded feasibility residual.
- Risk: a new crate creates publication/dependency overhead without a strong boundary.
  Rollback: keep Search domain behind an existing crate/module seam until evidence justifies extraction.
- Risk: regional evidence is unavailable.
  Rollback: record the gap explicitly and block any claim of mainland-China/global acceptance; do not simulate geography.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-09-30 | Atomic claim+activation proposal | Maintainer requested progression after #626 merged. I286 is Closed; I287 remains Planned/Unclaimed; I285 Review and DEPENDENCY-003-A Active work are explicitly retained. #627 proposes I288 as a non-overlapping architecture-only Active/Claimed slice; both states remain ineffective until #627 merges. |\n| 2026-09-30 | Claim effective | Governance PR #627 merged as `6f651a28`; I288 architecture/evidence Work Slice is effective. Published selection-time dispositions above remain historical baseline and are not rewritten by later parallel-work state changes. |\n| 2026-09-30 | Code-truth audit | Confirmed current `WebSearchTool` owns routing; `tokio::select!` is first-completion; upstream `rust_websearch::search()` source identity is discarded; Tavily participates from env-key presence; Wikipedia is generic fallback. |\n| 2026-09-30 | Architecture candidate | Added ADR-085 and `docs/reference/I288-SEARCH-ARCHITECTURE-AUDIT-2026-09-30.md`; selected private SearchBackend + Talos SearchRouter inside `talos-tools`, no new crate, no startup probe/GeoIP routing, explicit premium usage and evidence-gated native providers. |\n| 2026-09-30 | Deterministic evidence | Added a test-only `tokio::select!` characterization proving a fast error can win before a later success; production `execute_search` remains unchanged. |

## Verification Evidence

- Implementation PR: #629.
- Source/code-truth audit: `docs/reference/I288-SEARCH-ARCHITECTURE-AUDIT-2026-09-30.md`.
- Architecture decision candidate: `docs/decisions/085-talos-owned-search-routing-boundary.md`.
- Deterministic routing characterization: `select_pattern_propagates_fast_error_before_later_success` in `crates/talos-tools/src/web_search.rs`.
- Exact-head focused/workspace/governance CI: pending implementation PR.

## Completion Evidence

- Completion Commit: pending; retain Review/Partial until an already-existing evidence commit can be cited.

## Variance And Residuals

- None at publication; later regional evidence gaps or provider feasibility residuals must be explicitly owned.

## Retrospective

- Pending.
