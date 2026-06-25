# Iteration I047: v0.1.2 Release Readiness And Runtime Polish

> Document status: Active
> Published plan date: 2026-06-25
> Planned objective: Produce a release-ready Talos month slice that stabilizes installation,
>   first-run model setup, clears all known prerequisites for I019, opens the memory-system
>   foundation, bounds long-session behavior, and adds read-only governance awareness before the
>   next stable tag.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a runnable `talos` binary that can be packaged as `v0.1.2`, installed with the
>   simplified archive names, guide an unconfigured user through model setup, create the first
>   auditable memory foundation slice after its prerequisites are satisfied, expose safe compaction
>   controls for long sessions, and show current project governance status without mutating docs.

## Published Baseline

### Non-Terminal Iteration Inventory

| Iteration | State | Disposition |
|---|---|---|
| Now | No active iteration | I047 is created as `Planned`; activation requires a later explicit start record. |
| I011 S2 Provider Plugin Architecture | Paused | Remains paused; not reopened by I047. Provider plugin loading stays out of scope. |
| I018 Observability and Prompt Assets | Planned | Selected into I047 as the prerequisite-closure work for I019. The I018 baseline is preserved; if I047 completes the same bounded log + embedded prompt asset acceptance, I018 should be marked fulfilled/superseded by I047 during closeout. |
| I019 Layered Memory Foundation | Planned | Preserved as a published baseline. I047 must satisfy all known I019 prerequisites, then deliver the starter `MEM-001-A` slice without claiming the full I019 baseline is complete. |
| I020 Exploration Library | Planned | Remains blocked/deferred until I019 or an explicit research-priority replan. |
| I028 Delayed/Scheduled Tasks | Planned | Deferred. Scheduling is not release-critical for `v0.1.2`. |
| I036 Research Consolidation | Complete | No activation blocker; research outputs inform future work only. |

### Long-Running Task Owner

I047 is the first phase of [Long-Running Task: I047 -> I019 Memory And Release Readiness Sequence](../tasks/2026-06-25-i047-i019-memory-release-sequence.md).
That task record owns the multi-iteration dependency chain from I047 prerequisite closure through
the later I019 activation decision. I047 remains the iteration owner for this month's executable
slice; the long-running task is the recovery/checkpoint owner.

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `REL-001` Release And Installer Readiness | Release operations | Ready | I046 release follow-up record; existing `v0.1.1` Release preserved | `v0.1.2` can be tagged and published with musl Linux artifacts, simplified archive names, matching installers, and verified checksums. |
| `CONF-002` First-Run Model Configuration Onboarding | Configuration | Ready with dependency note | I045 delivered the CLI config primitives and inline-key boundary; TUI `/config` remains out of scope | Unconfigured interactive users get guided provider/credential/model setup instead of entering a broken first turn. |
| `OBS-001` Observability And Prompt Asset Prerequisite | I018 / I019 prerequisite | Ready | ADR-014 and ADR-015 accepted | Bounded file logs and compile-time embedded prompt assets land before memory storage/retrieval expands logs and prompt surface area. |
| `MEM-001-A` Memory Foundation Starter | MEM-001 / I019 | Ready after OBS-001 closure | I024 delivered working/episodic session wiring; ADR-016 accepted; OBS-001 must land first in I047 | A minimal memory module/crate boundary, SQLite schema, ADD-only semantic/procedural records, and bounded retrieval API exist with provenance tests, but no automatic prompt injection yet. |
| `MEM-005-A` Boundary-Aware Compaction Policy | MEM-005 | Ready as Phase 1 slice | MEM-002 complete; full MEM-003 LLM layers not required | Existing compaction layers 1-3 run under explicit trigger policy with status, manual control, and safe failure behavior. |
| `GOV-003-A` Read-Only Governance Status | GOV-003 | Ready as Phase 1 slice | CMD-001 complete; full WEB-001 UI not required | `/agile status` or equivalent command reports iteration/backlog/board/validation summary without modifying governance docs. |
| `I047-S5` Release Closeout And Documentation Sync | I047 | Ready | S1-S4 implemented and verified | README/zh-CN README, backlog owners, iteration record, board, and release notes agree before tagging `v0.1.2`. |

