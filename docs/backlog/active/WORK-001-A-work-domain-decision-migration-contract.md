# WORK-001-A: Canonical Work Domain Decision And Migration Contract

| Field | Value |
|---|---|
| Story ID | WORK-001-A |
| Type | Architecture / Migration Spike |
| Priority | P0 |
| Status | Active — I196 P0 decision/migration contract implementation in progress |
| Parent Epic | WORK-001 |
| Source | DESKTOP-001 P0 prerequisite; GitHub Issue #29; three-track development baseline |
| Selected Iteration | I196 Active |
| Depends On | RUNTIME-001 Complete; TODO-001 Complete; TODO-002 Complete; VALIDATION-001 Complete; current main inventory |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline governance session 2026-08-14 |
| Work Slice | WORK-001-A / I196 P0 only: inventory current Todo/runtime/session/projection obligations; decide the canonical work-state ownership and dependency boundary; define stable identity/revision plus migration, compatibility, rollback and P1-P4 contracts. No Work Graph, Evaluation, Evaluator, persistence, public API, Rust/Cargo, Desktop, Dashboard, TUI product or later-child implementation. |
| Claimed At | 2026-08-14 |
| Source Issue | #29 |
| Governance Claim PR | #226 |
| Authorization Mode | Independent review |
| Authorization Evidence | PR #226 exact head `8d0cce3230b4030aab946fb0757da705dcfa4e26` passed CI `31781768908` and independent approval comment `5291072895`, then merged to `main` as `453d1fba97470639835468664c58397770db384c`. The claim is effective. Independent exact-head architecture review remains required for P0 decision acceptance; shared-account identity limits must be disclosed. |
| Implementation PR | Not started |
| Last Updated | 2026-08-18 |
| Handoff / Release Condition | P0 branch starts from current `main@b59912e3`; complete only the decision/documentation slice, then obtain independent exact-head architecture review, exact-head CI and owner-first closeout. No P1-P4 or implementation authority is transferred. |

The `Claimed` record above became effective when PR #226 merged as `453d1fba`. The P0
implementation branch starts from the later current `main@b59912e3` activation checkpoint.

## Activation Checkpoint — 2026-08-18

I196 is activated as the mainline P0 decision/documentation slice alongside the separately
authorized, GET-only/read-only Dashboard I195 lane; the existing non-overlap authorization is
recorded by Dashboard activation PR #288. The scopes do not overlap: I196 changes only
WORK-001-A decision and migration evidence, while I195 owns Dashboard presentation artifacts.
Current `main@b59912e36025088e4e3fa76b7b5b4e2aa7a1396c` inventory: I159/I160/I161/I162/I188
Complete; I164 Paused; I189 Planned/Claimed; I195 Active/Claimed; no other iteration is Review.
No Rust, Cargo, persistence, public API, Desktop, Dashboard or TUI implementation authority is
created by this activation.

## Identity / Goal / Value

Resolve the architectural and migration decisions that must be explicit before Talos changes its
public or persisted work-state behavior. A maintainer or reviewer should be able to determine the
canonical ownership boundary, existing Todo obligations, identity/revision rules, compatibility
path and rollback behavior without inferring design from a later code diff.

## Scope

- Inventory current Todo schema, repository APIs, dependency semantics, agent tools, slash/TUI
  surfaces, prompt integration, permission facets and `talos-conversation` projections.
- Decide which existing crate owns canonical work state or whether a later P1 may add one narrowly
  scoped shared crate; define the permitted dependency direction without adding the crate in P0.
- Define stable Mission, Goal and WorkUnit identity and revision semantics at the contract level
  required to bind later Completion Claims and Evaluations.
- Publish a migration, compatibility and rollback contract for existing Todo data and callers,
  including version detection, failure behavior, downgrade/rollback expectations and the point at
  which a second independently mutable Todo source is forbidden.
