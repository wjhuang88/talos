# I056: Two-Month Closeout And v0.2.0 Readiness

**Status**: Planned
**Created**: 2026-06-26
**Depends On**: I055 exploration ingestion and citation workflow

## Objective

Close the two-month sequence with verification, documentation, residual mapping, and a release
readiness decision for the next minor release.

## Published Baseline

### Selected Stories

- DATA-001/I019/I020 closeout synchronization.
- Release-readiness audit for the memory/exploration milestone.

### MVP Deliverable

The project has a clear Review/Complete status for DATA-001, I019, and I020 slices, workspace gates
are green, user docs are current, and the architect has a concrete release/no-release decision.

### Scope

- Run full workspace gates and targeted runtime smoke tests.
- Verify storage, memory, and exploration docs match actual behavior.
- Record residuals under owning backlog items.
- Prepare release checklist and version decision.
- Do not tag without explicit approval.

### Non-Goals

- No new feature implementation except closeout fixes required by validation.
- No release tag or GitHub Release mutation without explicit approval.
- No broad refactor.

### Acceptance

- All selected two-month task items have evidence or explicit residual disposition.
- Workspace fmt/check/clippy/test/governance gates pass.
- README/user docs describe new storage, memory, and exploration behavior accurately.
- Board and iteration README agree with owner docs.
- Release checklist identifies tag/version, supported targets, installer status, and known
  residuals.

### Validation Plan

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `scripts/validate_project_governance.sh .`
- Release checklist review.
- Runtime smoke tests for storage, memory, and exploration command paths.

### Documentation To Update

- `README.md`
- `README.zh-CN.md`
- `docs/iterations/README.md`
- `docs/BOARD.md`
- Relevant backlog owners for any residuals.
- Release notes/checklist.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
