# Work Domain And Todo Compatibility

Talos exposes the canonical, storage-neutral Work Domain from `talos_core::work`. A `WorkGraph`
contains typed Mission, Goal and WorkUnit nodes plus UUID-identified dependency edges. Node and edge
revisions are persisted monotonic integers. Graph construction rejects duplicate identities,
missing endpoints, invalid containment, self-dependencies and cycles.

## Compatibility Boundary

The existing session Todo database remains the only durable authority during the compatibility
window. It is not mirrored or dual-written into another repository. Existing `/todo` commands,
`todo_*` tools, prompt projection, short IDs, query filters and confirmed deletion continue using
that authority. `TodoRepository::load_work_graph` reads nodes and edges in one SQLite snapshot and
returns the validated canonical projection.

The public serialized `TodoItem` shape is unchanged. Canonical revisions and edge identities stay
inside the storage adapter and appear through Work Domain projections, so existing Todo JSON and
Rust callers do not acquire a required field.

## Migration And Rollback

Opening a legacy Todo database performs an additive, fail-closed migration:

1. Recognize only the published legacy or current column set.
2. Validate UUIDs, status, priority, timestamps, tags, edge endpoints, duplicate identities/titles
   and graph acyclicity before changing source records.
3. Create a sibling `todos.sqlite.pre-work-v1.bak` SQLite backup.
4. In one immediate transaction, add node revisions and edge identity/revision columns, derive
   stable edge UUIDs, validate the resulting current schema, and advance `user_version`.
5. Roll the transaction back on any error. The backup is retained for explicit operator rollback;
   source rows are never deleted by migration.

An unknown/partial/newer schema, invalid value, orphan edge, duplicate identity/title, invalid
graph or pre-existing backup blocks migration. Talos does not coerce an unknown status to `todo` or
an unknown priority to `medium`.

## Mutation Semantics

- Create is idempotent by exact `(session_id, title)` identity and serialized with an immediate
  SQLite transaction; retries return the existing Todo/WorkUnit.
- Create/update batches are all-or-nothing transactions.
- Node updates use revision compare-and-swap and fail on concurrent revision movement.
- Adding/removing a dependency is serialized with cycle validation. A real edge change advances
  the dependent WorkUnit revision; duplicate add or absent remove is a no-op.
- All model-facing write tools retain the existing permission facets. A denied or failed
  permission decision prevents tool execution, so the repository is not mutated.
- Query, short-ID, prompt and UI projections are read-only and cannot change canonical state.

This boundary does not implement Completion Claims, Evaluation, evaluator runtime, Mission final
delivery gating, Desktop/Dashboard UI, `/auto`, or release behavior. Those remain WORK-001 P2-P4 or
separately governed work.
