# Long-Running Task: DATA-001 -> I019 -> I020 Two-Month Execution Plan

> Status: Planned
> Created: 2026-06-26
> Planning horizon: about two months / eight one-week iterations
> Owner iterations: [I048](../iterations/I048-local-data-lifecycle-storage-hygiene.md),
> [I049](../iterations/I049-storage-status-and-cleanup-cli.md),
> [I050](../iterations/I050-memory-consolidation-pipeline.md),
> [I051](../iterations/I051-bounded-memory-prompt-injection.md),
> [I052](../iterations/I052-procedural-memory-and-entity-linking.md),
> [I053](../iterations/I053-memory-quality-and-release-hardening.md),
> [I054](../iterations/I054-exploration-library-storage-foundation.md),
> [I055](../iterations/I055-exploration-ingestion-and-citation-workflow.md),
> [I056](../iterations/I056-two-month-closeout-and-v020-readiness.md)
> Programmer handoff: [Programmer Handoff](2026-06-26-programmer-handoff-data-memory-exploration.md)
> Baseline rule: this task inventory is preserved; changed objectives use a new task record or
> change-control entry.

## Startup Contract

### Outcome

Run an ordered two-month sequence that closes Talos local data lifecycle risk, activates layered
memory safely, then opens the exploration library on top of provenance-backed memory and storage.

The sequence intentionally treats storage lifecycle as the gate before autonomous memory writes:

1. DATA-001/I048-I049: visible local storage status, cleanup, SQLite maintenance, memory retention
   dry-run, and fork visibility.
2. I019/I050-I053: episodic-to-semantic consolidation, bounded prompt injection, procedural
   adaptation, entity linking, contradiction metadata, and release hardening.
3. I020/I054-I055: local research/exploration storage, source ingestion, claim/citation workflow.
4. I056: closeout, regression sweep, docs, and `v0.2.0` readiness decision.

### In Scope

- Complete DATA-001 user-facing storage lifecycle controls before enabling automatic memory writes.
- Activate I019 through small executable iterations rather than a single large batch.
- Keep all memory writes ADD-only and provenance-backed per ADR-016.
- Keep raw session JSONL as the durable source of truth.
- Add bounded retrieval and prompt injection with explicit token budgets and hidden-output guards.
- Add procedural memory only after semantic memory retrieval is bounded and observable.
- Add exploration library storage and citation workflow after memory provenance is reliable.
- Synchronize backlog, iteration records, Board, README/user docs, and validation evidence at each
  phase boundary.

### Out Of Scope

- Automatic deletion of user data by default.
- Vector, graph, or external storage dependencies without a Spike and ADR.
- Background daemons or scheduled cleanup.
- Remote session protocol, embedded web UI, WASM plugin runtime, or local model inference.
- Moving the `v0.1.2` tag or modifying published release history.

### Ordered Task Items

| ID | Iteration | Expected Output | Depends On | Completion Gate | Fallback | Status |
|---|---|---|---|---|---|---|
| T0 | Planning | Two-month task record and iteration sequence committed | Current clean `main` | Governance validation passes | Keep as planning-only draft | Planned |
| T1 | I048 | Library-level DATA-001 foundation validated and planned CLI scope preserved | T0; `v0.1.2` tag published | Session/memory maintenance APIs tested; I048 still honest about remaining CLI work | Leave I048 Planned if release workflow evidence blocks activation | Planned |
| T2 | I049 | Storage status and cleanup CLI commands | T1 | Missing-data, dry-run, apply, active-session protection, fork visibility tests pass | Keep cleanup library-only and ship status first | Review (2026-06-26) |
| T3 | I050 | Episodic-to-semantic consolidation pipeline | T2 | Batch/end-of-session consolidation tests with ADD-only evidence links | Keep consolidation manual-only if automatic trigger is risky | Review (2026-06-26) |
| T4 | I051 | Bounded memory retrieval prompt injection | T3 | Token-budgeted prompt section tests; hidden tool outputs never injected | Keep retrieval API exposed but injection disabled by config | Planned |
| T5 | I052 | Procedural memory extraction and entity linking | T4 | Entity extraction and procedural adaptation tests; no permission authority | Limit procedural memory to advisory prompt text | Planned |
| T6 | I053 | Memory quality gates and release hardening | T5 | Contradiction, decay, retention dry-run, observability, and docs verified | Defer non-blocking ranking polish | Planned |
| T7 | I054 | Exploration library storage foundation | T6 | SQLite schema + FTS source/chunk/claim/synthesis tests | Keep exploration storage library-only | Planned |
| T8 | I055 | Exploration ingestion and citation workflow | T7 | Permission-aware ingestion, claim citation, and synthesis tests | Keep network ingestion disabled by default | Planned |
| T9 | I056 | Two-month closeout and `v0.2.0` readiness decision | T8 | Workspace gates, governance, release checklist, residuals, and docs complete | Mark Review with exact blockers | Planned |

