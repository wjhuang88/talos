# 2026-07-04 Architect-Owned Four-Month High-Risk Execution

> Status: In Progress
> Created: 2026-07-04
> Owner boundary: direct senior-agent execution required
> Trigger: user requested a four-month high-risk, high-difficulty, unattended task plan for work
> that should not be delegated to frontline implementation.
> Baseline rule: this file is the execution contract. Append checkpoints instead of replacing the
> plan. Changed objectives use a new task or iteration ID.

## Outcome

Execute a four-month, high-risk Talos hardening track that focuses on architecture, permission,
extension, context, and release/self-bootstrap boundaries that require direct senior-agent
handling. The program should reduce the risk of hidden permission bypasses, unbounded ingestion,
unsafe plugin/distribution behavior, context corruption, and misleading release claims.

This task does not supersede the product hardening plan. It is the direct-owner track for work that
is not suitable for routine frontline delegation.

## In Scope

- Finish or explicitly pause I085 before activating a new iteration.
- Plan and execute four high-risk monthly iterations:
  - I090: tool/ingestion permission boundary and bounded document extraction design.
  - I091: local plugin/hook/distribution safety boundary.
  - I092: active context compression and autonomy permission gates.
  - I093: self-bootstrap, runtime SDK, and release posture closeout.
- Use existing backlog owners where possible: `WEBFETCH-001`, `TOOL-011`, `PLUGIN-001`,
  `HOOK-001`, `DIST-001`, `MEM-007`, `SCHED-001`, `PERM-001`, `RUNTIME-001`, `REL-002`,
  `GOV-003`, and `ARCH-030`.
- Create checkpoints before each phase transition.
- Keep implementation slices local, test-backed, and recoverable.

## Out Of Scope

- No crate publish, release tag, GitHub Release, or version-history mutation.
- No push unless the user explicitly asks at that time.
- No destructive cleanup outside test fixtures.
- No network spend, credential use, remote plugin install, marketplace behavior, remote dashboard,
  browser automation, PDF/Office/OCR dependency, or permission-default change.
- No new native/runtime dependency without an existing accepted ADR or a new ADR accepted first.
- No v1.0 readiness claim unless `REL-002` evidence is complete.

## Ordered Task Items

| ID | Task | Expected Output | Depends On | Completion Gate | Fallback | Status |
|---|---|---|---|---|---|---|
| A0 | Establish execution contract | This task record plus I090-I093 planned iteration shells exist; Board and iteration index point to the track. | User request | `scripts/validate_project_governance.sh .` and `git diff --check` pass. | Keep task in Planned if governance fails. | In Progress |
| A1 | Close or pause I085 | MC107 disposition recorded: README depth and real TUI `/model`/`/connect` verification either complete or explicitly paused. Catalog.db first-run creation semantics must be correct before closeout. | A0 | I085 owner docs and Board agree; workspace checks pass if code changes. | Pause I085 with exact residual owner before activating I090. | Complete |
| A2 | Activate I090 | Start the month-1 high-risk tool/ingestion boundary iteration. | A1 | Iteration README, Board, and relevant backlog owners mark I090 Active/In Progress. | Keep I090 Planned and record blocker. | Complete |
| A3 | WEBFETCH bounded extraction decision | Decide the first Rust-native bounded extraction slice and explicit unsupported formats. | A2 | Proposal/ADR if needed; permission and size/time budgets named. | Keep WEBFETCH Phase 2+ design-only. | Complete |
| A4 | Implement first bounded extraction slice | A local `document_extract` or equivalent bounded extractor for safe text/HTML/JSON/CSV/Markdown-like inputs, no PDF/Office/OCR/browser. | A3 | Tool tests, permission tests, runtime smoke, docs. | Ship design only; no runtime tool. | Complete |
| A5 | TOOL-011 search stabilization gate | Decide whether ripgrep-backed grep must land before more ingestion work; implement only if required. | A3 | Behavior compatibility tests and no host-`rg` runtime dependency. | Keep current grep and record deferral. | Complete |
| A6 | Activate I091 | Start local plugin/hook/distribution boundary iteration. | A4/A5 | Owner docs synchronized and I091 Active. | Keep I091 Planned with blocker. | Complete |
| A7 | Plugin/hook diagnostics hardening | Local plugin and hook diagnostics expose state without auto-discovery, remote install, or write-capable tools. | A6 | CLI/TUI/command tests and provenance checks. | Diagnostics-only docs, no runtime change. | Complete |
| A8 | Distribution safety policy | Optional asset/plugin package policy names manifest, checksum, cache, offline/mirror behavior, and consent. | A6 | DIST-001 proposal/ADR update and governance pass. | Defer runtime distribution. | In Progress |
| A9 | Activate I092 | Start context compression and autonomy permission iteration. | A7/A8 | Owner docs synchronized and I092 Active. | Keep I092 Planned with blocker. | Planned |
| A10 | MEM-007 cache-safe compression prototype | Deterministic pre-entry compression prototype for selected tool outputs, preserving stable prefix and raw export. | A9 | Stable-prefix hash test, determinism test, raw-output export proof, token-savings report. | Reject strategy and keep MEM-005 only. | Planned |
| A11 | Autonomy permission packet | SCHED-001/PERM-001/TOOL-010 are split into non-bypass slices with deny/ask/allow tests before any write/execute scheduling ships. | A9 | Permission regression matrix passes. | Keep features disabled/research-only. | Planned |
| A12 | Activate I093 | Start self-bootstrap/runtime/release closeout iteration. | A10/A11 | Owner docs synchronized and I093 Active. | Keep I093 Planned with blocker. | Planned |
| A13 | Runtime SDK and governance readiness audit | RUNTIME-001/GOV-003/ARCH-030 audit names the minimum self-bootstrap gaps. | A12 | Readiness report updated with concrete gaps and tests. | Keep pre-1.0 posture unchanged. | Planned |
| A14 | REL-002 self-bootstrap rehearsal | Record one Talos-on-Talos rehearsal packet or a failed rehearsal with exact gap evidence. | A13 | REL-002 evidence table updated; no v1.0 claim unless criteria met. | Record non-qualifying evidence. | Planned |
| A15 | Final closeout | Four-month matrix, residual owners, release posture, docs, Board, and handoff are synchronized. | A14 | workspace tests, clippy, governance, final checkpoint. | Mark Partial with exact unfinished owners. | Planned |

