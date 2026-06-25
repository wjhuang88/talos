# Long-Running Task: I047 -> I019 Memory And Release Readiness Sequence

> Status: Planned
> Created: 2026-06-25
> Owner iteration: [I047 v0.1.2 Release Readiness And Runtime Polish](../iterations/I047-v012-release-readiness-and-runtime-polish.md)
> Baseline rule: this task inventory is preserved; changed objectives use a new task record or
> change-control entry.

## Startup Contract

### Outcome

Run a multi-iteration sequence that makes Talos release-ready for `v0.1.2`, clears every known
prerequisite for the published I019 memory foundation baseline, opens the memory system with a
small executable starter slice, and leaves the project ready to activate full I019 afterward.

The intended sequence is:

1. **I047**: release readiness, first-run setup, I019 prerequisite closure, memory starter,
   compaction policy, read-only governance status, closeout docs.
2. **I019**: full layered memory foundation activation after I047 confirms prerequisites.
3. **I020**: exploration library remains blocked until I019 or an explicit research-priority replan.

### In Scope

- I047 release/install readiness (`REL-001`) and `v0.1.2` preparation.
- First-run model configuration onboarding (`CONF-002`).
- I019 prerequisite closure:
  - I024/MEM-002 already complete; verify and record.
  - I018/OBS-001 bounded logs and embedded prompt assets delivered inside I047.
- Memory starter (`MEM-001-A`): memory layer boundaries, SQLite schema, ADD-only records, evidence
  links, bounded retrieval API.
- Context compaction policy Phase 1 (`MEM-005-A`) around existing layers 1-3.
- Read-only governance status Phase 1 (`GOV-003-A`).
- Owner-doc synchronization: backlog, iterations, Board, README/zh-CN README, release notes,
  prerequisite status, and residuals.
- Checkpoints at every phase boundary.

### Out Of Scope

- Moving, deleting, or overwriting `v0.1.1`.
- Pushing a release tag without explicit final release approval.
- Windows ARM64 release builds.
- `ring`/TLS migration to `native-tls`.
- Full I019 completion inside I047.
- Full MEM-003 LLM compaction layers 4-5.
- Automatic memory prompt injection before bounded retrieval and provenance gates are proven.
- Vector/graph database adoption.
- I020 exploration library implementation.
- GOV-003 gate enforcement, auto-repair, or WEB-001 project management UI.
- Delayed/scheduled tasks, remote session control, WASM plugins, or provider plugin architecture.

### Ordered Task Items

| ID | Task | Expected Output | Depends On | Completion Gate | Fallback | Status |
|---|---|---|---|---|---|---|
| T0 | Confirm execution contract | User-approved scope, permissions, release/tag boundary, and stop conditions recorded | This Planned task | Task status can move from Planned to In Progress | If approval is partial, split unauthorised work into residuals | Planned |
| T1 | Close pre-I047 repair leftovers | `.gitignore`, installers, I045/CONF-001 doc drift, and I046 post-handoff notes are committed or intentionally carried into I047 | T0 | `scripts/validate_project_governance.sh .`; installer parse checks | Leave as pre-I047 residual and do not activate I047 until resolved | Planned |
| T2 | Activate I047 baseline | I047 gains activation record; Board Now points to I047; selected story statuses are synchronized | T1 | Owner docs and Board agree; governance validation passes | Keep I047 Planned and stop if inventory conflicts remain | Planned |
| T3 | REL-001 release/install readiness | Supported target matrix, artifact names, installers, checksum behavior, and `v0.1.2` strategy are validated | T2 | Packaging smoke or documented target subset; installer dry-run; no tag mutation | Defer tag; record blocking target or installer defect | Planned |
| T4 | CONF-002 first-run onboarding | Empty-config users get guided setup; `talos init` re-runs setup; non-interactive mode does not hang | T2 | Temp-home runtime tests; masked credential display; config round-trip | Provide actionable error path and keep wizard partial | Planned |
| T5 | OBS-001/I018 prerequisite closure | Bounded file logs and compile-time embedded prompt assets land; I019 no longer blocked on I018 | T2 | ADR-014/015 tests; I018/MEM-001/I019/Board status synchronized | If OBS-001 expands, deliver only bounded logs + embedded prompts and defer R3 logging | Planned |
| T6 | MEM-001-A memory starter | Memory boundary, SQLite schema, ADD-only writes, evidence links, and bounded retrieval API | T5 | Migration/schema/retrieval tests; no vector/graph dependency; provenance returned | Stop at schema + API; defer prompt injection/consolidation to I019 | Planned |
| T7 | MEM-005-A compaction policy | Threshold policy, safe-boundary compaction, manual command/status, failure fallback | T6 | Unit/mock session/TUI command tests; hidden output never printed | Keep policy library-only if command integration risks the timebox | Planned |
| T8 | GOV-003-A read-only governance status | Governance status command reads iteration/backlog/board/validation state without writing docs | T2 | Empty/partial/full workspace tests; dirty-worktree guard | Keep as library/status report only; defer prompt injection | Planned |
| T9 | I047 closeout and release rehearsal | I047 evidence, docs, release checklist, and residuals are synchronized; release decision ready | T3-T8 | check/clippy/test/governance pass; release rehearsal recorded | Mark I047 Review/Partial if any required gate fails | Planned |
| T10 | I019 activation decision | I019 can activate, be replanned, or remain deferred with explicit reason | T9 | I019 prerequisites recorded as satisfied; Board/iterations index agree | Create a new I048/I019 activation plan if full I019 scope changes | Planned |
| T11 | I020 dependency disposition | I020 remains blocked/deferred until I019 or explicit research-priority replan | T10 | Board and iterations index state dependency clearly | Leave I020 unchanged if no research activation is requested | Planned |

