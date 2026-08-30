# WORK-001-B: Canonical Work Domain And Todo Compatibility

| Field | Value |
|---|---|
| Story ID | WORK-001-B |
| Type | Architecture / Domain Story |
| Parent Epic | WORK-001 |
| Priority | P0 |
| Status | Review / Claimed — stable local candidate; implementation PR not started |
| Source | GitHub Issue #29; WORK-001 P1; Desktop prerequisite chain §20.2 |
| Selected Iteration | I237 |
| Depends On | WORK-001-A / I196 Complete; ADR-061 Proposed boundary; RUNTIME-001; TODO-001; TODO-002; VALIDATION-001 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline governance session |
| Work Slice | WORK-001-B / I237 P1 only: implement one canonical Work Domain and adapt the existing session Todo surface to it, including identity, revision, status/priority/tags, dependency and cycle rules, idempotent and batch mutation, permission-gated writes, query/filter and short-ID projection. Preserve existing Todo data and behavior through a compatibility path; no Completion Claim/Evaluation, evaluator runtime, Mission gate, Desktop, Dashboard, `/auto`, release or publication work. |
| Claimed At | 2026-08-30 |
| Source Issue | #29 |
| Governance Claim PR | #440 |
| Authorization Mode | Independent review |
| Authorization Evidence | Claim PR #440 exact head `4e2bd58dc5d54cea2846a2756b56b46438debbc9` passed CI `33312960052` and independent governance approval, then merged to `main` as `3f6f036dfcce612ca88dc57e654b23aedf761678`; P0 completion `779a4c71` and ADR-061 remain the boundary basis. |
| Implementation PR | Not started |
| Last Updated | 2026-08-30 |
| Handoff / Release Condition | Start implementation only from the claim merge or later current `main`; keep P2-P4 blocked until this Story has implementation evidence and closeout. |

## Identity / Goal / Value

Provide one shared, testable work-state domain for future Mission, Goal and WorkUnit consumers while
keeping current Todo commands, tools, persistence and permission boundaries usable. This removes the
need for a second independently mutable Todo repository before later evaluation work.

## Scope

- Define the canonical in-process Work Domain for Mission/Goal/WorkUnit identity and containment.
- Preserve UUID stable identity and monotonic revision semantics.
- Represent status, priority, tags and dependency edges with deterministic cycle rejection.
- Provide idempotent create and batch mutation operations while retaining existing single-item shapes.
- Route all write-capable mutations through the existing permission pipeline.
- Provide query/filter and short-ID read-only projections, prompt integration and confirmed delete.
- Adapt existing `talos-session` Todo persistence/surface to the canonical domain; do not create a
  second independently mutable Todo repository.
- Provide migration/compatibility and rollback behavior for existing Todo records and fixtures.

## Exclusions

- No Completion Claim, Evaluation, verdict, evaluator runtime or Mission final gate (P2-P4).
- No Desktop, GPUI, Dashboard, localization, `/auto`, sandbox-policy expansion or session rewrite.
- No release, version, tag, crates.io publication or unrelated runtime refactor.
- No destructive migration or removal of legacy Todo records without explicit compatibility evidence.

## Dependencies And Decision Constraints

- `docs/decisions/061-canonical-work-domain-and-todo-migration.md` governs the shared boundary and
  requires one source of truth plus a reversible compatibility path.
- `RUNTIME-001` remains a pre-1.0 facade; do not imply stable 1.0 SDK guarantees.
- `TODO-001` and `TODO-002` are shipped behavior baselines; preserve their permission, idempotency,
  batch, cycle-rejection, query and confirmed-delete contracts.
- `VALIDATION-001` produces evidence but is not a Goal completion authority.
- Public API or persistent-schema breaks require a decision record and migration plan.

## Required Reads

- `docs/proposals/talos-desktop-goal-oriented-workspace.md` §20.2
- `docs/tasks/2026-08-13-three-track-development-baseline.md`
- `docs/backlog/active/WORK-001-goal-oriented-work-evaluation-foundation.md`
- `docs/backlog/active/WORK-001-A-work-domain-decision-migration-contract.md`
- `docs/decisions/061-canonical-work-domain-and-todo-migration.md`
- `docs/backlog/active/RUNTIME-001-embeddable-agent-runtime-api.md`
- `docs/backlog/active/SESSION-009-multi-client-session-architecture.md`
- `docs/backlog/active/TODO-001-session-todo-list.md`
- `docs/backlog/active/TODO-002-todo-mutation-reliability.md`
- `docs/backlog/active/VALIDATION-001-internal-validation-service.md`
- `docs/reference/WORK-DOMAIN-TODO-COMPATIBILITY.md`

## Acceptance For Behavior

- Given existing Todo records, when the compatibility surface loads them, then identity, status,
  priority, tags and dependencies remain observable without a second mutable repository.
- Given a dependency edge that closes a cycle, when it is submitted, then the mutation is rejected
  atomically and the prior revision remains unchanged.
- Given a retry or duplicate create, when the same effective request is submitted, then it is
  idempotent and does not create a second WorkUnit/Todo record.
- Given a write-capable mutation, when permission evaluation denies or fails, then no durable state
  changes; confirmed delete remains explicit.
- Given a query/filter or short-ID projection, when a caller requests it, then read-only results are
  deterministic and preserve enough identity for follow-up operations.

## Acceptance For Technical Work

- [ ] Canonical domain and Todo adapter have focused unit/integration tests for identity, revision,
      dependency/cycle, idempotency, batch mutation, permission denial and compatibility fixtures.
- [ ] Existing Todo tests remain green and no second independently mutable Todo repository exists.
- [ ] Migration/rollback and public API compatibility evidence is documented.
- [ ] User/API documentation explains the preserved Todo surface and canonical Work Domain boundary.
- [ ] Locked focused/full validation, governance validators and `git diff --check` pass at the
      implementation candidate exact head.
- [ ] Independent exact-head review and merge-time CAS are recorded before completion.

## State / Status Owners

- Story scope, claim and completion: this owner document.
- Parent dependency order: `WORK-001-goal-oriented-work-evaluation-foundation.md`.
- Iteration execution: `docs/iterations/I237-work-domain-and-todo-compatibility.md`.
- Derived operating views: Board, Product Backlog, iterations README and manifest only.

## Residual Destination

Any unresolved migration, public API or compatibility issue remains here or receives a separately
governed child; do not move P2/P3/P4 into this Story.

## Local Convergence Checkpoint (2026-08-30)

The implementation candidate currently adds a storage-neutral `talos_core::work` domain with typed
Mission/Goal/WorkUnit roles, UUID identities, validated DAG snapshots and stable edge identities.
The existing Todo SQLite authority remains the only durable writer; its compatibility adapter adds
an additive, version-checked revision/edge migration, deterministic read projections, checked
revision updates, atomic batch mutation, and fail-closed validation for unknown values, orphan
edges and duplicate identities. Focused core/session tests pass locally. Full workspace validation,
user/API documentation, exact-head CI and independent implementation review remain pending; this
checkpoint is not completion evidence and does not modify the Published Baseline.
