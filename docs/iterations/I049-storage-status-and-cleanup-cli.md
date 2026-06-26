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