- State the P1-P4 input/output, acceptance and exclusion boundaries so later work cannot be hidden
  inside P0 or combined into one implementation PR.
- Record every public API, persistence or evaluator-isolation choice that must be settled before P1
  in a Proposed ADR; leave later implementation-specific choices with their owning child.

## Exclusions

- No Work Graph, Mission, Goal, WorkUnit, Completion Claim or Evaluation implementation.
- No new crate, schema migration, data rewrite, dual-write, persistence behavior or public API change.
- No Evaluator runtime, model call, permission role or Validation Service behavior change.
- No `talos-desktop`, GPUI, localization, native dependency, Dashboard or TUI product work.
- No Cargo manifest or `Cargo.lock` change.
- No acceptance or implementation of WORK-001-B through WORK-001-E.

## Dependencies And Existing Compatibility Baseline

- `TODO-001` preserves session-scoped SQLite Todo identity, status, priority, tags, dependencies,
  cycle rejection, agent-tool mutations, read-oriented surfaces, exports and prompt projection.
- `TODO-002` preserves idempotent create, batch create/update, short-ID display and confirmed delete.
- `RUNTIME-001` is the reusable pre-1.0 facade and must receive a discoverable downstream reference
  without changing its completed evidence.
- `VALIDATION-001` may later supply Evidence but cannot be treated as the Goal judgment authority.
- `SESSION-009` owns future attach/reconnect/multi-client behavior and receives a downstream
  compatibility reference; P0 does not select or implement it.
- ADR-008, ADR-024, ADR-042 and ADR-052 constrain storage, runtime and public-boundary decisions.

## Decision Links And Constraints

- Hard: Rust-first, no new `unsafe`, and no unreviewed native dependency.
- Hard: crate public APIs are semver-bound; breaking changes require an ADR and migration plan.
- Hard: existing persisted Todo data cannot be silently discarded, duplicated or made unreadable.
- Hard: write-capable compatibility tools remain permission-gated.
- Soft decision: the eventual canonical crate/module name and exact ownership boundary; P0 must
  select it from repository evidence rather than assume the proposal's candidate `talos-work` name.
- Assumption to validate: Mission/Goal lifetime requires ownership beyond one current session.
  P0 must state the evidence and reversal trigger rather than treating that assumption as fact.

## Runnable / Testable Deliverable

P0 is an explicit decision/infrastructure-only exception and claims no new user behavior. Its
deliverable is runnable and testable when a reviewer can execute the recorded inventory and
governance checks, compare the contract with the current Todo/runtime surfaces, and answer without
ambiguity:

1. where canonical work state may live and which dependency directions are forbidden;
2. which existing Todo data and behavior P1 must preserve mechanically;
3. how stable identity, revision, migration failure and rollback behave;
4. what P1-P4 each own and explicitly do not own;
5. which unresolved decision blocks P1, if any.

The implementation PR must prove it contains documentation/decision material only: no Rust source,
Cargo manifest, lockfile, schema or fixture behavior change.

## Acceptance For Technical / Governance Work

- [x] A current-state inventory maps Todo persistence, repositories, tools, commands, prompt and
      projection consumers to their authoritative crates and public/transitional surfaces.
- [x] A Proposed ADR selects the canonical ownership/dependency boundary and states alternatives,
      compatibility constraints, evaluator-isolation invariants and reversal triggers.
- [x] The migration contract defines identity/revision mapping, upgrade, compatibility window,
      failure behavior, rollback/downgrade and duplicate-authority prevention.
- [x] P1 acceptance requires mechanical regression evidence for every existing `todo_*` tool
      contract and the TODO-001/TODO-002 behavior matrix; prose compatibility is insufficient.
- [x] P1-P4 remain separate owners, iterations, claims, implementation PRs and exact-head reviews.
- [x] `RUNTIME-001` and `SESSION-009` contain discoverable downstream references without changing
      their completed/refinement evidence or selecting SESSION-009.