### Story Slices

| Slice | Owner Story | Deliverable | Verification |
|---|---|---|---|
| `I047-S1A` | REL-001 | Release matrix audit: supported targets, artifact names, installer URLs, checksum names, and unsupported Windows ARM64 behavior are written down before any tag work. | Static audit of `build.sh`, release workflow, installers, README download text. |
| `I047-S1B` | REL-001 | Local packaging smoke produces the final expected file set, or records which target subset was validated locally and why. | `./build.sh` or documented subset; `dist/checksum.sha256` contains every generated archive. |
| `I047-S1C` | REL-001 | Installer dry-run path proves URL construction without requiring a real tag push. | Mocked base URL or controlled failing version shows expected archive names before network failure. |
| `I047-S2A` | CONF-002 | Startup pre-flight classifies config state: usable, missing provider, missing credential, missing model, non-interactive. | Unit tests in `talos-config`/CLI startup code. |
| `I047-S2B` | CONF-002 | Guided setup writes provider, credential reference or inline key, and model through one config API path. | Temp-home runtime test verifies saved TOML and masked display. |
| `I047-S2C` | CONF-002 | `talos init` re-runs setup; CI/non-interactive mode exits clearly or respects a no-init flag. | CLI tests for TTY and non-TTY behavior; no prompt hang. |
| `I047-S3A` | OBS-001 | Bounded file logging config and rotation/retention behavior are implemented under ADR-014. | Config tests plus runtime/log-rotation test prove files cannot grow unbounded. |
| `I047-S3B` | OBS-001 | Built-in prompt text moves to standalone repository assets embedded at compile time under ADR-015. | Tests prove required prompt assets exist, are non-empty, and provider prompt assembly still works. |
| `I047-S3C` | OBS-001 | I019 prerequisite gate is recorded: I024/MEM-002 complete, OBS-001 complete, I019 can activate in a later iteration if desired. | I018/I019/MEM-001 owners and Board agree on prerequisite status. |
| `I047-S4A` | MEM-001-A | Define memory layer types and module/crate boundary for working, episodic, semantic, and procedural concepts. | Compile-time API tests prove no circular dependency and raw session records stay the episode source. |
| `I047-S4B` | MEM-001-A | Add SQLite-backed semantic/procedural memory schema with evidence links, ADD-only writes, confidence, timestamps, and contradiction metadata fields. | Migration/schema tests cover insert, exact dedup, conflict preservation, and evidence links. |
| `I047-S4C` | MEM-001-A | Implement bounded retrieval API using FTS5 + recency + evidence scoring at minimum; no vector/graph dependency. | Retrieval tests prove bounds, provenance, conflict ranking, and stable ordering. |
| `I047-S5A` | MEM-005-A | Policy object documents threshold math, limit source precedence, output reserve, and reasoning reserve placeholder. | Unit tests for threshold cases. |
| `I047-S5B` | MEM-005-A | Pre-turn boundary-aware compaction applies existing layers 1-3 only at safe boundaries unless hard overflow requires refusal/fallback. | Mock session tests for safe boundary, mid-tool deferral, hard overflow. |
| `I047-S5C` | MEM-005-A | Manual compaction command reports compacted/skipped/failed status without exposing hidden tool output. | TUI/command tests and hidden-output regression. |
| `I047-S6A` | GOV-003-A | Governance reader parses a bounded summary from standard docs and tolerates missing files. | Unit tests for empty, partial, and Talos-governed workspaces. |
| `I047-S6B` | GOV-003-A | Read-only command renders iteration/backlog/board/validation state and never writes docs. | Runtime command test plus dirty-worktree guard. |
| `I047-S7A` | I047-S5 | Closeout docs and release rehearsal agree on behavior, install names, I019 prerequisites, and residuals. | README/zh-CN README/Board/backlog/iteration diff review plus governance validation. |

