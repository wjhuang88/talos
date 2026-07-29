# Iteration I157: Provider Removal And Credential Clear

> Document status: Complete
> Published plan date: 2026-07-26
> Planned objective: A user can remove a provider entry or clear one credential through `talos config unset ... --confirm` without hand-editing TOML.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: A user can remove a provider entry or clear one credential through `talos config unset ... --confirm` without hand-editing TOML.
> Activation rule: this iteration is not implementation authority until its Selected Story is Ready and the activation gate is recorded.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `MODEL-010` | `None` | `Ready` | `I156` Complete; provider config baseline unchanged | A user can remove a provider entry or clear one credential through `talos config unset ... --confirm` without hand-editing TOML. |

### Start Here

Read in order:

1. `AGENTS.md`
2. `docs/sop/START-ITERATION.md`
3. `docs/sop/ITERATION-WORKFLOW.md`
4. `docs/sop/CHANGE-CONTROL.md`
5. `docs/tasks/2026-07-26-v0.6-runtime-productization-program.md`
6. this iteration
7. the selected Story
8. governing ADRs/specifications in Required Reads
9. the exact source files named by the Story

The selected Story owns scope and acceptance. This iteration owns activation, execution evidence,
variance, and completion state.

### Authorized Scope

- add the CLI-only `ConfigCommand::Unset` surface;
- implement dotted-key provider entry removal and API-key clear;
- enforce `--confirm`;
- perform one atomic write on success;
- prove active-provider removal remains picker-recoverable;
- update EN/zh-CN/config reference docs.

### Forbidden Changes

- no TUI `/connect` delete action;
- no bulk/wildcard deletion;
- no environment variable deletion;
- no second secret store;
- no model-catalog change;
- no unrelated config refactor;
- no tag/release.

### Implementation Slices

1. **Baseline**
   - inspect current code and record current behavior;
   - run focused baseline tests;
   - list expected files before editing.
2. **Tests**
   - add failing focused tests for the Story acceptance;
   - do not rewrite unrelated tests.
3. **Minimum implementation**
   - implement the smallest change satisfying the selected Story.
4. **Runtime wiring**
   - prove the real `talos config unset` CLI path and the subsequent startup/model-picker recovery path.
5. **Documentation and owner sync**
   - update Story, iteration, parent, Board, and user/reference docs named by the Story.
6. **Validation and commit**
   - run focused and full validation;
   - inspect staged diff;
   - create one logical commit referencing `MODEL-010` and `I157`.

### Non-Goals

- no TUI `/connect` delete action;
- no bulk/wildcard deletion;
- no environment variable deletion;
- no second secret store;
- no model-catalog change;
- no unrelated config refactor;
- no tag/release.

### Acceptance

All unchecked Acceptance items in `MODEL-010` that belong to this iteration must be satisfied. Do not
invent acceptance criteria here.

### Planned Validation

```bash
cargo test --locked -p talos-config
cargo test --locked -p talos-cli
cargo fmt --all -- --check
cargo check --workspace --locked
cargo clippy --workspace --locked -- -D warnings
cargo test --workspace --locked
scripts/validate_project_governance.sh .
git diff --check
```

### Runtime Evidence

Use an isolated HOME/config fixture and record:
- custom provider removal;
- builtin-backed credential clear/disconnect wording;
- missing `--confirm` leaves file byte-identical;
- active-provider removal followed by startup or `/model` picker recovery;
- logs/config list do not reveal cleared credentials.

### Documentation To Update

- selected Story;
- this iteration;
- parent Epic/Story if its actual state changes;
- `docs/BOARD.md` derived view;
- `docs/backlog/PRODUCT-BACKLOG.md` compact row if state changes;
- `docs/iterations/README.md`;
- user/reference docs named by the Story.

### Risks And Rollback

- Preserve the previous runnable path until the new path has focused and runtime equivalence evidence.
- Roll back the iteration commit if a security, permission, persistence, or product-mode regression is found.
- Do not hide a failed gate by weakening acceptance or deleting tests.

### Stop And Escalate Conditions

- atomic write cannot be achieved with existing config save path;
- active-provider removal panics or requires changing product policy;
- public config API break is required without an ADR;
- implementation needs TUI plumbing or secret-store redesign.

If a stop condition occurs:

1. stop editing;
2. record the exact code/document conflict under Variance And Residuals;
3. keep the iteration `Blocked` or `Review`;
4. do not create a speculative workaround;
5. request maintainer/architecture input.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-28 | Activation | Full inventory at `2bb2b6185f2f9ca35af269efa63c618076f4a32e`: I166/TUI-036 Complete; no other implementation iteration Active or in Review; I164 remains Paused; I158-I162 remain Blocked; ADR-053 remains Proposed. MODEL-010 is Ready and every I157 dependency is satisfied. I157 is the sole Active implementation authority on direct `main` execution. The four-month execution owner is `docs/tasks/2026-07-28-four-month-v06-execution-package.md`. |

## Verification Evidence