## Dependencies And Prerequisites

- I085 was explicitly paused on 2026-07-04 with MC107 manual TUI walkthrough as the only residual,
  satisfying the activation gate for I090 without erasing the residual.
- I086-I089 remain valid product-hardening planned shells; this task does not erase them.
- R27 and the architect-owned high-risk group are standing gates. This user request resumes direct
  senior-agent planning, but does not authorize restricted actions.
- Existing ADRs remain binding, especially ADR-008, ADR-009, ADR-010, ADR-013, ADR-016, ADR-020,
  ADR-021, ADR-023, ADR-024, ADR-026, ADR-027, ADR-028, ADR-029, ADR-030, ADR-031, ADR-032,
  and ADR-034.

## Artifacts And State Owners To Update

- This task record.
- Iteration shells: I090-I093.
- `docs/iterations/README.md`.
- Backlog owners for touched stories.
- `docs/BOARD.md` after owner docs.
- ADRs under `docs/decisions/` for dependency, protocol, permission, storage, or release-boundary
  decisions.
- README/site docs only when user-visible behavior changes.
- `EVOLUTION.md` only for reusable lessons from failed validation or user corrections.

## Validation And Acceptance Evidence

Every implementation phase must run:

```sh
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace -- -D warnings
cargo test --workspace
scripts/validate_project_governance.sh .
git diff --check
```

Planning-only phase gates may use:

```sh
scripts/validate_project_governance.sh .
git diff --check
```

Packet-specific evidence:

- Ingestion: permission facets, content-type/body-sniffing classification, size/time budgets,
  unsupported-format behavior, and no silent persistence.
- Plugin/distribution: package-root confinement, provenance, checksum/cache policy, no remote
  install, no write-capable tools.
- Compression: stable-prefix hash unchanged, deterministic output, raw export preservation,
  token-savings measurement.
- Autonomy: deny/ask/allow matrix for schedule, batch, exec, and write paths.
- Self-bootstrap: runtime used, files changed, validation evidence, docs sync, and whether Codex
  remained primary executor.

## Branch, Worktree And Checkpoint Plan

- Work in the current worktree unless the user explicitly requests a branch.
- Use one logical commit per completed phase if commits are requested.
- Do not push unless the user explicitly asks at that time.
- Append a checkpoint before moving between A1/A2, A5/A6, A8/A9, A11/A12, and A15.
- Recovery starts from this file, then `docs/BOARD.md`, then the active iteration owner.

## Allowed Permissions And External Actions

Allowed by this contract:

- Edit repository files in the workspace.
- Run local build, lint, tests, governance checks, and runtime smoke tests.
- Create and update docs, backlog, iteration, ADR/proposal files.
- Commit locally only if the user requests a commit during the run.