### Timebox And Execution Order

| Week | Focus | Exit Checkpoint |
|---|---|---|
| Week 1 | `REL-001` release/install chain and local package smoke | Expected archives and checksums are produced locally; installer URL construction is tested without moving tags. |
| Week 2 | `CONF-002` first-run setup plus `OBS-001` prerequisite closure | Empty-config interactive path writes a usable provider/model config; non-interactive mode never hangs; bounded logs and embedded prompt assets are in place. |
| Week 3 | `MEM-001-A` memory foundation starter after prerequisites are met | Memory layer boundary, SQLite schema, ADD-only write semantics, and bounded retrieval tests exist without prompt injection. |
| Week 4 | `MEM-005-A`, `GOV-003-A`, docs sync, and release rehearsal | Compaction policy and governance status command work; I019 prerequisite status is recorded; `v0.1.2` release checklist is ready. |

### Weekly Planning Detail

#### Week 1: Release Surface

- Confirm target matrix:
  - `x86_64-pc-windows-msvc` -> `talos-x86_64-windows.zip`
  - `x86_64-unknown-linux-musl` -> `talos-x86_64-linux.tar.gz`
  - `aarch64-unknown-linux-musl` -> `talos-aarch64-linux.tar.gz`
  - `x86_64-apple-darwin` -> `talos-x86_64-darwin.tar.gz`
  - `aarch64-apple-darwin` -> `talos-aarch64-darwin.tar.gz`
- Confirm `scripts/install.sh` and `scripts/install.ps1` construct exactly those names.
- Add a dry-run or testable helper only if it is needed to validate installers without a network
  release.
- Record whether local validation covered every target or a documented subset. A subset is
  acceptable for planning, but the release cannot close until CI validates the full matrix.

#### Week 2: First-Run Setup

- Classify startup config state before entering a provider turn.
- Prefer env-var credential references when users choose them; allow inline `api_key` when chosen,
  preserving ADR-023 masking rules.
- Keep the wizard narrow: provider, credential, model, optional connectivity test, confirmation.
- Ensure cancellation has a clean exit path and does not leave partial config writes unless the
  user confirmed a completed config.

#### Week 2: I019 Prerequisite Closure

- Complete I018/OBS-001 inside I047 before memory implementation work begins:
  - File logging has in-process rotation/cleanup and bounded retention.
  - TUI file logging cannot grow unbounded.
  - Built-in prompt text lives in standalone repository assets and is embedded at compile time.
  - Runtime user-editable prompt packs remain out of scope.
- Record the prerequisite gate explicitly:
  - I024/MEM-002 working + episodic wiring is already complete.
  - OBS-001 is completed in I047.
  - I019 may activate after I047 without being blocked on I018.

#### Week 3: Memory Foundation Starter

- Do not start `MEM-001-A` until the I019 prerequisite closure above is complete or a change-control
  record explicitly changes the dependency.
- Start memory as a small executable foundation, not a full autonomous memory product.
- Add or refine a memory module/crate boundary that can represent the four ADR-016 layers without
  coupling `talos-core` to storage crates.
- Implement the first SQLite schema for semantic/procedural memory items:
  - ADD-only records; no semantic overwrite/delete path.
  - `kind`, `key`, `content`, `confidence`, `created_at`, `last_reinforced`, optional
    `last_accessed`, contradiction metadata, and source/evidence links.
  - FTS5 index where available through bundled SQLite.
- Implement bounded retrieval that returns content plus provenance and score explanation. Retrieval
  may be library-only in I047; automatic prompt injection is explicitly deferred.
- Keep entity linking as optional schema shape or future extension unless it fits without expanding
  beyond the timebox.

#### Week 4: Context Policy, Governance Status, And Closeout

- Implement policy around current compaction layers rather than introducing new summarization.
- Treat hidden tool output as model-context material only; never reprint it into scrollback during
  compaction status or summaries.
