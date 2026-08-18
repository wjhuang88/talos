# I196 / WORK-001-A Current-State And Migration Contract

**Status**: P0 decision packet — implementation evidence pending

**Exact activation base**: `main@34b43ab9f8007329bf3cbf402bddf362255a68f9`

**Decision**: [ADR-061](../decisions/061-canonical-work-domain-and-todo-migration.md)

## Current Authority Inventory

| Surface | Current owner and evidence | P0 disposition |
|---|---|---|
| Todo durable records | `crates/talos-session/src/todo/repository.rs`; `TodoRepository` opens `todos.sqlite`, creates `todo_items` and `todo_dependencies`, and enforces session-scoped reads/writes | Sole current planning authority; no second store or schema change |
| Todo domain values | `crates/talos-session/src/todo/model.rs`; UUID item identity, session id, status, priority, timestamps, turn assignment and tags; dependency edges are UUID pairs | Preserve as P1 compatibility invariants; map to WorkUnit/edge without lossy conversion |
| Mutation tools | `crates/talos-session/src/todo/tools.rs` and `tool_contributions.rs`; create/update/delete/dependency/batch tools are registered per session | Keep permission-gated; later adapters must preserve idempotency, batch and cycle rejection |
| User read surfaces | `crates/talos-cli/src/todo_view.rs`, conversation slash parsing, and `crates/talos-tui/src/app.rs` Todo panel | Read-only projections during compatibility window; no independent client state |
| Prompt projection | `crates/talos-cli/src/mode_runtime.rs::format_session_todo_prompt` | Advisory, bounded active-item projection; never an authority or completion verdict |
| Session lifetime | `crates/talos-session/src/manager.rs::todo_repository`; repository path is under the sessions directory | Existing session ownership remains until P1 cutover; SESSION-009 owns later reconnect/attachment |
| Runtime SDK | `crates/talos-runtime/src/lib.rs`; public facade exposes provider/tools/permissions/session but no direct Todo repository | No P0 API change; later additive exposure requires semver and migration review |
| Validation evidence | `VALIDATION-001` shared internal validation service and repository governance validators | Evidence producer only; cannot judge Goal completion or replace evaluator authority |

## Required Compatibility Matrix

| Existing behavior | P1 preservation rule | Failure/rollback rule |
|---|---|---|
| UUID item identity and session scope | Same identity remains addressable; no cross-session lookup | Duplicate or ambiguous mapping fails closed and leaves source intact |
| `todo`, `in_progress`, `completed`, `blocked` and priority values | Map explicitly to WorkUnit status/priority with an unknown-value error | Never coerce unknown values to `todo` during migration |
| Tags, description, turn assignment and timestamps | Preserve values or record an explicit, reviewed lossless adapter rule | Lossy field conversion blocks cutover |
| Dependency edges and cycle rejection | Preserve edge direction and reject self/cyclic edges | Invalid graph blocks migration; source remains authoritative |
| Idempotent create and batch mutation | Repeated requests produce the same subject and batch semantics | Retry mismatch blocks cutover; no duplicate writes |
| Permission-gated write tools | Every adapter write traverses the existing permission pipeline | Missing permission context fails closed |
| Query/filter, short IDs and read-only projections | Keep deterministic filtering and projection compatibility | Projection mismatch is a compatibility failure, not a second store |
| Prompt budgeting | Keep bounded advisory projection and exclude completed items as current code does | Prompt truncation cannot alter canonical state |
| Confirmed deletion | Preserve explicit confirmation and dependency cleanup semantics | Unconfirmed or partial delete is rejected and rolled back |

## Migration Phases

1. **Inventory** — enumerate schema, fields, callers, tools, projections and tests; record the
   exact source revision.
2. **Design/verify** — define Work Graph identity/revision mapping and run a mechanical fixture
   matrix against every Todo invariant before changing persistence.
3. **Expand** — add only reviewed additive adapters/storage, with source reads still authoritative;
   no dual mutable writers.
4. **Cut over** — verify counts, identities, edges, revisions and projections; atomically select
   one writer and retain the compatibility read window.
5. **Rollback/retire** — on any verification failure restore the source authority and snapshot;
   retire legacy names only after a separately accepted compatibility closeout.

## P1–P4 Boundaries

- **P1**: canonical Work Graph and Todo compatibility only; no Completion Claim or evaluator verdict.
- **P2**: Completion Claim, Acceptance Criteria, Evaluation subject/report/finding/verdict and
  staleness; no independent evaluator runtime.
- **P3**: evaluator runtime, fresh context, evidence validation and executor/evaluator separation.
- **P4**: Mission final gate, UI-neutral projections and end-to-end closure; no Desktop renderer.

## Reproduction And Validation Commands

```text
rg --files crates/talos-session/src/todo crates/talos-runtime/src crates/talos-cli/src crates/talos-tui/src
rg -n "TodoRepository|todo_items|todo_dependencies|todo_" crates/talos-session/src crates/talos-cli/src crates/talos-conversation/src crates/talos-tui/src
scripts/validate_project_governance.sh .
COLLABORATION_VALIDATION_BASE=origin/main bash scripts/validate_collaboration_claims.sh .
git diff --name-only origin/main...HEAD
git diff --check
```

The changed-path assertion must report documentation/governance files only. P0 cannot claim that
the Work Graph, migration, evaluator or Desktop behavior is implemented.