Not allowed without separate explicit approval:

- Push commits, tags, or release artifacts.
- Publish crates or GitHub Releases.
- Add major runtime/native dependencies.
- Use credentials, paid/network services, remote dashboards, marketplace/package installs, or
  browser automation.
- Perform destructive data operations outside temporary test fixtures.

## Destructive Or Irreversible Operations

No destructive or irreversible production operation is authorized. Destructive behavior is limited
to temporary test fixtures and must be covered by tests.

## Time, Cost And Resource Limits

- Timebox: 16 weeks of planned work, executed incrementally through checkpoints.
- Monetary spend: zero.
- Network: avoid by default; official/current-source checks require explicit reason and no
  credentials/spend.
- Retry deterministic failures at most twice after concrete fixes before recording a blocker.
- Stop rather than weaken permission, storage, cache, dependency, or release gates.

## Failure, Retry And Fallback Policy

- If I085 cannot be closed, pause it with exact residuals before activating I090.
- If extraction needs PDF/Office/OCR/browser/heavy deps, stop for ADR or defer.
- If plugin/distribution cannot prove provenance and confinement, keep diagnostics-only.
- If compression changes stable-prefix bytes or loses raw output, reject the strategy.
- If scheduled/autonomous execution cannot prove denial behavior, keep it disabled.
- If self-bootstrap evidence remains Codex-primary, record non-qualification under REL-002.

## Default Decisions For Foreseeable Ambiguity

- Prefer read-only/status/diagnostic slices before write-capable or executable behavior.
- Prefer config-disabled and opt-in defaults.
- Prefer Rust-native deterministic logic over native or remote dependencies.
- Prefer preserving raw data and adding bounded model-facing summaries.
- Prefer recording a blocker over broadening scope silently.

## Residual-Work Destination

- I085 residuals: I085 and MC-001.
- Ingestion/search: WEBFETCH-001, TOOL-011, TOOL-012, TOOL-013, TOOL-014.
- Plugin/distribution: PLUGIN-001, HOOK-001, DIST-001.
- Compression/autonomy: MEM-007, MEM-005, PERM-001, SCHED-001, TOOL-010.
- Runtime/self-bootstrap/release: RUNTIME-001, GOV-003, REL-002, ARCH-030.

## Unattended Execution Policy

The user asked for unattended execution. This task may continue without further user prompts only
inside the allowed local-edit/local-validation boundary above. The run must stop for separate
approval before any push, publish, tag, dependency expansion, credentialed network use, destructive
operation, permission-default change, remote/plugin install, browser automation, or v1.0 claim.

## Checkpoints

### A0 — Contract Drafted (2026-07-04)

Completed task items:

- Created the long-running execution contract.
- Selected four direct-owned high-risk monthly iterations: I090-I093.
- Preserved I085 as the current Active gate; I090 cannot activate until I085 is complete or paused.

Current state and artifacts:

- This file is the owner record.
- Iteration shells and derived views are created in the same planning change.

Commands/checks and actual results:

- Pending in this checkpoint; run after all docs are written.

Open risks or deviations:

- No code implementation begins until I085 closeout state is resolved.
- No push/release/destructive/network actions are authorized by this task.

Next task item:

- A1: close or pause I085 MC107 residual, then activate I090.

Recovery or resume instruction:

- Run `git status --short`.
- Read this file, `docs/BOARD.md`, `docs/iterations/README.md`, and
  `docs/iterations/I085-model-catalog-modernization.md`.
- Continue with A1 unless the user explicitly changes priority.

### A1 — Catalog Creation Semantics Corrected (2026-07-04)

Completed task items:

- Corrected the I085 catalog lifecycle requirement after maintainer clarification.
- Fresh installs now implicitly create and seed `~/.talos/catalog.db` from packaged `models.toml`
  on first catalog access.
- Existing corrupt or incompatible catalog DBs are not overwritten; startup degrades to built-in
  data.
- `talos --import-models <path>` now refreshes/seeds the SQLite catalog from models.dev JSON and
  still writes the legacy JSON cache as a compatibility side effect.

Current state and artifacts:

- Code: `crates/talos-cli/src/model_lifecycle.rs`, `crates/talos-cli/src/main.rs`,
  `crates/talos-cli/src/tests.rs`, `crates/talos-cli/src/mode_runners.rs`.
- Owner docs: I085, MC-001, Board updated to the implicit-create semantics.

Commands/checks and actual results:

