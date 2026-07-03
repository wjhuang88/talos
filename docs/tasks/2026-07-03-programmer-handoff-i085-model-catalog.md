# Programmer Handoff: I085 Model Catalog Modernization

> Status: Ready for assignment
> Created: 2026-07-03
> Applies to:
> [Four-Month Product Hardening Plan](2026-07-03-four-month-product-hardening-plan.md)
> Current iteration shell: [I085](../iterations/I085-model-catalog-modernization.md)
> Owner backlog: [MC-001](../backlog/active/MC-001-model-catalog-modernization.md)

## Purpose

This handoff tells implementation programmers how to take the first I085 model catalog work
without expanding provider execution, permission, release, or interactive UI scope.

The current delegated goal is Stage 1 only: create the catalog data layer and resolver foundation
for H100-H101 / MC100-MC103. The `/model` and `/connect` interactive changes are Stage 2 and must
wait until the resolver precedence path is implemented and tested.

## Current Baseline

- I085 is Planned and is the next iteration candidate after `v0.2.2` closeout.
- `MC-001` is Planned and owns the catalog modernization epic.
- `crates/talos-config/src/model.rs` currently owns `ModelMetadata`, `builtin_models()`, and the
  broken `import_models_dev()` path.
- `crates/talos-config/src/config.rs` owns current model resolution and provider authentication
  checks.
- `crates/talos-config/src/models.toml` is the committed built-in catalog fallback.
- `crates/talos-cli/src/model_lifecycle.rs` and `crates/talos-tui/src/state.rs` own the current
  interactive picker path. Do not change picker behavior in Stage 1 unless needed for compile-only
  type moves.

## Delegation Boundary

Stage 1 is delegable with normal senior review because it is Rust data plumbing and local storage.
It still needs explicit review at the catalog/resolver boundary because it touches SQLite and model
selection semantics.

| Stage | Assignment | Delegable Now | Deliverable | Gate |
|---|---|---:|---|---|
| S1-A | MC100 shared types and `talos-models` crate shell | Yes | Shared catalog types outside `talos-config`; crate builds with focused tests | No UI or command behavior change |
| S1-B | MC100 SQLite store and migrations | Yes | Versioned providers/models/pricing/meta tables with safe open/query behavior | DB corruption/incompatibility falls back through caller |
| S1-C | MC101 models.dev import/fetch parser | Yes | Parser handles object-keyed models.dev input and layered provider/model/pricing data | No startup network dependency |
| S1-D | MC102 deterministic built-in refresh | Yes | `BUILD_MODELS=1` path can regenerate `models.toml`; normal builds stay offline | No network in normal build |
| S1-E | MC103 catalog-aware resolver | Yes, review required | user config > catalog.db > builtin TOML > conservative fallback, with tests | Must pass before Stage 2 starts |
| S2-A | MC104 `/connect` provider setup | Not yet | Full provider list, credential and optional endpoint merge | Requires S1-E evidence |
| S2-B | MC105-MC106 `/model` picker refactor and group filtering | Not yet | Usable-model picker grouped by provider | Requires S1-E evidence |
| S2-C | MC107 docs and closeout | Not yet | Docs, validation, residuals | Requires S2 validation |

## Non-Negotiable Rules

- Do not activate Stage 2 until S1-E resolver precedence tests pass.
- Do not add a startup-time network dependency.
- Do not fetch models.dev during normal `cargo build`; only `BUILD_MODELS=1` may refresh the
  committed built-in data.
- Do not change provider protocol behavior, `LanguageModel`, reasoning behavior, or stream parsing.
- Do not introduce Node, Python, arbitrary native bindings, or new provider SDKs.
- Do not add `unsafe`.
- Do not print, log, serialize into fixtures, or commit API keys.
- Do not weaken ADR-013: provider config remains schema/config only, with no dynamic provider code
  loading.
- Do not make `talos-config` implicitly open SQLite. CLI/TUI/runtime callers should pass an
  optional catalog/resolver handle.
