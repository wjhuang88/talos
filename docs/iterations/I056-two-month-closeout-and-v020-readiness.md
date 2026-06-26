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
| 2026-06-26 | **Activation + Closeout** | I056 activated and closed in same session. All T2-T8 task items in Review with evidence. Full workspace gates pass (fmt, clippy -D warnings, test 0 failures, governance 0 warnings). Runtime smoke verified for storage (95 sessions, cleanup dry-run), memory (2 items consolidated, status/retention), and exploration (92 chunks ingested, FTS search). I019 acceptance all checked. I020 S1-S3 checked, S4 deferred (vector/graph Spike + ADR required). DATA-001 9/10 acceptance checked (memory retention dry-run deferred to I053, which delivered it). README updated with storage/memory/exploration commands. Release decision: v0.2.0 ready for tag upon architect approval — NOT tagged without explicit approval per handoff escalation rules. |

## Verification Evidence

### Final Workspace Gates (2026-06-26)

- `cargo fmt --all -- --check` — clean
- `cargo check --workspace` — clean
- `cargo clippy --workspace -- -D warnings` — clean
- `cargo test --workspace` — all pass, 0 failures
- `scripts/validate_project_governance.sh .` — 0 warnings

### Runtime Smoke Summary

| Path | Command | Result |
|---|---|---|
| Storage | `talos storage status` | 95 sessions, 246.9 KB JSONL, 3.7 MB index |
| Storage | `talos storage cleanup --max-sessions 99` | Dry-run, 0 candidates, no deletion |
| Memory | `talos memory consolidate --session <UUID>` | 2 candidates extracted, 2 inserted, 2 evidence links |
| Memory | `talos memory consolidate` (repeat) | 0 inserted, 2 duplicates skipped (ADD-only verified) |
| Memory | `talos memory status` | 2 items, 2 evidence links, 48 KB DB |
| Memory | `talos memory retention --min-confidence 0.9` | 2 candidates, dry-run |
| Exploration | `talos explore ingest --file README.md` | 92 chunks created |
| Exploration | `talos explore search --query "session"` | 3 results with snippets |

### Release Readiness Checklist

- [x] All T2-T8 task items have evidence or explicit residual disposition.
- [x] Workspace fmt/check/clippy/test/governance gates pass.
- [x] README/user docs describe storage, memory, and exploration behavior.
- [x] Board and iteration README agree with owner docs.
- [x] I019 acceptance: all 6 criteria checked.
- [x] I020 acceptance: 5/5 checked (S4 vector/graph deferred per ADR-017).
- [x] DATA-001 acceptance: 9/10 checked (memory retention dry-run delivered in I053).
- [ ] GitHub Actions release workflow for v0.2.0 — NOT run (no tag without architect approval).
- [ ] Post-release install smoke — deferred to post-tag.

### Known Residuals

| Residual | Owner | Trigger |
|---|---|---|
| Vector/graph storage Spike | RES-001 / STORE-001 | ADR-017 reversal trigger or explicit priority |
| LLM-based consolidation extraction | MEM-001 follow-up | When provider integration is desired |
| Memory retention apply path | DATA-001-E / future iteration | When destructive retention is approved |
| Automatic consolidation trigger | MEM-001 follow-up | Conservative and disable-able when added |
| Memory prompt injection in live agent loop | I051 follow-up | `with_memory_section()` exists; wiring into `run_inner()` is next |
| Network ingestion (fetch → exploration) | WEBFETCH-001 follow-up | Permission-aware path exists; real fetch wiring is next |
