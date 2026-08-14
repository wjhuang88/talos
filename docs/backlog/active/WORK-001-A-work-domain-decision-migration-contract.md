# WORK-001-A: Canonical Work Domain Decision And Migration Contract

| Field | Value |
|---|---|
| Story ID | WORK-001-A |
| Type | Architecture / Migration Spike |
| Priority | P0 |
| Status | Ready — I196 planned; proposed claim is not yet effective |
| Parent Epic | WORK-001 |
| Source | DESKTOP-001 P0 prerequisite; GitHub Issue #29; three-track development baseline |
| Selected Iteration | I196 Planned |
| Depends On | RUNTIME-001 Complete; TODO-001 Complete; TODO-002 Complete; VALIDATION-001 Complete; current main inventory |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned — proposed P0 scope is defined below |
| Claimed At | Not applicable |
| Source Issue | #29 |
| Governance Claim PR | Pending |
| Authorization Mode | Not applicable until the Draft claim PR is finalized |
| Authorization Evidence | Not applicable until the Draft claim PR is finalized |
| Implementation PR | Not started |
| Last Updated | 2026-08-14 |
| Handoff / Release Condition | Finalize the proposed claim with its actual PR number, pass exact-head governance/CI checks and independent review, repeat merge-time CAS, and merge it to main before creating any P0 implementation branch. |

No ownership is effective from this branch or an open PR. The implementation branch must start
from the claim merge commit or a later current `main` commit.

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

- [ ] A current-state inventory maps Todo persistence, repositories, tools, commands, prompt and
      projection consumers to their authoritative crates and public/transitional surfaces.
- [ ] A Proposed ADR selects the canonical ownership/dependency boundary and states alternatives,
      compatibility constraints, evaluator-isolation invariants and reversal triggers.
- [ ] The migration contract defines identity/revision mapping, upgrade, compatibility window,
      failure behavior, rollback/downgrade and duplicate-authority prevention.
- [ ] P1 acceptance requires mechanical regression evidence for every existing `todo_*` tool
      contract and the TODO-001/TODO-002 behavior matrix; prose compatibility is insufficient.
- [ ] P1-P4 remain separate owners, iterations, claims, implementation PRs and exact-head reviews.
- [ ] `RUNTIME-001` and `SESSION-009` contain discoverable downstream references without changing
      their completed/refinement evidence or selecting SESSION-009.
- [ ] `git diff --name-only` proves no Rust, Cargo, lockfile, persistence schema or Desktop asset
      changed in P0.
- [ ] Both governance validators and `git diff --check` pass with zero warnings/errors.
- [ ] Independent exact-head architecture review finds no unresolved breaking or migration decision
      hidden for P1.

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