- Keep raw session history as the durable source of truth.
- Make failure modes explicit: skip, defer, safe fallback, or clear refusal when context cannot fit.
- Keep `/agile status` read-only. It may report missing/stale docs but must not repair them.
- Bound output size so the command remains useful in large repositories.
- Run release rehearsal only after installer/setup/compaction/governance docs are synchronized.
- Decide whether to tag `v0.1.2` only after validation evidence is recorded and reviewed.

### Cross-Cutting Constraints

- Config writes must go through `talos-config`; no ad hoc TOML editing in CLI/TUI code.
- Secret display surfaces must mask `api_key`, including wizard confirmation, config list/get,
  debug output, logs, and governance/release diagnostics.
- Write-capable commands remain permission-gated; new commands that only read status should be
  explicit about being read-only.
- Native/C dependency behavior must degrade safely. Release work must not add a new native build
  dependency without an ADR or an explicit decision record.
- Provider/network checks must be optional and cancellable. A failed connectivity test should not
  erase a valid manually entered config.
- Memory writes are persistent local state. They must be auditable, schema-versioned, and linked to
  evidence; no prompt-injected memory may lack provenance.
- Memory retrieval must be bounded before any future prompt injection. I047 may expose retrieval as
  a library/API surface without wiring it into every provider request.
- Governance docs remain owner-first: update backlog or iteration owners before updating
  `docs/BOARD.md`.

### Definition Of Done

- All selected slices have implementation evidence or an explicitly recorded defer/split decision.
- `cargo check`, `cargo clippy`, `cargo test`, governance validation, and release-specific smoke
  checks are recorded in this file.
- User-facing docs describe only behavior that actually shipped.
- Existing `v0.1.1` Release/tag remains untouched unless a later user decision changes strategy.
- The next release is either tagged as `v0.1.2` with successful workflow evidence, or closeout
  records why tagging is deferred and what remains.

### Scope

- Keep the existing `v0.1.1` Release and remote tag untouched; use `v0.1.2` for the next stable release.
- Verify and, where needed, repair archive naming across `build.sh`, release workflow notes, and installers.
- Add an interactive first-run model setup path and re-runnable `talos init` path.
- Preserve inline `api_key` behavior from ADR-023: persisted locally, masked in all display surfaces.
- Complete all known prerequisites for I019:
  - I024/MEM-002 is already complete and remains the working/episodic source.
  - I018/OBS-001 bounded logs and embedded prompt assets are delivered in I047.
  - I019 activation status is updated after owner docs record the prerequisite closure.
- Open the ADR-016 memory system with a starter implementation: memory layer types, SQLite schema,
  ADD-only semantic/procedural writes, evidence links, and bounded retrieval API.
- Implement MEM-005 Phase 1 only: deterministic thresholds, pre-turn ordering, manual `/compact`
  command or equivalent command path, TUI/status feedback, and failure fallback around existing
  compaction layers 1-3.
- Implement GOV-003 Phase 1 only: read standard governance files, summarize current iteration,
  backlog, board, decisions, and validation state, and display it through a read-only command.
- Keep every write-capable path gated by the existing permission/config pipeline.

### Non-Goals

- Do not move, delete, or overwrite `v0.1.1`.
- Do not re-enable Windows ARM64 release builds.
- Do not migrate `ring`/`reqwest`/TLS dependencies to `native-tls`; that requires a separate
  dependency strategy story and likely ADR update.
- Do not implement full MEM-003 LLM summarization layers 4-5.
- Do not complete the full I019 memory baseline in I047: no automatic semantic consolidation from
  every session, no procedural skill adaptation, no vector/graph index, and no autonomous research
  library integration.
- Do not implement the full `GOV-003` web UI, governance initialization, or auto-repair flow.
- Do not implement TUI `/config` editing unless it is strictly needed for the first-run wizard.
- Do not activate delayed/scheduled tasks, remote session control, WASM plugins, or the exploration
  library in this iteration.

### Acceptance

- Given a clean checkout on `main`
  When the release package script runs for supported targets
  Then it produces only the documented archive names plus `checksum.sha256`.