- `cargo test -p talos-cli import_models_creates_and_seeds_catalog_db -- --nocapture`: passed.
- `cargo test -p talos-cli open_catalog_snapshot_missing_file_creates_seeded_catalog -- --nocapture`: passed.
- `cargo test -p talos-cli`: 131 unit tests + CLI integration tests passed.
- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy -p talos-cli -- -D warnings`: passed.
- `scripts/validate_project_governance.sh .`: 0 warnings.
- `git diff --check`: clean.

Open risks or deviations:

- MC107 manual real-terminal `/connect` walkthrough remains open and is recorded as the I085 pause
  reason. README onboarding is closed.

Next task item:

- A2: activate I090.

Recovery or resume instruction:

- Run `git status --short`.
- Continue with MC107 manual TUI/README disposition.

### A2 — I090 Activated (2026-07-04)

Completed task items:

- Activated I090 after I085 was explicitly paused with only MC107 real-terminal `/connect`
  walkthrough remaining.
- Preserved I086-I089 as planned product-hardening shells and I091-I093 as planned direct-owner
  shells.
- Started A3 with an audit-first stance because local evidence shows `document_extract`,
  `fetch_url`/`save_url`, and ripgrep-backed `grep` already exist.

Current state and artifacts:

- `docs/iterations/I090-high-risk-ingestion-search-boundary.md` is Active.
- Board, iteration index, WEBFETCH-001, and TOOL-011 are synchronized in the same phase change.

Commands/checks and actual results:

- `scripts/validate_project_governance.sh .`: 0 warnings.
- `git diff --check`: clean.

Open risks or deviations:

- Need to distinguish already-landed implementation from unmet I090 acceptance before writing code.
- No browser/PDF/Office/OCR/crawler scope is authorized.

Next task item:

- A3: audit WEBFETCH/document extraction/search acceptance and decide whether code or docs-only
  closeout is needed.

Recovery or resume instruction:

- Run `git status --short`.
- Read I090, WEBFETCH-001, TOOL-011, `crates/talos-tools/src/document_extract.rs`,
  `crates/talos-tools/src/search_tools.rs`, and `crates/talos-tools/src/search_engine.rs`.

### A3/A4 — Bounded Local Extraction Slice Closed (2026-07-04)

Completed task items:

- Audited the existing `document_extract` implementation before expanding scope.
- Confirmed the first safe local slice already supports bounded text, Markdown, HTML, JSON, JSONL,
  CSV, TSV, and XML extraction.
- Fixed the high-risk gap where ASCII-like PDF, image, Office, or archive bytes could be treated as
  text when no NUL byte was present.
- Added explicit PDF/Office/image/archive extension and magic-byte classification that returns
  metadata-only unsupported output and does not dump embedded bytes.

Current state and artifacts:

- Code: `crates/talos-tools/src/document_extract.rs`.
- Owner docs: I090 and WEBFETCH-001 updated with A3/A4 evidence and residual scope.
- No PDF parser, Office parser, OCR, browser automation, crawler behavior, or heavy conversion
  dependency was added.

Commands/checks and actual results:

- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy -p talos-tools -- -D warnings`: passed.
- `cargo test -p talos-tools document_extract`: 31 matching unit tests passed.
- `cargo test -p talos-tools --test document_boundaries`: 15 tests passed.

Open risks or deviations:

- Full `cargo clippy --workspace -- -D warnings` and `cargo test --workspace` remain the final
  implementation-phase gate for I090 closeout; this checkpoint used targeted tool-crate gates to
  keep the phase small.
- TOOL-011 search stabilization still needs audit before broader ingestion depends on search.

Next task item:

- A5: audit ripgrep-backed `grep` behavior against TOOL-011, decide whether it blocks I090 closeout,
  and either fix the minimum gaps or record a precise deferral.

Recovery or resume instruction:

- Run `git status --short`.
- Read I090, TOOL-011, `crates/talos-tools/src/search_tools.rs`, and
  `crates/talos-tools/src/search_engine.rs`.

### A5 — Ripgrep Search Stabilization Closed (2026-07-04)

Completed task items:

- Audited the existing ripgrep-backed search path against TOOL-011 instead of assuming it was
  complete.
- Closed the bounded-search gaps: file-count budget, per-file byte budget, total input-byte
  budget, total output-byte budget, per-line output budget, elapsed-time gate, compact search
  summary, binary skip summary, oversized skip summary, and controlled timeout error.
- Added regression coverage for `.ignore`, binary skip, oversized skip, invalid UTF-8, max-result
  truncation, output-byte truncation, symlink-not-followed default behavior, path escape, fixed
  fixture parity, and Talos-repo query parity.
- Preserved `GrepInput` compatibility and kept the runtime path self-contained with no host `rg`.