### Dependencies And Prerequisites

- I024/MEM-002 is complete and provides working/episodic session wiring.
- I018/OBS-001 is still Planned and is the known blocking prerequisite for I019.
- ADR-014, ADR-015, ADR-016, ADR-002, and ADR-008 remain binding.
- Existing release workflow and installer files are available.
- No remote release/tag mutation is authorized by this task record alone.

### Artifacts And State Owners To Update

- Iterations: I047, I018, I019, I020, iterations README.
- Backlog: REL-001, CONF-002, OBS-001, MEM-001, MEM-005, GOV-003, Product Backlog.
- Derived view: `docs/BOARD.md` only after owner docs are updated.
- User docs: `README.md`, `README.zh-CN.md`, release notes/checklist.
- Decision/lesson records: ADR only if a new dependency/security/storage decision appears;
  EVOLUTION only if a reusable lesson or failed validation appears.
- Task checkpoints: this file after each task item or phase boundary.

### Validation And Acceptance Evidence

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `scripts/validate_project_governance.sh .`
- Installer parse and dry-run checks.
- Packaging smoke or documented supported-target subset.
- Temp-home first-run setup runtime scenario.
- OBS-001 bounded log and embedded prompt asset tests.
- Memory schema/retrieval tests with ADD-only conflict preservation.
- Compaction threshold/manual/failure tests.
- Governance status empty/partial/full workspace tests.

### Branch, Worktree And Checkpoint Plan

- Default branch/worktree: current `main` workspace unless the user requests a branch.
- Do not force-push or rewrite history.
- Commit in logical slices after gates pass:
  - pre-I047 repair leftovers;
  - I047 activation;
  - release/install readiness;
  - first-run setup;
  - OBS-001 prerequisite closure;
  - memory starter;
  - compaction/governance status;
  - closeout/release rehearsal.
- Append a checkpoint before stopping, handing off, or starting a new phase.

### Allowed Permissions And External Actions

Planned authorization for later confirmation:

- Read/edit repository files.
- Run local builds, tests, packaging smoke, installer dry-runs, and governance scripts.
- Make local commits if the user asks for execution and commit.

Not authorized without explicit later approval:

- Push commits.
- Create, move, or delete tags.
- Create, delete, or overwrite GitHub Releases.
- Publish `v0.1.2`.
- Add new runtime dependencies.
- Perform destructive filesystem or git operations.
- Spend money or use paid provider credentials for validation.

### Destructive Or Irreversible Operations

None authorized. Release/tag operations are deliberately excluded until an explicit release approval.

### Time, Cost And Resource Limits

- Intended planning horizon: about one month for I047 plus a later activation decision for I019.
- Monetary spend: zero unless separately approved.
- Retry a failing deterministic validation at most twice after a concrete fix.
- Keep generated artifacts under `dist/` and do not commit them.

### Failure, Retry And Fallback Policy

- If a prerequisite gate fails, do not start dependent memory implementation.
- If release packaging or installer validation fails, defer tagging and record the exact blocker.
- If memory schema/retrieval grows beyond timebox, stop at schema + bounded retrieval API and defer
  automatic consolidation/prompt injection to I019.
- If governance parsing is brittle, keep GOV-003-A read-only/tolerant and defer enforcement.

### Default Decisions For Foreseeable Ambiguity

- Prefer `v0.1.2` over moving `v0.1.1`.
- Prefer local/dry-run validation before network release actions.
- Prefer SQLite/FTS5 and pure Rust; no vector/graph DB.
- Prefer preserving raw session JSONL as source of truth.
- Prefer deferring optional polish over weakening prerequisite or validation gates.

### Residual-Work Destination

- Full I019 memory foundation remains in `docs/iterations/I019-layered-memory-foundation.md`.
- I020 exploration library remains in `docs/iterations/I020-exploration-library.md`.
- TLS/native-tls migration should become a future dependency strategy story/ADR.
- Structured JSON logs/shared span contracts remain outside OBS-001 R2.

## Checkpoints

No execution checkpoint yet. This task is Planned and awaits explicit activation/confirmation.