- Do not overwrite unrelated provider fields when later Stage 2 config merge work begins.
- Do not publish crates, tag releases, push branches, or change `publish = false`.
- Update owner docs before `docs/BOARD.md`.

## Required Reads Before Starting

Read these in order:

1. `AGENTS.md`
2. `docs/sop/START-ITERATION.md`
3. `docs/sop/ITERATION-WORKFLOW.md`
4. `docs/tasks/2026-07-03-four-month-product-hardening-plan.md`
5. `docs/tasks/2026-07-03-programmer-handoff-i085-model-catalog.md`
6. `docs/iterations/I085-model-catalog-modernization.md`
7. `docs/backlog/active/MC-001-model-catalog-modernization.md`
8. `docs/backlog/active/MODEL-001-model-catalog-and-reasoning.md`
9. `docs/backlog/active/MODEL-004-catalog-runtime-integration.md`
10. `docs/backlog/active/MODEL-005-interactive-model-selection.md`
11. `docs/backlog/active/CONF-002-model-onboarding.md`
12. `docs/decisions/013-provider-config-schema-boundary.md`
13. `docs/decisions/008-sqlite-bundled-storage.md`
14. `docs/decisions/023-inline-api-key-boundary.md`
15. `crates/talos-config/src/model.rs`
16. `crates/talos-config/src/config.rs`
17. `crates/talos-config/src/builtin.rs`
18. `crates/talos-cli/src/model_lifecycle.rs`
19. `crates/talos-tui/src/state.rs`
20. `crates/talos-conversation/src/types.rs`
21. `crates/talos-conversation/src/command_registry.rs`

## Assignment Details

### S1-A Shared Types And Crate Shell

Expected work:

- Move or introduce shared catalog/provider/model types at a non-cyclic boundary, preferably
  `talos-core::model`.
- Preserve serde and schemars coverage for config/protocol-facing types.
- Add `crates/talos-models` to the workspace.
- Keep public APIs documented with `///`.
- Keep `talos-core` dependency-free.

Validation:

```sh
cargo fmt --all -- --check
cargo check --workspace
cargo test -p talos-core
cargo test -p talos-models
```

### S1-B SQLite Catalog Store

Expected work:

- Add SQLite-backed `ModelCatalog` APIs for open/create, migrations, seed, query, and search.
- Use `rusqlite/bundled` only under the accepted SQLite boundary.
- Store explicit schema version metadata.
- Return errors to callers; do not panic or exit on DB errors.
- Include tests for migration, CRUD, query, missing rows, and invalid/corrupt DB fallback behavior
  at the integration boundary.

Validation:

```sh
cargo test -p talos-models
cargo clippy -p talos-models -- -D warnings
```

### S1-C models.dev Import Parser

Expected work:

- Replace the array-only assumption in the existing import path with parsing for object-keyed
  models.dev input.
- Preserve unknown fields and future format tolerance where practical by ignoring fields Talos does
  not use.
- Parse provider identity, model identity, context/output limits, capability booleans, source
  provenance, and pricing only when present.
- Add fixture tests using checked-in small JSON samples; do not depend on live network in tests.

Validation:

```sh
cargo test -p talos-models import
cargo test -p talos-config model
```

### S1-D Gated Built-In Refresh

Expected work:

- Add or update a deterministic refresh path so `BUILD_MODELS=1 cargo build` can regenerate
  `crates/talos-config/src/models.toml` from models.dev data.
- Normal builds must not require network access.
- Output ordering and formatting must be stable enough for review.
- Do not invent pricing when upstream data is absent.

Validation:

```sh
cargo build
BUILD_MODELS=1 cargo build
git diff -- crates/talos-config/src/models.toml
```

The `BUILD_MODELS=1` command may require network approval in restricted environments. If network is
unavailable, record that as validation not run instead of replacing the live fetch with a hidden
host script.

### S1-E Catalog-Aware Resolver

Expected work:

- Implement a resolver path that uses this precedence:
  1. explicit user `config.toml`;
  2. `catalog.db` when available and readable;
  3. committed built-in `models.toml`;
  4. conservative fallback for runtime safety.