- Given the existing `v0.1.1` Release and remote tag
  When the team prepares the next stable release
  Then the plan uses `v0.1.2` and leaves `v0.1.1` history untouched.

- Given an interactive user with no usable model/provider credential configured
  When they start Talos normally
  Then Talos runs a guided setup before entering the session and persists the result through
  `talos-config`.

- Given a non-interactive or CI environment with no usable model configuration
  When Talos starts
  Then it exits with an actionable message or respects an explicit no-init escape hatch without
  blocking on prompts.

- Given I019's prerequisite gate
  When I047 reaches the memory implementation checkpoint
  Then I024/MEM-002 is confirmed complete, OBS-001 is complete, and I018/I019 owner docs record
  that I019 is no longer blocked on observability/prompt assets.

- Given TUI file logging is enabled
  When logs exceed configured retention bounds
  Then Talos rotates or cleans up logs in-process and does not grow local log files without bound.

- Given built-in prompts are loaded
  When Talos builds provider prompt context
  Then prompt assets come from standalone embedded files with tests proving required assets exist.

- Given semantic/procedural memory records are written
  When the same key appears with conflicting evidence
  Then Talos preserves both entries, links each to evidence, and retrieval ranks rather than
  overwrites them.

- Given a memory retrieval query
  When matching records exist
  Then Talos returns a bounded result set with provenance, confidence/freshness scoring, and stable
  ordering without requiring vector or graph dependencies.

- Given an active session approaches the configured context threshold
  When a safe boundary is reached before the next provider call
  Then Talos applies the documented compaction policy or reports why compaction was skipped.

- Given a user invokes the manual compaction command
  When compaction is eligible
  Then Talos applies bounded compaction without exposing hidden tool output into scrollback.

- Given a governed Talos workspace
  When the user invokes the governance status command
  Then Talos reports current iteration/backlog/board/validation state read-only and degrades
  gracefully when expected docs are missing.

- Given I047 reaches release closeout
  When `README.md`, `README.zh-CN.md`, backlog owners, iteration records, and `docs/BOARD.md` are
  inspected
  Then they agree on install names, first-run setup, compaction controls, governance status, and
  release strategy.

### Planned Validation

- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `scripts/validate_project_governance.sh .`
- Local packaging smoke: run `./build.sh` or a documented supported-target subset and verify
  archive names plus `checksum.sha256`.
- Installer validation: shell installer syntax, PowerShell script parse, and mocked or dry-run URL
  construction for Linux/macOS/Windows x86_64.
- Runtime setup scenario: temp `HOME` with no Talos config, run `talos init` or equivalent guided
  setup through the binary, verify masked credential display and config round-trip.
- Runtime prerequisite scenario: bounded log rotation/cleanup and embedded prompt asset tests pass
  before `MEM-001-A` is implemented.
- Runtime/library memory scenario: schema migration, ADD-only insert, conflict preservation,
  bounded retrieval, and provenance links are proven through focused tests.
- Runtime compaction scenario: deterministic test or mock session proving threshold, manual command,
  skipped case, and failure fallback.
- Runtime governance scenario: governed workspace returns read-only status; empty workspace degrades
  without panic or write.
- Release rehearsal: confirm no tag is pushed until all validation evidence is recorded.

### Documentation To Update

- `README.md`
- `README.zh-CN.md`
- `docs/backlog/PRODUCT-BACKLOG.md`
- `docs/backlog/active/REL-001-release-installer-readiness.md`
- `docs/backlog/active/CONF-002-model-onboarding.md`
- `docs/backlog/active/OBS-001-observability-prompt-assets.md`
- `docs/backlog/active/MEM-001-layered-memory-foundation.md`
- `docs/backlog/active/MEM-005-context-compaction-policy.md`
- `docs/backlog/active/GOV-003-builtin-project-governance.md`
- `docs/iterations/I047-v012-release-readiness-and-runtime-polish.md`
- `docs/iterations/README.md`
- `docs/BOARD.md`
- `EVOLUTION.md` only if validation fails, release drift recurs, or a reusable lesson is discovered.