- Focused tests: 10 talos-config + 11 talos-cli = 21 new tests, all pass.
- Full locked validation: `cargo test --workspace --locked` 2566 passed, 0 failed;
  `cargo clippy --workspace --locked -- -D warnings` clean;
  `cargo fmt --all -- --check` clean;
  `scripts/validate_project_governance.sh .` 0 warnings.
- Runtime evidence: isolated HOME fixture verified missing-`--confirm` byte-identical,
  custom-provider removal, builtin-backed disconnection semantics, api_key clear
  (no empty string), active-provider removal → picker recovery (no panic), and
  credential scan clean in `config list`/`config get` output.
- Governance validation: I157 sole Active; MODEL-010 Complete; I158-I162 Blocked;
  ADR-053 Proposed; no release/version change.

## Completion Evidence

- Completion Commit: `8055f7ad` (Phase 1 correction: ConfigStore persisted-unset) + `bbe76021` (Phase 2 correction: CLI integration + recovery seam)
- Previous premature completion: `84e7a6a3` + `46c919ee` (retained as historical implementation; superseded by correction).
- Phase 1 correction: `ConfigStore` with atomic temp-file-then-rename writes to
  both config.toml and credentials.toml; raw file reads without merge_credentials;
  credentials.toml resurrection prevention; 11 real persistence tests that reload
  via simulated Config::load.
- Phase 2 correction: 11 real CLI integration tests via `std::process::Command`
  with isolated HOME and `env!("CARGO_BIN_EXE_talos")`; startup model recovery
  seam (`resolve_startup_model_action`) extracted from mode_runners.rs as the
  single production decision source; 4 recovery unit tests.

## Variance And Residuals

- 2026-07-28 priority shift: maintainer selected and activated I164/TUI-038
  after I163 completed. I157 remains Planned, its published scope and baseline
  are unchanged, and it is deferred until I164 disposition rather than
  superseded.
- 2026-07-28 shortcut priority shift: after I165 completed, the maintainer
  selected I166/TUI-036 before I157. I157/MODEL-010 remains Planned/Ready with
  its published baseline unchanged and resumes after I166 reaches a terminal
  disposition.
- 2026-07-28 activation: I166 reached Complete with maintainer acceptance.
  The required inventory found no competing Active or Review implementation
  iteration, so the published I157/MODEL-010 baseline is now Active/In Progress.
  No scope or acceptance target changed.
- 2026-07-29 premature completion correction: the 2026-07-28 completion was
  premature. Acceptance correction findings:
  1. **Atomic save finding**: `Config::save()` uses `fs::write` (truncate-then-write),
     not the atomic temp-file-then-rename pattern used elsewhere in the codebase
     (`recent_models.rs`, `compact_text.rs`). A mid-write I/O failure can leave a
     truncated file.
  2. **Credential resurrection finding**: `Config::load()` reads `credentials.toml`
     via `Credentials::load()` and calls `merge_credentials()`, which re-injects
     old API keys for providers where `api_key` is `None` and even creates new
     provider entries for credentials that no longer have a config entry. After
     `config unset`, the next `Config::load()` resurrects the old credential.
  3. **False-positive test finding**: `unset_success_uses_atomic_save_path` called
     `fs::write` directly, not the production save path.
     `unset_write_failure_preserves_original_file` did not inject a real write
     failure. CLI output tests hand-constructed strings instead of capturing real
     binary output. The active-provider test did not reach the startup/model-picker
     decision seam.
  4. **Missing real CLI evidence**: no integration test exercised the real `talos`
     binary under an isolated HOME.
  5. **Public API variance**: `ConfigUnsetOutcome` is additive; no breaking change.
     The correction adds a narrow `ConfigStore` persisted-unset entrypoint.
  - Correction baseline HEAD: `6cde6508` (I167 Complete; no Active/Review iterations).
  - Selected design: Option A — keep credentials.toml backward-compatibility read
    and durably update both raw config.toml and raw credentials.toml using atomic
    temp-file-then-rename, without `merge_credentials`. The CLI calls one
    authoritative persisted-unset path via `ConfigStore::unset_provider`.
  - Correction commits: `8055f7ad` (Phase 1: ConfigStore + atomic persistence
    + credentials.toml resurrection fix + 11 persistence tests) and `bbe76021`
    (Phase 2: 11 real CLI integration tests via `std::process::Command` with
    isolated HOME + startup model recovery seam extraction + 4 recovery tests).
  - Full workspace validation: 2591 tests, 0 failed; clippy clean; fmt clean;
    governance 0 warnings; no new dependencies; no unsafe; no Cargo.lock change.

## REL-002 Execution Record

- Primary executor/runtime: frontline implementation agent to be dispatched from the four-month package
- External assistance: pending
- Planning/activation/docs ownership: Codex (`gpt-5`); implementation/testing/commit ownership:
  pending dispatch; push ownership remains maintainer-only
- Qualification verdict: pending; do not assume qualification.

## Retrospective

- Outcome: pending
- Documentation: pending
- Lessons: pending