- Keep `talos-config` from opening SQLite implicitly.
- Add tests proving catalog DB failure does not block startup or model resolution.
- Preserve current model selection behavior until Stage 2 changes the interactive picker.

Validation:

```sh
cargo test -p talos-models
cargo test -p talos-config model
cargo test -p talos-cli model
cargo check --workspace
```

## Escalate Before Proceeding

Stop and report instead of guessing if:

- The implementation needs a new runtime dependency beyond accepted SQLite usage.
- The work would change provider request/response semantics.
- The work would make startup or normal build require network.
- The work would add or expose secrets in config display, logs, tests, or fixtures.
- The work requires modifying permission, sandbox, plugin, dashboard, browser, or release behavior.
- The work requires breaking public APIs without a migration plan.
- Resolver precedence conflicts with ADR-013 or the existing config schema.
- Stage 2 UI work appears necessary before Stage 1 validation passes.

## Handoff Prompt

Copy this prompt when assigning Stage 1 work to a programmer:

```text
You are working in the Talos repository on I085 Model Catalog Modernization, Stage 1 only.

Read these first, in order:
1. AGENTS.md
2. docs/sop/START-ITERATION.md
3. docs/sop/ITERATION-WORKFLOW.md
4. docs/tasks/2026-07-03-four-month-product-hardening-plan.md
5. docs/tasks/2026-07-03-programmer-handoff-i085-model-catalog.md
6. docs/iterations/I085-model-catalog-modernization.md
7. docs/backlog/active/MC-001-model-catalog-modernization.md
8. docs/backlog/active/MODEL-001-model-catalog-and-reasoning.md
9. docs/backlog/active/MODEL-004-catalog-runtime-integration.md
10. docs/backlog/active/MODEL-005-interactive-model-selection.md
11. docs/backlog/active/CONF-002-model-onboarding.md
12. docs/decisions/013-provider-config-schema-boundary.md
13. docs/decisions/008-sqlite-bundled-storage.md
14. docs/decisions/023-inline-api-key-boundary.md

Your assignment is: <S1-A/S1-B/S1-C/S1-D/S1-E>.

Hard constraints:
- Implement only Stage 1 catalog plumbing and resolver work.
- Do not change /model or /connect interactive behavior except for compile-only type moves.
- Do not add startup network access.
- Do not fetch models.dev during normal cargo build; only BUILD_MODELS=1 may refresh built-ins.
- Do not change provider protocol behavior, reasoning behavior, LanguageModel, stream parsing, plugin behavior, permission behavior, dashboard behavior, or release behavior.
- Do not add unsafe.
- Do not expose secrets.
- Keep talos-config from implicitly opening SQLite.
- Update owner docs before docs/BOARD.md.

Expected output:
- Implement the assigned slice only.
- Add focused tests for the assigned behavior.
- Record validation evidence in the owner doc or handoff note.
- Run the validation commands listed for the assignment.
- Leave Stage 2 (/model and /connect UX) untouched until resolver precedence tests pass.

Stop and report instead of guessing if:
- A new dependency, API break, network requirement, permission boundary change, or provider protocol change seems necessary.
- Normal build cannot stay offline.
- Corrupt or incompatible catalog DBs cannot degrade to built-in TOML fallback.
```

## Completion Report Template

Use this in the PR/commit summary or handoff note:

```text
Assignment:
Files changed:
Behavior/API changed:
Catalog/resolver precedence changed:
Network/build behavior changed:
Secrets handling impact:
Validation run:
Validation not run:
Residual work:
Blocked items:
```

## Recovery Instructions

If the delegated work is interrupted:

1. Run `git status --short`.
2. Re-read this handoff, I085, and MC-001.
3. Identify the current S1 assignment and whether any Stage 2 files were touched.
4. Run `cargo fmt --all -- --check`, targeted crate tests, and `git diff --check` if code changed.
5. Append a checkpoint to the handoff note or I085 execution log before transferring work.