### Dependencies And Prerequisites

- `v0.1.2` tag has been pushed; I047 remains Review until release workflow evidence is recorded.
- DATA-001 foundation APIs landed in commit `71b0392`.
- I019 prerequisites from I047 are cleared, but I019 activation must wait for DATA-001 completion
  or an explicit change-control exception.
- ADR-002, ADR-008, ADR-016, and ADR-017 remain binding.
- Any vector/graph dependency requires Spike evidence and a follow-up ADR before implementation.

### Artifacts And State Owners To Update

- Long-running task record: this file.
- Iterations: I048 through I056 and `docs/iterations/README.md`.
- Backlog owners: DATA-001, MEM-001, MEM-005, RES-001, STORE-001, WEBFETCH-001 as touched.
- Derived view: `docs/BOARD.md` only after owner docs are updated.
- User docs: `README.md`, `README.zh-CN.md`, release notes, and config reference when CLI or user
  behavior changes.
- Decision records: new ADR only if storage, dependency, security, or release policy changes.
- Lessons: `EVOLUTION.md` only for reusable corrections, validation failures, or user corrections.

### Validation And Acceptance Evidence

Each implementation iteration must run at minimum:

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `scripts/validate_project_governance.sh .`

Iteration-specific validation adds focused CLI/runtime tests, temp-home storage scenarios, prompt
injection snapshot tests, and storage migration tests as listed in each iteration plan.

### Branch, Worktree And Checkpoint Plan

- Default branch/worktree: `main`, unless the user requests a branch.
- Commit in logical iteration checkpoints after gates pass.
- Push commits after each iteration checkpoint when the user has authorized it.
- Append checkpoint evidence to this file and the owning iteration before stopping or handing off.
- Never force-push `main`; never move existing tags.

### Allowed Permissions And External Actions

Allowed during execution after user approval:

- Read/edit repository files.
- Run local tests, builds, CLI smoke tests, and governance scripts.
- Commit and push completed iteration checkpoints.

Not allowed without separate explicit approval:

- Tagging or publishing a release.
- Adding major new runtime dependencies.
- Deleting user data outside test temp directories.
- Network calls that spend money or require private credentials.
- Destructive git operations.

### Destructive Or Irreversible Operations

No destructive production/user-data operations are authorized by this plan. Cleanup behavior must
be validated in temp directories and exposed as explicit dry-run/apply commands only.

### Time, Cost And Resource Limits

- Intended pace: one iteration per week, eight weeks total.
- Monetary spend: zero unless separately approved.
- Retry deterministic gates at most twice after concrete fixes.
- Prefer deferring optional polish over weakening storage, permission, provenance, or hidden-output
  boundaries.

### Failure, Retry And Fallback Policy

- If DATA-001 user-facing lifecycle controls fail, do not enable automatic memory writes.
- If prompt injection can expose hidden tool output, disable injection and keep retrieval
  library-only.
- If procedural memory affects permission/security decisions, stop and replan; procedural memory is
  advisory only.
- If exploration ingestion cannot meet permission or citation requirements, keep storage foundation
  and defer ingestion.