- [x] `git diff --name-only` proves no Rust, Cargo, lockfile, persistence schema or Desktop asset
      changed in P0.
- [x] Both governance validators and `git diff --check` pass with zero warnings/errors.
- [ ] Independent exact-head architecture review finds no unresolved breaking or migration decision
      hidden for P1.

## Execution Evidence — 2026-08-18

- Current-state inventory: `docs/reference/I196-WORK-001-A-CURRENT-STATE-MIGRATION-CONTRACT.md`.
- Proposed decision: `docs/decisions/061-canonical-work-domain-and-todo-migration.md`.
- Changed paths are documentation/governance only; no Rust, Cargo, lockfile, schema or Desktop asset.
- `scripts/validate_project_governance.sh .`: 0 warnings.
- `COLLABORATION_VALIDATION_BASE=origin/main bash scripts/validate_collaboration_claims.sh .`: 0 warnings.
- `git diff --check`: pass.
- Independent exact-head architecture review of the decision packet remains pending.

## Planned Validation

- Inventory searches over `crates/talos-session`, `crates/talos-runtime`,
  `crates/talos-conversation`, `crates/talos-cli` and `crates/talos-tui`.
- Existing focused Todo tests are recorded as unchanged-behavior baseline; P0 does not alter them.
- `scripts/validate_project_governance.sh .`
- `bash scripts/validate_collaboration_claims.sh .`
- `git diff --check`
- Changed-path assertion excluding `*.rs`, `Cargo.toml`, `Cargo.lock`, migrations and Desktop assets.

## State / Status Owners

- Story scope, acceptance and residuals: this file.
- Epic dependency order: `docs/backlog/active/WORK-001-goal-oriented-work-evaluation-foundation.md`.
- Iteration planning/execution: `docs/iterations/I196-work-domain-decision-migration-contract.md`.
- Directional source: `docs/backlog/active/DESKTOP-001-desktop-product-direction.md`.
- Current operating view: `docs/BOARD.md`.
- Compact selection view: `docs/backlog/PRODUCT-BACKLOG.md`.

## User-Facing Documentation

P0 changes no user-visible behavior. The implementation PR must update architecture/developer
documentation only and must not describe Work Graph, Evaluation, Delivery or Desktop as shipped.
P1 and later behavior-facing children own their corresponding README/usage changes.

## Required Reads

- `AGENTS.md`
- `docs/sop/AGENT-COLLABORATION.md`
- `docs/sop/START-ITERATION.md`
- `docs/sop/ITERATION-WORKFLOW.md`
- `docs/tasks/2026-08-13-three-track-development-baseline.md`
- `docs/proposals/talos-desktop-goal-oriented-workspace.md`, section 20
- `docs/backlog/active/WORK-001-goal-oriented-work-evaluation-foundation.md`
- `docs/backlog/active/DESKTOP-001-desktop-product-direction.md`
- `docs/backlog/active/RUNTIME-001-embeddable-agent-runtime-api.md`
- `docs/backlog/active/SESSION-009-multi-client-session-architecture.md`
- `docs/backlog/active/TODO-001-session-todo-list.md`
- `docs/backlog/active/TODO-002-todo-mutation-reliability.md`
- `docs/backlog/active/VALIDATION-001-internal-validation-service.md`
- `docs/decisions/008-sqlite-bundled-storage.md`
- `docs/decisions/024-embeddable-runtime-api-boundary.md`
- `docs/decisions/042-embedded-durable-runtime-session-boundary.md`
- `docs/decisions/052-sdk-publication-and-composition-boundary.md`

## Residual Destination

- WORK-001-B owns canonical Work Domain and Todo compatibility implementation after P0 is accepted.
- WORK-001-C/D/E own Completion/Evaluation state, evaluator runtime and Mission/projection closure.
- Existing I193/manifest/Board closeout wording drift remains a separate consistency correction and
  is not evidence for, or part of, this P0 claim.
