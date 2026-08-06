# Iteration I171: Workspace Architecture Rebaseline

> Document status: Planned
> Published plan date: 2026-08-06
> Planned objective: re-audit the current v0.7.0 workspace and produce a complete, reproducible,
> bounded remediation queue without changing product/runtime/public API behavior.
> Baseline rule: once committed, preserve this target; production remediation uses later iteration
> IDs and separately claimed stories.
> MVP deliverable: an operator can reproduce the current architecture measurements and inspect one
> finding register that accounts for every crate, material production root/seam, prior finding, and
> required remediation owner.
> Infrastructure-only exception: this iteration produces architecture evidence and deterministic
> validation; it does not claim new user behavior.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 active architecture session 2026-08-06 |
| Work Slice | Current v0.7.0 whole-workspace architecture rebaseline, test/audit-harness-only baseline repair, finding reconciliation, and bounded remediation owner creation; no production refactor. |
| Claimed At | Not applicable |
| Source Issue | None |
| Governance Claim PR | Pending |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | User requested a complete governance-based whole-project architecture audit and behavior-preserving remediation on 2026-08-06; claim becomes effective only after exact-head CI, both governance validators, merge-time CAS, and recorded single-maintainer reason. |
| Implementation PR | Not started |
| Last Updated | 2026-08-06 |
| Handoff / Release Condition | Release if the rebaseline cannot be reproduced or if scope requires a production, public-API, permission, sandbox, or process-hardening change. |

## Closure Ledger

| Dimension | Record |
|---|---|
| Requested outcome | Establish current architecture truth and an executable route to close every validated architecture issue without behavior change. |
| Artifacts to create/update | ARCH-034-D, I171, August audit/report register, justified audit harness, ARCH-034 owners, iteration/backlog/Board/manifest mirrors. |
| Existing assets to preserve | I144 published baseline, July audit artifacts, I158 evidence, ADRs, public APIs, current runtime behavior, user changes (none present at planning). |
| State/status owners | ARCH-034 parent/D/B/C/R01, I171, `docs/iterations/README.md`, Product Backlog, Board, manifest; later Rxx owners for production remediation. |
| Validation required | Locked metadata/fmt/check/all-target Clippy/workspace tests, governance/claim/scale validators, deterministic audit reproduction, `git diff --check`. |
| Evidence/uncertainty | Current commands and source are facts; architectural conclusions remain inferences until traced and counterevidence reviewed; old v0.4 measurements are historical only. |
| Residual-work destination | Separately claimed ARCH-034-Rxx stories and later iterations; security-sensitive work requires independent review. |

## Startup Contract

| Field | Approved baseline |
|---|---|
| Outcome | Finish ARCH-034-D and leave every validated production remediation as an owned, bounded, dependency-ordered story. |
| In scope | Full read-only source/governance analysis; audit/report/harness changes; test/audit-fixture-only repair needed for deterministic baseline; owner synchronization. |
| Out of scope | Production refactor; behavior/API/dependency/security-policy changes; release/tag/publish/deploy; destructive actions. |
| Dependencies | Current `main`; pinned Rust 1.97.0; Cargo.lock; I158 retained in Review; I159-I162 retained Blocked. |
| Artifacts/state owners | Closure Ledger paths above. |
| Validation/acceptance | ARCH-034-D acceptance and validation commands. |
| Branch/worktree/checkpoints | Claim branch/PR first; implementation from claim merge or later main; one worktree; checkpoint after evidence, findings, and owner-sync phases. |
| Allowed external actions | Read GitHub state; create/push governance and implementation branches/PRs; use single-maintainer merge only after exact-head gates. No release/deploy/publish. |
| Destructive/irreversible actions | None. Existing recovery PRs/branches and tags remain immutable. |
| Time/cost/resource limits | No paid services or new dependencies; use local deterministic checks; retry a failing validation once before classifying it. |
| Failure/retry/fallback | Reproduce once; repair only test/audit harness defects in this slice; otherwise record blocker and create a bounded owner. |
| Default ambiguity decisions | Prefer behavior-preserving facade/module extraction; treat LOC as locator; defer security/API choices to ADR/review; do not broaden scope silently. |
| Residual destination | ARCH-034-Rxx child owner plus later iteration. |

## Ordered Task Items

| ID | Task | Expected Output | Depends On | Completion Gate | Fallback | Status |
|---|---|---|---|---|---|---|
| A1 | Establish current baseline | Clean-state, non-terminal inventory, graph, scale, locked validation results | Effective claim | Commands recorded with true outcomes | Record reproducible blocker | Planned |
| A2 | Measure and trace architecture | Current crate/root/hotspot/extension/native-boundary evidence | A1 | Every ARCH-034-D audit dimension covered | Mark evidence unknown and add validation task | Planned |
| A3 | Reconcile findings | August report plus machine-readable register | A2 | Every prior/new finding has disposition and owner | Keep unresolved finding Proposed | Planned |
| A4 | Repair deterministic audit baseline | Only justified test/audit-harness defects fixed | A1/A2 | Full required validation passes without production behavior change | Separate bounded blocker story | Planned |
| A5 | Synchronize governance | Parent/children, iteration index, backlog, Board, manifest consistent | A3/A4 | Both governance validators and semantic owner audit pass | Retain Review with exact residual | Planned |

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| ARCH-034-D | ARCH-034 | Ready | ARCH-034-A history preserved; current v0.7.0 source available | Reproducible current-state audit and bounded remediation queue |

### Non-Terminal Iteration Inventory And Disposition

| Iteration | Current state | I171 disposition |
|---|---|---|
| I158 | Review | Retain in Review. Re-evaluate R01 exceptions/documentation during finding reconciliation; no production continuation without a new claim. |
| I159 | Blocked | Continue Blocked on I158 Complete/Paused and recorded TUI-037 disposition. |
| I160 | Blocked | Continue Blocked on I159 Complete. |
| I161 | Blocked | Continue Blocked on I160 plus independent sandbox/security review. |
| I162 | Blocked | Continue Blocked on I161 and explicit publication-readiness authorization. |

I171 is a maintainer-requested architecture evidence/replanning interruption. It does not activate
or bypass any blocked dependency chain and does not claim I158 production residuals.

### Planned Validation

Use the exact locked commands and governance gates in ARCH-034-D. Architecture evidence must be
reproducible from current source; a narrow passing check cannot substitute for the workspace gate.

### Documentation To Update

- `docs/reference/ARCHITECTURE.md` only where current facts are stale.
- ARCH-034 finding/owner documents, iteration index, Product Backlog, Board, and manifest.
- No README/site feature update is expected because behavior is unchanged; any discovered
  user-facing drift becomes an owned residual rather than an invented feature claim.

### Risks And Rollback

- Risk: line counts cause arbitrary splits. Mitigation: responsibility/change-reason and
  counterevidence are mandatory.
- Risk: audit scope hides behavior changes. Mitigation: no production edits; test/harness-only
  repair must have explicit proof.
- Rollback: revert the audit/harness commit; production artifacts remain untouched.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-06 | Planning | Preliminary read-only evidence: current main is v0.7.0; governance/claim/scale/fmt/check/all-target Clippy pass; workspace test repeatedly exposes a provider-discovery unreachable-endpoint fixture timeout; no production edit made. |

## Verification Evidence

- Pending effective claim and execution.

## Completion Evidence

- Completion Commit: not assigned; iteration is Planned.

## Variance And Residuals

- Production remediation remains outside I171 and must use later claimed iterations.

## Retrospective

- Pending execution.