### Release Readiness Checklist

- [ ] No local uncommitted code changes unrelated to I047 release scope.
- [ ] `v0.1.2` version/tag choice confirmed; `v0.1.1` is not moved.
- [ ] Installer scripts parse and construct expected archive names.
- [ ] Release workflow target list matches `build.sh`.
- [ ] Download table in generated release notes matches actual artifacts.
- [ ] `checksum.sha256` includes every artifact and no stale files.
- [ ] README install instructions match `scripts/install.sh` / `scripts/install.ps1`.
- [ ] GitHub Actions release workflow succeeds for the tag.
- [ ] Post-release install smoke is run from the published assets, or failure is recorded with a
  rollback/follow-up plan.

### Risks And Rollback

- Risk: First-run wizard accidentally writes or erases user config.
  Rollback: Use temp-home tests, preserve existing config load/save semantics, and require explicit
  confirmation before overwriting existing provider fields.

- Risk: I019 prerequisite closure expands beyond the month.
  Rollback: Treat OBS-001 as the required prerequisite gate and defer nonessential observability
  enhancements such as structured JSON logs and shared span contracts.

- Risk: Compaction hides needed task context mid-operation.
  Rollback: Phase 1 only compacts at explicit boundaries or manual command unless a hard overflow
  would otherwise fail; raw session history remains recoverable.

- Risk: Memory starter becomes an unbounded mini-I019 and crowds out release readiness.
  Rollback: Stop at schema + bounded retrieval API. Automatic consolidation, prompt injection,
  entity linking, vector/graph acceleration, and procedural adaptation remain follow-up work.

- Risk: Memory records become stale or misleading without enough evidence.
  Rollback: ADD-only semantics preserve conflicts; retrieval must surface provenance and confidence
  instead of presenting a single record as unquestioned truth.

- Risk: Governance status parsing becomes brittle against Markdown drift.
  Rollback: Keep Phase 1 read-only and tolerant; missing or unparsable sections become warnings,
  not crashes or document rewrites.

- Risk: Release rehearsal accidentally mutates `v0.1.1`.
  Rollback: Treat all release validation before closeout as local or `v0.1.2`-scoped; no tag push
  until the release checklist is explicitly approved.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-06-25 | Planning | Created after I046 handoff audit and post-handoff repair. No active iteration exists. Existing planned baselines I018/I019/I020/I028 are preserved and deferred; I047 uses a new ID because its objective is release readiness and runtime polish, not those prior objectives. |
| 2026-06-25 | Planning update | User requested that memory-system implementation start this month. Added `MEM-001-A` as a starter slice while preserving the full I019 baseline for later memory work. |
| 2026-06-25 | Planning update | User clarified that I047 must satisfy every known I019 prerequisite. Added OBS-001/I018 prerequisite closure before `MEM-001-A`; I019 itself remains a preserved future baseline. |
| 2026-06-25 | Planning update | Created long-running task record `docs/tasks/2026-06-25-i047-i019-memory-release-sequence.md` to track I047 prerequisite closure, later I019 activation decision, and I020 dependency disposition. |
| 2026-06-25 | **Activation** | I047 activated. Execution contract confirmed (T0): work on `main`, auto-commit + push after phase gates, continuous pace through closeout, `talos init` subcommand for CONF-002, version bump at closeout, OBS-001 log rotation re-audit before declaring complete. T1 confirmed: planning/installer changes already committed (`c616940`, `1b9a9e4`), governance validation 0 warnings, working tree clean. Board Now moved to I047. Selected stories synchronized. |

## Verification Evidence

- Planning validation to be recorded before activation.

## Variance And Residuals

- This is a one-month plan. If implementation starts with only a subset, split the unselected
  stories into a new iteration rather than silently narrowing this baseline.

## Retrospective

- Outcome: Pending.
- Documentation: Pending.
- Lessons: Pending.