### Default Decisions For Foreseeable Ambiguity

- Prefer reversible, read-only status before write-capable cleanup.
- Prefer manual maintenance commands before automatic background work.
- Prefer SQLite/FTS5 and pure Rust; no vector/graph dependency in this sequence.
- Prefer bounded prompt sections over aggressive context stuffing.
- Prefer explicit residuals over expanding an iteration beyond its acceptance boundary.

### Residual-Work Destination

- Vector/graph acceleration: STORE-001 + new ADR after Spike.
- Rich document conversion and external search integrations: WEBFETCH-001 follow-up.
- Release automation beyond readiness checklist: REL-001 follow-up.
- Remote/web/plugin surfaces: REMOTE-001, WEB-001, PLUGIN-001.

## Checkpoints

### T0 — Planning Baseline Created (2026-06-26)

Created the two-month task record and iteration sequence I049-I056. This is planning only; no new
runtime behavior is claimed by this checkpoint.

Recovery/resume instruction: start with I048/I049 owner docs, verify I047 release workflow status,
then activate I049 only if DATA-001 CLI/storage status scope remains the next priority.

### T0b — Programmer Handoff Created (2026-06-26)

Created `docs/tasks/2026-06-26-programmer-handoff-data-memory-exploration.md` for assignment
distribution. It records assignment boundaries, required reads, gates, risks, and handoff note
format for I049-I056.

Recovery/resume instruction: distribute the handoff before assigning I049; require each programmer
to append activation/evidence records to the owning iteration before implementation.

### T2 — I049 Storage Status And Cleanup CLI Complete (2026-06-26)

I049 (Assignment A1) implemented and verified. All four DATA-001 user-facing CLI slices delivered:

- `talos storage status`: read-only report of sessions, index DB (+WAL/SHM), fork counts, logs,
  cache, and memory DB. Tolerates missing `~/.talos`.
- `talos storage cleanup`: dry-run default; `--apply` requires explicit criteria; active-session
  protection via `--protect-session`.
- `talos storage maintenance --checkpoint/--vacuum/--reconcile`: explicit SQLite maintenance.
- `SessionManager::get_forks()` public API added for fork visibility.

Gates: fmt, check, clippy (`-D warnings`), test (all pass), governance (0 warnings).
Runtime smoke: verified with real `talos` binary on real user data.
Pre-existing init_wizard HOME env var race fixed with `ENV_MUTEX`.

Deferred to I053: memory retention dry-run (DATA-001-E). Memory store remains library-only.

Recovery/resume instruction: I049 is in Review. Next assignment A2 (I050 Memory Consolidation
Pipeline) may start after I049 is committed — DATA-001 user-facing lifecycle controls are
operational. Read I050 iteration doc and MEM-001 backlog before activation.

### T3 — I050 Memory Consolidation Pipeline Complete (2026-06-26)

I050 (Assignment A2) implemented and verified. Episodic-to-semantic consolidation pipeline:

- `EpisodeExtractor` trait with deterministic `RuleBasedExtractor` (no provider dependency).
- `consolidate_episodes()` ADD-only pipeline: extract → insert (content-hash dedup) → evidence links.
- `ConsolidationConfig` default `enabled: false` (opt-in safety).
- CLI `talos memory consolidate [--session <UUID>]` reads session JSONL and runs pipeline.
- 6 unit tests covering all acceptance criteria (evidence creation, dedup, conflict preservation,
  malformed sessions, disabled config, determinism).

Gates: fmt, check, clippy (`-D warnings`), test (all pass), governance (0 warnings).
Runtime smoke: `talos memory consolidate --session <UUID>` extracted 2 candidates, inserted 2 with
evidence links; second run deduped all (ADD-only verified).

Recovery/resume instruction: I050 is in Review. Next assignment A3 (I051 Bounded Memory Prompt
Injection) may start after I050 is committed — consolidation evidence exists. Read I051 iteration
doc and MEM-005 backlog before activation.
