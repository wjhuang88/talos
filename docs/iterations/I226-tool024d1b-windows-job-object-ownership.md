# Iteration I226: Windows Job Object Process-Tree Ownership

> Document status: Review / Claimed
> Published plan date: 2026-08-25
> Planned objective: implement ADR-068's assigned-before-exec Windows Job Object boundary for TOOL-024-D1-B.
> Baseline rule: preserve this target; changed objectives require a new iteration ID.
> MVP deliverable: an authorized Windows background command is owned by a Job Object before resume, with bounded descendant cleanup and real Windows evidence.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 Sol mainline Issue #59 session |
| Work Slice | TOOL-024-D1-B Windows launcher, Job Object ownership, allowlisted stdio inheritance, fail-closed cleanup and focused Windows tests only. |
| Claimed At | 2026-08-25 |
| Source Issue | #59 |
| Governance Claim PR | #393 |
| Authorization Mode | Independent review |
| Authorization Evidence | ADR-068 accepted on `main@93ee3253`; I225 decision evidence `fca45c46` / CI `32797375011` / review `5404361120`; closeout review `5405268380`. |
| Implementation PR | #394 — open; implementation code commit `70e8b674` plus later evidence/governance synchronization |
| Last Updated | 2026-08-25 |
| Handoff / Release Condition | Candidate remains in Review until Windows exact-head CI and independent process/security/API review pass; no completion or merge authority is implied. |

## Published Baseline

### Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| TOOL-024-D1-B | TOOL-024 / Issue #59 | Ready / Unclaimed | I225 Complete; ADR-068 Accepted | Windows Job Object launcher with real child/grandchild and handle-isolation evidence. |

### Scope

- Implement ADR-068 in the minimum Windows production boundary and integrate the existing supervisor.

### Non-Goals

- D2 CLI/TUI, I223 validation, Dashboard, permissions, `/auto`, persistence, Unix behavior, release and Desktop.

### Acceptance

- Given a permitted Windows background command, when launched, then Job assignment precedes thread resume and descendants are owned.
- Given any ownership setup uncertainty, when launch is attempted, then it fails closed with no leaked child or handle.
- Given concurrent launches, when a child inspects inherited handles, then only required stdio is visible.

### Planned Validation

- Focused Windows tests plus full locked workspace validation and release preflight.
- Exact-head CI and independent process/unsafe/API review.
- Real Windows marker, descendant, cancellation, timeout, shutdown and concurrent-handle fixtures.

### Documentation To Update

- TOOL-024-D1-B owner, TOOL-024 parent, Issue #59 long task, Board and iteration index.
- Directly affected user/API documentation only; D2 help and TUI projection remain separate.

### Risks And Rollback

- Risk: any pre-assignment execution or broad handle inheritance violates ADR-068.
- Rollback: retain `background_process_tree_unsupported` on Windows and remove the private launcher feature gate.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-25 | Atomic claim + activation | PR #393 exact head `2905c99d`, CI `32810069430`, independent approval `5405428154`, merge-time CAS and merge `d1f2a126`; implementation starts from this merge. |
| 2026-08-25 | Stable implementation candidate | PR #394 implementation candidate `70e8b674` contains the Windows-only Job Object launcher and Bash/Exec integration. Local fmt, diff and focused locked check/test pass; release preflight reaches its governance checks but the full build is interrupted by local `ENOSPC`. Exact-head CI `32820263589` is green, including Windows workspace compilation, Clippy and runtime tests. |
| 2026-08-25 | Candidate reconciliation update | PR #394 moved to exact head `95740c34`. The candidate adds only the I226 implementation plus owner/derived synchronization and the open-Issue reconciliation entry for #395 (`OBS-002` Intake / Unclaimed, no implementation authority). CI `32821573565` completed the Rust, Windows workspace, installer and classification jobs successfully; remote issue reconciliation failed because #395 was not yet present in that head. The subsequent local reconciliation now passes against all 52 open Issues. |

## Verification Evidence

- Local candidate checks: `cargo fmt --all`, `git diff --check`, `cargo check -p talos-tools --features shell --locked`, and `cargo test -p talos-tools --features shell --locked` pass (110 unit tests plus 3 hardening integration tests). The full release preflight was interrupted by local `ENOSPC` after governance validation passed.
- Exact-head CI `32820263589` passes 5/5 for the candidate implementation, and the Windows workspace test log records the six I226 launcher tests passing (quoting, case-insensitive environment filtering, output/reap, invalid cwd fail-closed, Job termination and grandchild cleanup). Independent Windows/process/unsafe/API review is still required; no merge or completion claim is made.
- Exact head `95740c34` superseded `70e8b674` for evidence and completed all Rust/Windows jobs, but its remote issue reconciliation failed because #395 was absent. The corrected local tree passes the same remote validator after OBS-002 owner creation and Issue comment `5407182562`; fresh exact-head CI and review are still required after push.

## Changed-File Inventory

- I226 production/dependency authority: `Cargo.lock`, `crates/talos-tools/Cargo.toml`,
  `crates/talos-tools/src/bash_tool.rs`, `crates/talos-tools/src/exec_tool.rs`, and
  `crates/talos-tools/src/process_boundary.rs`.
- I226 owner/derived synchronization: `docs/BOARD.md`,
  `docs/backlog/active/TOOL-024-D1-B-windows-job-object-ownership.md`,
  `docs/backlog/active/TOOL-024-background-command-jobs.md`, this iteration,
  `docs/iterations/README.md`, and the Issue #59 long-task record.
- Unrelated remote reconciliation only: `docs/backlog/active/OBS-002-structured-diagnostics-contract.md`,
  `docs/backlog/PRODUCT-BACKLOG.md`, and
  `docs/reference/ISSUE-DOC-CODE-STATUS-2026-08-23.md`. These files register #395 as
  Intake / Unclaimed and grant no observability implementation authority.

## Completion Evidence

- Completion Commit: Pending.

## Variance And Residuals

- D2 and I223 remain separately governed and are not activated by I226. Windows child/grandchild, handle-isolation and cancellation evidence remain pending until a green exact-head candidate exists.

## Retrospective

- Pending implementation.
