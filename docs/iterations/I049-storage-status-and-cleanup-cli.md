# I049: Storage Status And Cleanup CLI

**Status**: Planned
**Created**: 2026-06-26
**Depends On**: I048 foundation; DATA-001; I047 release workflow evidence reviewed or explicitly deferred

## Objective

Expose user-facing local storage visibility and explicit cleanup commands so DATA-001 becomes
operational before I019 automatic memory writes.

## Published Baseline

### Selected Stories

- DATA-001-A: storage status read-only report.
- DATA-001-B: session cleanup dry-run/apply with active-session protection.
- DATA-001-C: fork storage visibility.
- DATA-001-D: explicit SQLite checkpoint/vacuum command surface.

### MVP Deliverable

`talos storage status` reports Talos-owned local storage without writing files, and
`talos storage cleanup --dry-run` reports session cleanup candidates without deleting data.

### Scope

- Add `talos storage status`.
- Add `talos storage cleanup --dry-run` and guarded `--apply`.
- Add workspace-scoped cleanup flags for max sessions and max age.
- Protect the active session at the command boundary.
- Show session index, WAL/SHM, log, model cache, memory DB, and fork storage information.
- Call existing explicit checkpoint/vacuum APIs only from maintenance commands.

### Non-Goals

- No automatic cleanup.
- No copy-on-write fork optimization.
- No autonomous memory writes.
- No release tag.

### Acceptance

- Given no `~/.talos` directory, `storage status` exits successfully with zero/missing surfaces.
- Given sessions and index rows, cleanup dry-run reports candidates without deleting files.
- Given cleanup apply, selected non-active sessions lose JSONL and index/fork rows together.
- Given an active session ID, cleanup refuses to select it.
- Given forked sessions, status reports fork count/source relationship where known.
- Given WAL files, maintenance can checkpoint/truncate explicitly.

### Validation Plan

- CLI temp-home tests for status missing/partial/full roots.
- CLI temp-home tests for cleanup dry-run/apply/active protection.
- Session manager tests for index and fork row deletion.
- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `scripts/validate_project_governance.sh .`

### Documentation To Update

- `README.md`
- `README.zh-CN.md`
- `docs/backlog/active/DATA-001-local-data-lifecycle-storage-hygiene.md`
- `docs/BOARD.md`

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-06-26 | **Activation** | I049 activated. Gate verification: (1) I047 release workflow evidence confirmed — `v0.1.2` GitHub Actions release workflow completed successfully with all 6 assets uploaded (checksum + darwin/linux/windows binaries); (2) I048 pre-activation foundation APIs verified present in `71b0392` (SessionManager cleanup_candidates/apply_cleanup/checkpoint_index/vacuum_index/reconcile_index, MemoryStore checkpoint_truncate/vacuum/count). Working tree clean at `04d5e01`. Scope: `talos storage status` (read-only), `talos storage cleanup --dry-run/--apply` (active-session protected), `talos storage maintenance --checkpoint/--vacuum/--reconcile`. No memory DB file-path wiring changes beyond status reporting; MemoryStore remains library-only per I047 boundary. |
| 2026-06-26 | **Implementation** | All four slices delivered: DATA-001-A (`storage status` read-only report with sessions/index/WAL/forks/logs/cache/memory surfaces), DATA-001-B (`storage cleanup` dry-run default + `--apply` requiring explicit criteria + `--protect-session` active-session protection), DATA-001-C (fork count visibility in status via new `SessionManager::get_forks()` wrapper), DATA-001-D (`storage maintenance --checkpoint/--vacuum/--reconcile` on session index + memory DB). New module `crates/talos-cli/src/storage.rs` (~478 lines). 7 new CLI tests. Pre-existing flaky `init_wizard` test race fixed with `ENV_MUTEX` serialization. |

## Verification Evidence

### Workspace Gates (2026-06-26)

- `cargo fmt --all -- --check` — clean
- `cargo check --workspace` — clean
- `cargo clippy --workspace -- -D warnings` — clean
- `cargo test --workspace` — all pass, 0 failures
- `scripts/validate_project_governance.sh .` — 0 warnings

### End-to-End Runtime Evidence (ITERATION-WORKFLOW §3a)

- `talos storage status` on real user data: reports 95 sessions across 3 workspaces, 246.9 KB JSONL, 3.7 MB index.db, 1.2 MB logs, memory DB not initialized. Exits 0.
- `talos storage cleanup --apply` (no criteria): correctly rejected with "requires at least one selection criterion".
- `talos storage cleanup --max-sessions 99` (dry-run): reports no candidates, confirms "no files deleted".
- `talos storage maintenance --vacuum`: "Session index: vacuum completed."
- `talos storage status` with missing HOME: "Talos root (~/.talos): not found", exits 0.
- `talos storage --help`: shows Status/Cleanup/Maintenance subcommands.

### Changed Files

| File | Change |
|---|---|
| `crates/talos-session/src/manager.rs` | Added `ForkInfo` import + `get_forks()` public method |
| `crates/talos-cli/src/storage.rs` | NEW: StorageCommand enum, status/cleanup/maintenance functions |
| `crates/talos-cli/src/main.rs` | Added `mod storage;`, `Storage` variant to `TalosCommand`, dispatch |
| `crates/talos-cli/Cargo.toml` | Added `talos-memory` dependency |
| `crates/talos-cli/src/tests.rs` | 7 new tests covering all acceptance criteria |
| `crates/talos-cli/src/init_wizard.rs` | Fixed pre-existing HOME env var race with `ENV_MUTEX` |