Current state and artifacts:

- Code: `crates/talos-tools/src/search_engine.rs`, `crates/talos-tools/src/search_tools.rs`.
- Owner docs: I090 and TOOL-011 updated; I090 is Complete.

Commands/checks and actual results:

- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy -p talos-tools -- -D warnings`: passed.
- `cargo test -p talos-tools grep_tool_tests`: 13 tests passed.
- `cargo test -p talos-tools search_engine::regression_tests`: 18 tests passed.
- `cargo test -p talos-tools`: 225 tests passed.
- `cargo test --workspace`: passed.
- `scripts/validate_project_governance.sh .`: passed, 0 warnings.
- `git diff --check`: clean.

Open risks or deviations:

- The `SearchOutput` struct gained bounded-search statistics as part of TOOL-011 stabilization;
  downstream callers should treat those fields as the committed search contract for this story.

Next task item:

- A6: after this phase commit/push, activate I091 in a separate phase commit.

Recovery or resume instruction:

- Run `git status --short`.
- Read I090, I091, TOOL-011, and this task's A5 checkpoint.

### A6 — I091 Activated (2026-07-04)

Completed task items:

- Activated I091 after I090 completed full workspace/governance closeout.
- Preserved I085 as Paused with only MC107 real-terminal `/connect` walkthrough residual.
- Preserved I086-I089 as planned product-hardening shells and I092-I093 as planned direct-owner
  shells.
- Confirmed I091 starts with audit-first diagnostics/policy work, not runtime expansion.

Current state and artifacts:

- `docs/iterations/I091-plugin-hook-distribution-boundary.md` is Active.
- PLUGIN-001, HOOK-001, DIST-001, Board, and iteration index are synchronized in this phase.

Commands/checks and actual results:

- `scripts/validate_project_governance.sh .`: passed, 0 warnings.
- `git diff --check`: clean.

Open risks or deviations:

- No remote install, marketplace, automatic plugin discovery, standalone executable hook carrier,
  write-capable plugin tool, Lua, or dynamic library support is authorized.

Next task item:

- A7: audit local plugin/hook diagnostics and implement only the minimum state/provenance visibility
  gaps needed for I091 acceptance.

Recovery or resume instruction:

- Run `git status --short`.
- Read I091, PLUGIN-001, HOOK-001, DIST-001, and this task's A6 checkpoint.

### A7 — Plugin/Hook Diagnostics Hardened (2026-07-04)

Completed task items:

- Added `/hooks` to the slash command registry and conversation engine as a read-only diagnostics
  command.
- `/hooks` reports config-introduced hooks as not enabled, executable hook carriers as disabled,
  and lists the builtin hook event catalog without executing or loading hook code.
- Added `HookRegistry::registrations()` to expose a read-only diagnostic snapshot of registered
  handlers.
- Extended plugin package manifest parsing with validated `[[hooks]]` declarations, including
  event-name validation and duplicate hook rejection, without instantiating hook carriers.

Current state and artifacts:

- Code: `crates/talos-conversation/src/command_registry.rs`,
  `crates/talos-conversation/src/engine.rs`, `crates/talos-conversation/src/engine_tests.rs`,
  `crates/talos-plugin/src/registry.rs`, `crates/talos-plugin/src/manifest.rs`,
  `crates/talos-plugin/src/lib.rs`, `crates/talos-conversation/Cargo.toml`.
- Owner docs: I091, PLUGIN-001, HOOK-001, CMD-002, and Board updated.

Commands/checks and actual results:

- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy -p talos-conversation -p talos-plugin -- -D warnings`: passed.
- `cargo test -p talos-conversation slash_hooks`: passed.
- `cargo test -p talos-conversation every_visible_slash_command_has_an_execution_path`: passed.
- `cargo test -p talos-plugin manifest`: 16 matching tests passed.
- `cargo test -p talos-plugin registrations_reports_handlers_without_dispatch`: passed.
- `cargo test -p talos-conversation -p talos-plugin`: passed.
- `cargo test --workspace`: passed.
- `scripts/validate_project_governance.sh .`: passed, 0 warnings.
- `git diff --check`: clean.

Open risks or deviations:

- Config-introduced hook execution remains disabled. This phase validates declarations and exposes
  diagnostics only.
- DIST-001 policy remains open for A8.

Next task item:

- A8: write the optional asset/plugin package distribution policy with manifest, checksum, cache,
  offline/mirror, consent, and failure behavior.

Recovery or resume instruction:

- Run `git status --short`.
- Read I091, DIST-001, and this task's A7 checkpoint.
