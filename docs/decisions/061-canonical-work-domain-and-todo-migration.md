# ADR-061: Canonical Work Domain And Todo Migration Boundary

**Status**: Proposed

**Date**: 2026-08-18

**Owners**: WORK-001-A / I196

## Context

Talos already has one session-owned Todo domain in `talos-session`. It persists items and
dependency edges in SQLite, exposes permission-gated mutation tools, and supplies read-only CLI,
TUI and prompt projections. The Desktop direction requires Mission, Goal and WorkUnit state that
can later outlive one executor session and bind exact revisions to Completion Claims and Evaluation.

Creating a second Desktop Goal store would create competing planning authorities. The proposal
therefore describes one canonical Work Graph and a compatibility path for existing Todo behavior,
while explicitly leaving the concrete shared-crate name for a governed implementation slice.

## Decision

1. The long-term canonical planning and execution authority is one DAG-capable Work Graph. Mission,
   Goal and WorkUnit are projections/roles in that graph; a Desktop Goal store is prohibited.
2. P0 selects a **domain boundary**, not a new crate: the future canonical domain owns graph
   identity, containment, dependencies, revisions, acceptance criteria and later claim/evaluation
   subjects. A dedicated crate such as `talos-work` is a candidate for P1, but its name and module
   placement are not authorized by this ADR alone.
3. Until P1 is accepted, the existing `talos-session` Todo SQLite repository remains the sole
   durable planning authority. P0 adds no second store, schema, dual-write path, migration code or
   public API. Existing Todo data remains readable and mutation behavior remains unchanged.
4. P1 must adapt `/todo`, `todo_*`, prompt and read-only UI surfaces over the canonical domain while
   preserving their observable contracts. During a declared compatibility window, legacy names may
   remain adapters; they must not write an independent repository.
5. Stable identity is UUID-based for every durable graph node and edge subject. A node revision is
   a monotonic, persisted integer scoped to that node/aggregate. Any completion claim or evaluation
   binds to the exact mission/goal/work-unit revision tuple plus the inspected workspace/evidence
   subject revision. Locale and presentation projections are excluded from that identity.
6. Migration is expand/verify/cutover/rollback, never silent replacement: backup or snapshot the
   source, verify every Todo item and edge, cut over one authority, retain read compatibility for
   the published window, and fail closed on unknown schema, duplicate identity or lossy mapping.
   Rollback restores the pre-cutover authority without deleting source records. No irreversible
   migration is permitted until P1 acceptance proves the mechanical compatibility matrix.

## Dependency And Ownership Boundary

- The domain model must depend only on stable core protocol/value types and must not depend on CLI,
  TUI, Dashboard, Desktop, provider, evaluator or host-command crates.
- Session persistence and adapters may depend on the domain; clients project it read-only.
- `talos-runtime` may expose the accepted domain through a later additive SDK surface, but P0 does
  not change its public API or imply a 1.0 semver promise.
- SESSION-009 owns attach/detach/reconnect and multi-client custody. It consumes the domain
  contract; it does not create a second planning authority.
- P2-P4 separately own Completion/Evaluation models, evaluator runtime/evidence, and Mission gate
  projections. No P1-P4 behavior is authorized by this decision.

## Compatibility Invariants For P1

The following existing semantics are mandatory migration inputs: UUID identity, session ownership,
status and priority values, tags, dependency edges and cycle rejection, idempotent create, batch
mutation, permission-gated writes, query/filter behavior, short-ID/read-only projections, prompt
budgeting, and confirmed deletion. A compatibility adapter may reject an operation only with a
documented lossless reason; it may not silently drop data or bypass permissions.

## Alternatives Rejected

- A Desktop-only Goal database: creates two mutable sources of truth.
- Making `talos-session` the permanent Mission/Evaluation owner: couples longer-lived work to one
  session lifetime and prevents a clean SDK/evaluator boundary.
- Adding a generic workflow/scheduler framework in P0: exceeds the prerequisite decision scope.
- Immediate schema or crate creation: would hide unresolved migration and semver decisions in code.

## Validation And Reversal

P0 validation is documentation-only: the current-state inventory and migration matrix must be
reproducible from repository paths, both governance validators must pass, and changed paths must
contain no Rust, Cargo, lockfile, schema or Desktop implementation asset. P1 must revalidate the
decision against actual dependency graphs and a mechanical Todo compatibility suite. If P1 cannot
preserve a listed invariant without lossy conversion, reject or supersede this ADR and keep the
existing Todo authority unchanged.

## Consequences

The repository has one planning authority during transition and a clear path to a longer-lived
Work Graph. P1 carries the cost of an explicit adapter and migration matrix. Desktop, evaluator and
runtime implementation cannot claim real Mission/Goal behavior until the separately reviewed P0-P4
contracts and APIs are accepted.
