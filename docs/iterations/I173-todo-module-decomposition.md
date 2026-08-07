# Iteration I173: Todo Module Decomposition

> Document status: Active
> Published plan date: 2026-08-07
> Planned objective: decompose `talos-session/src/todo.rs` into private responsibility modules without changing Todo behavior or public paths.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: existing Todo repository and nine tool adapters compile through the same public API while model, repository, formatting, and adapter ownership is separated internally.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 architecture session 2026-08-07 |
| Work Slice | Move existing Todo domain types, SQLite repository logic, display formatting, and nine AgentTool adapters into private submodules behind the current `todo` facade; preserve every public path, schema, SQL/query order, idempotency/dependency rule, tool name, contribution, permission facet, and output string. |
| Claimed At | 2026-08-07 |
| Source Issue | None |
| Governance Claim PR | #148 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | No independent reviewer is currently available; exact-head CI, both governance validators, merge-time CAS, and no blocking review feedback are required. |
| Implementation PR | Not started |
| Last Updated | 2026-08-07 |
| Handoff / Release Condition | Claim merge precedes implementation branch; release if public-path, SQL, serialization, permission, or output equivalence cannot be proven. |

## Non-Terminal Inventory At Selection

| Item | State | Disposition |
|---|---|---|
| I159 | Blocked | Unchanged; TUI-037 disposition remains required. |
| I160 | Blocked | Unchanged; requires I159 Complete. |
| I161 | Blocked | Unchanged; requires I160 and independent security review. |
| I162 | Blocked | Unchanged; requires I161 and publication authorization. |
| I164 | Paused | Historical superseded target; no activation. |
| Recovery PRs #120/#121 | Open archival evidence | Immutable; no implementation authority. |
| ARCH-034-R04 | Refinement | Native/panic/unsafe boundary remains excluded pending security review. |
| ARCH-034-R05..R11 | Ready / unclaimed | Retained for later independent claims; no overlap with Todo ownership. |

No Active or Review iteration overlaps this work. I172/R02 is Complete with Completion Commit
`4084138dc0652d3200045847d42518d9ecb66231`.

## Scope

- Keep `todo` as the public facade and preserve all existing `talos_session` exports.
- Move domain/config/input types and defaults into a private model module.
- Move SQLite repository, row mapping, normalization, and dependency validation into a private repository module.
- Move exact user/model-visible Todo formatting into a private formatting module.
- Move the nine `AgentTool` adapters and shared permission facet into a private tools module.
- Add source-layout or compile-path regression evidence for directionality and facade ownership.

## Non-Goals

- No schema migration, SQL/query change, transaction change, new abstraction, crate split, tool rename,
  permission change, output wording change, new command, or feature.
- No native/sandbox/process work and no changes to R04.

## Acceptance

- Repository code imports no tool adapter type; adapters consume the repository facade.
- Existing public imports, serde/schemars schemas, SQL behavior, idempotency and dependency validation remain identical.
- All Todo tools retain names, schemas, permission facets, contribution order, and output text.
- Focused Todo repository/batch/dependency/contribution/permission tests and full locked workspace validation pass.

## Planned Validation

- `cargo fmt --all -- --check`
- `cargo check --workspace --locked`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked --no-fail-fast`
- `scripts/validate_project_governance.sh .`
- `bash scripts/validate_collaboration_claims.sh .`
- `git diff --check`
- Exact-head Unix/Windows CI and rebuilt CLI smoke.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-07 | Activation | Claim PR #148 passed exact-head CI run `31140533666`; merge-time CAS confirmed head `9700e196` against base `cc743601`, and the claim merged at `e9836ddf`. The implementation branch starts from that effective claim. |
| 2026-08-07 | Implementation | Moved the existing Todo model, SQLite repository, formatted output, nine tool adapters, and unit tests into private responsibility modules behind the unchanged `todo` facade. Added source-layout and compile-path regression coverage. |

## Verification Evidence

- `cargo test -p talos-session --locked --no-fail-fast`: passed (171 unit tests, 23 integration tests, and doc tests, including two I173 regression tests).
- `cargo fmt --all -- --check`, `cargo check --workspace --locked`, and `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo test --workspace --locked --no-fail-fast`: passed outside the workspace sandbox; the sandboxed attempt failed only where local listeners, `sandbox-exec`, or restricted filesystem operations were denied.
- `./scripts/release_preflight.sh`: passed outside the workspace sandbox.
- `scripts/validate_project_governance.sh .`, `bash scripts/validate_collaboration_claims.sh .`, and `git diff --check`: passed.
- Exact-head implementation CI and rebuilt CLI smoke remain pending the implementation PR.

## Completion Evidence

- Completion Commit: not assigned; retain Planned/Review until implementation evidence exists.

## Residuals

- New Todo behavior remains a separate product story.
- R04 and R05–R11 remain separately owned and independently claimable after I173 closes.
