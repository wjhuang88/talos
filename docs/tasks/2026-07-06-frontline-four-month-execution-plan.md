# 2026-07-06 Frontline Four-Month Execution Plan

**Status**: In Progress
**Created**: 2026-07-06
**Timebox**: 16 weeks / roughly 4 months
**Owner boundary**: frontline implementation package; maintainer reviews phase closeouts
**Trigger**: maintainer requested a four-month long-running task plan suitable for frontline developers.

## Outcome

Deliver a low-ambiguity four-month implementation package that improves daily Talos usability
without asking the receiving developer to decide architecture, security policy, release readiness,
or product scope. The work focuses on bounded UI/display fixes, static documentation/site polish,
tests, walkthrough evidence, and small model-onboarding improvements that already have clear
acceptance.

This plan is a delegation contract. It does not authorize release tags, crate publishing, remote
deployment, permission-default changes, sandbox changes, process-hardening changes, or broad
architecture rewrites.

## In Scope

- TUI display polish with existing rendering boundaries preserved.
- CLI model-list browsing usability for large model catalogs.
- `/model` and `/connect` docs and regression coverage for already-defined behavior.
- Standard-provider connect flow polish where catalog metadata already supplies endpoint/protocol
  defaults.
- Static documentation and product-site updates that do not add new build tools.
- Focused tests and walkthrough evidence for each user-visible change.
- Monthly checkpoints with exact commands, results, commits, residuals, and next-step recovery
  instructions.

## Out Of Scope

- Permission, approval, sandbox, process-hardening, tool execution policy, or bash auto-approval
  semantics.
- Native Git replacement, `gix` dependency upgrades, or host-Git fallback changes.
- Provider streaming protocol redesign, new provider families, or speculative model routing.
- `catalog.db` resurrection or runtime catalog database initialization.
- Plugin runtime expansion, remote plugin install, executable hooks, browser profile reuse, or
  authenticated browser automation.
- Binary session-log migration or storage-format default changes.
- Release tags, GitHub Releases, crate publishing, installer signing, or distribution deployment.
- Any change requiring new external services, credentials, paid API calls, or network-dependent
  tests.

## Required Reads

The receiving developer must read these files before making changes:

1. `AGENTS.md`
2. `docs/sop/LONG-RUNNING-TASK.md`
3. `docs/sop/ITERATION-WORKFLOW.md`
4. `docs/sop/GIT-WORKFLOW.md`
5. `docs/sop/DOC-CHECK.md`
6. `docs/BOARD.md`
7. `docs/backlog/PRODUCT-BACKLOG.md`
8. `docs/backlog/active/MC-002-remove-runtime-catalog-db-residuals.md`
9. `docs/backlog/active/MODEL-006-interactive-model-catalog-browser.md`
10. `docs/backlog/active/MODEL-005-interactive-model-selection.md`
11. `docs/backlog/active/TUI-015-head-tail-truncation.md`
12. `docs/backlog/active/TUI-019-tool-output-visual-hierarchy.md`
13. `docs/backlog/active/TUI-025-tool-argument-line-fit-display.md`
14. `docs/backlog/active/WEB-003-site-internationalization.md`
15. `docs/backlog/active/WEB-004-site-theme-branding.md`

## Operating Rules

- Work in task order. Do not skip ahead unless a task is explicitly blocked and the fallback says
  to continue.
- Before coding each item, restate the exact files expected to change and the tests expected to
  prove the change.
- Preserve existing architecture boundaries. If the fix needs cross-crate API redesign, stop and
  write a blocker instead of implementing.
- Change owner docs before derived docs such as `docs/BOARD.md`.
- Commit only at phase boundaries after `git diff --cached` review.
- Use conventional commits with `[model:<model-name>]`.
- Never claim full workspace validation unless the exact command passed in this worktree.
- If validation cannot run, record the command, failure output summary, environment assumption, and
  fallback validation actually run.

## Ordered Task Items

| ID | Week | Task | Expected Output | Depends On | Completion Gate | Fallback | Status |
|---|---:|---|---|---|---|---|---|
| F0 | 1 | Start inventory | Check current Board/backlog/task state and append a kickoff checkpoint to this file. | None | `scripts/validate_project_governance.sh .` and `git diff --check` pass. | If owner docs conflict, record conflict and stop. | Planned |
| F1 | 1 | Catalog residual audit | Confirm no user-facing flow depends on runtime `catalog.db`; list any leftover names/docs/tests. | F0 | `rg "catalog\\.db|ModelCatalog|models.toml" crates docs README.md` reviewed and findings recorded. | If runtime dependency is found, create a blocker under MC-002 and do not remove blindly. | Planned |
| F2 | 2 | Catalog residual cleanup | Remove stale `catalog.db` docs/code references that are clearly dead after audit. | F1 | Targeted tests for affected crates plus `cargo check --workspace`. | If ownership is unclear, leave code unchanged and document the exact stale reference. | Planned |
| F3 | 3 | `/model` and `/connect` docs sync | Update README/docs so `/model` shows configured/usable models and `/connect` owns provider setup. | F2 | `rg "/model|/connect" README.md README.zh-CN docs -n` reviewed; governance passes. | If current behavior differs, write a behavior gap instead of documenting false behavior. | Planned |
| F4 | 4 | Month 1 closeout | Close catalog and command-doc residuals with evidence. | F1-F3 | `cargo fmt --all -- --check`, `cargo check --workspace`, targeted tests, governance, `git diff --check`. | Mark Partial with residual owner and exact failing command. | Planned |
| F5 | 5 | Standard-provider connect regression | Ensure built-in providers do not ask for base URL; only custom providers do. | F4 | Tests cover standard provider, custom provider, config merge, and masked secret rendering. | If behavior is already covered, link tests and make no code change. | Planned |
| F6 | 6 | Protocol metadata display audit | Verify model/provider protocol metadata from packaged `models.toml` is surfaced where setup needs it. | F5 | Tests or snapshots prove known protocol-backed providers route correctly without user URL input. | If metadata is missing from packaged data, record sync blocker; do not add runtime DB. | Planned |
| F7 | 7 | CLI model list usability | Improve `--available-models` for large catalogs with an independent scroll/search browser or bounded paged output. | F6 | Terminal/manual evidence shows large lists do not flood stdout and entries remain provider-qualified. | If interactive browser is too broad, implement `--available-models --filter`/paging only and record browser residual. | Planned |
| F8 | 8 | Month 2 closeout | Close model setup/listing usability package. | F5-F7 | `cargo test -p talos-cli`, `cargo test -p talos-config`, `cargo check --workspace`, governance. | Mark Partial with exact residuals. | Planned |
| F9 | 9 | Tool argument line-fit display | Improve TUI tool-call parameter rendering so arguments are shown fully when one line has room, truncating only when needed. | F8 | Focused TUI tests cover short args, long one-line args, multi-line args, and secret-safe rendering. | If rendering helper is shared with approval secrets, stop and ask. | Planned |
| F10 | 10 | Head-tail retained lines | When middle elision is triggered, keep only first 3 and last 3 lines without changing the trigger or summary routing. | F9 | Tests prove short outputs stay full, long fallback keeps 3+3, omitted count is correct, export/model payload remains full. | If trigger logic must change, do not implement; record blocker. | Planned |
| F11 | 11 | Tool output visual hierarchy | Make grouped/header text more readable using existing TUI palette constants. | F10 | TUI tests or snapshots cover group/header style; no one-off color literals if palette constants exist. | If contrast target is ambiguous, choose existing high-contrast palette constant and record rationale. | Planned |
| F12 | 12 | Month 3 closeout | Close TUI display package. | F9-F11 | `cargo test -p talos-tui`, `cargo test -p talos-tools`, `cargo check --workspace`, governance. | Mark Partial with screenshot/test residual. | Planned |
| F13 | 13 | Static site i18n inventory | Inventory public site pages and untranslated strings. | F12 | Checklist lists every page and whether zh-CN counterpart exists. | If site validator is missing, record manual validation plan. | Planned |
| F14 | 14 | Static site i18n implementation | Add or update zh-CN static pages using existing assets and relative links. | F13 | Site validator if present, manual link check, no new JS framework/build tool. | If a page is too ambiguous to translate, add a deferral note. | Planned |
| F15 | 15 | Static site branding polish | Apply small static CSS/SVG polish consistent with Talos identity. | F14 | Visual/manual evidence; no remote assets, analytics, fonts, or build tooling. | If design direction is unclear, limit to contrast/accessibility fixes. | Planned |
| F16 | 16 | Final closeout | Produce final handoff closeout with commits, validation, residuals, and next-cycle candidates. | F0-F15 | `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo test --workspace`, governance, `git diff --check`. | Close as Partial only with exact failed gate and owner for every residual. | Planned |

## Detailed Acceptance Standards

### Catalog Residuals

- No new `catalog.db` code path, doc instruction, installer step, or user-visible initialization
  path is introduced.
- Packaged catalog behavior remains based on embedded/static `models.toml` or existing built-in
  model metadata.
- Any remaining `ModelCatalog` type or test is classified as either active library capability,
  planned removal, or dead residual. Do not delete active library capability without maintainer
  confirmation.

### `/model` And `/connect`

- `/model` must not advertise unconfigured providers as selectable ready models.
- `/connect` is the setup path for unavailable providers.
- Standard providers use built-in/catalog endpoint metadata and should not ask the user to type a
  base URL.
- Custom providers still require an explicit base URL.
- Tests must cover both standard and custom provider paths.

### `--available-models`

- Output entries are provider-qualified, for example `provider/model`.
- Large catalogs must not dump an unbounded wall of text by default if an interactive or paged path
  is implemented.
- The implementation remains independent from the main TUI runtime because this command runs before
  entering the primary TUI.
- Filtering/search must match provider name, model id, and provider-qualified id.

### TUI Tool Display

- Display-only changes must not alter the model-visible tool result payload.
- Display-only changes must not alter export/transcript source content unless the owner doc
  explicitly requires it.
- Middle-elision retained lines are exactly first 3 and last 3 after the existing elision condition
  has already selected that path.
- Summary routing remains orthogonal: changing retained lines must not make `grep`, `read`, or other
  specialized summaries enter or leave their summary path.
- Tool argument rendering should show complete args when they fit in available line width and
  truncate only when needed.

### Static Site

- Use existing static structure and assets.
- Keep language links relative.
- Do not introduce remote fonts, analytics, external scripts, package managers, or build steps.
- English pages remain semantically unchanged except language-switch links and corrected stale
  Talos behavior.

## Validation Matrix

Run the smallest targeted checks for each task, then run the phase gates before monthly closeout.

Baseline checks:

```sh
cargo fmt --all -- --check
cargo check --workspace
scripts/validate_project_governance.sh .
git diff --check
```

Full closeout checks:

```sh
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
scripts/validate_project_governance.sh .
git diff --check
```

Optional only when already available in the repo:

```sh
cargo clippy --workspace -- -D warnings
scripts/validate_public_site.sh
```

## Branch, Commit, And Checkpoint Plan

- Work on the maintainer-provided branch/worktree.
- Commit at the end of each monthly closeout item: F4, F8, F12, and F16.
- Use one logical commit per month unless a phase produces clearly separable docs-only and code
  changes.
- Do not push unless explicitly instructed by the maintainer.
- Append checkpoint sections to this file after F4, F8, F12, and F16.

Checkpoint template:

```text
## Checkpoint F<N> - <Title> (<date>)

Completed items:
Commits:
Changed files:
Validation:
Open deviations:
Residual owner:
Next item:
Recovery instructions:
```

## Stop-And-Ask Conditions

Stop and ask the maintainer before continuing if any task appears to require:

- Changing permission defaults, approval reuse semantics, sandbox rules, process hardening, or tool
  execution policy.
- Reintroducing runtime `catalog.db` creation, migration, or initialization.
- Adding a new external dependency for static site or CLI browsing.
- Using real provider credentials or making paid provider calls.
- Deleting public APIs or changing crate semver-bound behavior.
- Broad refactors outside the named owner files.
- Deciding a product behavior not already stated in this plan or the owner docs.

## Handoff Prompt

Use this prompt when assigning the work:

```text
You are taking over the Talos frontline four-month execution plan:
docs/tasks/2026-07-06-frontline-four-month-execution-plan.md

Start with F0 only. Read the required files listed in the plan, inventory current Board/backlog/task
state, and append the kickoff checkpoint. Do not code before F0 is complete.

Follow the task order F0-F16. Each task has an expected output, completion gate, fallback, and stop
conditions. Do not broaden scope. If a change touches permission semantics, sandboxing, process
hardening, runtime catalog.db initialization, release artifacts, or architecture-wide APIs, stop and
record a blocker instead of implementing.

At each monthly closeout, run the required validation, stage only related files, review
`git diff --cached`, and commit with a conventional message including `[model:<model-name>]`.
Do not push unless explicitly instructed.
```

## Residual Destination

- Catalog/model residuals: `docs/backlog/active/MC-002-remove-runtime-catalog-db-residuals.md`,
  `docs/backlog/active/MODEL-006-interactive-model-catalog-browser.md`, or
  `docs/backlog/active/MODEL-005-interactive-model-selection.md`.
- TUI display residuals: `docs/backlog/active/TUI-015-head-tail-truncation.md`,
  `docs/backlog/active/TUI-019-tool-output-visual-hierarchy.md`, or
  `docs/backlog/active/TUI-025-tool-argument-line-fit-display.md`.
- Static site residuals: `docs/backlog/active/WEB-003-site-internationalization.md` or
  `docs/backlog/active/WEB-004-site-theme-branding.md`.
- Any permission/tool-execution concern: do not implement here; record under the relevant
  permission/tool backlog item for maintainer-owned planning.

## Consolidated Confirmation (recorded 2026-07-06)

Executed as one long-running task per `docs/sop/LONG-RUNNING-TASK.md`. One consolidated
confirmation covers the full F0-F16 cycle. Approved contract:

| Decision | Resolution |
|---|---|
| Already-complete tasks (F5-F11) | Verify + run cited tests + record evidence + close. No re-implementation. |
| `talos-models` crate (F2) | Quarantine + document as non-runtime. Ensure no CLI/TUI/runtime crate depends on it. Add a guard test proving `~/.talos/catalog.db` is never created. Do NOT remove the crate (avoids semver/stop-and-ask). |
| Branch / commits | Stay on `main`. Commit at F4 / F8 / F12 / F16 (and F0 kickoff). |
| Push | Push to `origin/main` after each phase commit (user instruction 2026-07-06: "请每个阶段进行一次提交和推送"). |
| Pacing | F0-F16 continuous; stop only on the plan's defined Stop-And-Ask conditions. |
| Model name in commits | `[model:glm-5.2]` |

Key codebase facts discovered during confirmation (affecting plan shape):

- F7 (MODEL-006 CLI model browser): **already Complete** — `--available-models-browser` shipped with viewport-windowed large-catalog rendering and all 10 acceptance criteria checked (I101).
- F9 (TUI-025 tool argument line-fit): **already Complete** (2026-07-04). Tests `tool_args_summary_uses_available_budget_before_truncating` and `approval_state_preserves_full_multibyte_arguments` exist.
- F10 (TUI-015 head-tail 3/3): **already Complete** — retained lines changed to 3/3 on 2026-07-04.
- F11 (TUI-019 tool output visual hierarchy): **already Complete** in I076/T105.
- F5/F6 (standard-provider connect): **already Complete** per MODEL-006 I101 update — standard catalog providers skip URL input; custom providers require it.
- F13/F14 (site i18n): `site/zh/` already contains all 7 pages, but WEB-003 acceptance (language switcher on every page, `validate_public_site.sh` coverage of `site/zh/`, `site/README.md` docs) is unchecked — real work remains.
- F1/F2 (catalog.db cleanup): genuinely remaining — `talos-models` crate with SQLite `store.rs` still present; `catalog.db`/`ModelCatalog` references remain in `crates/talos-models/`, `crates/talos-cli/src/main.rs`, READMEs, and historical docs.
- TUI-025 ID collision: two backlog files share `TUI-025` (`composer-multiline-wrap` and `tool-argument-line-fit-display`). Cosmetic only; does not block execution.
- MODEL-005 status drift: marked Planned with unchecked acceptance criteria, but its `/model` + `/connect` behavior is delivered via MODEL-006/I101. Status not yet synced in the owner doc.

## Checkpoint F0 - Kickoff (2026-07-06)

Completed items:
- F0: start inventory.

Current state and artifacts:
- Working tree clean on `main` (`7dea613`), 18 commits ahead of `origin/main`, not pushed.
- Required reads completed: `AGENTS.md`, `docs/sop/LONG-RUNNING-TASK.md`, `docs/sop/ITERATION-WORKFLOW.md`, `docs/sop/GIT-WORKFLOW.md`, `docs/sop/DOC-CHECK.md`, `docs/BOARD.md`, `docs/backlog/PRODUCT-BACKLOG.md`, and the active backlog items MC-002, MODEL-005/006, TUI-015/019/025, WEB-003/004.
- Confirmed overlap with already-complete work cataloged in the Consolidated Confirmation table above.

Commands/checks and actual results:
- `scripts/validate_project_governance.sh .` → "Governance validation passed: 0 warning(s)." (exit 0).
- `git diff --check` → CLEAN.
- `git status` → on `main`, nothing to commit, working tree clean.

Open risks or deviations:
- F5-F11 reference already-complete backlog items. Resolution per confirmation: verify + record evidence + close, do not re-implement.
- `talos-models` crate remains in the workspace. F2 will quarantine (not remove) per confirmation.
- User added per-phase push requirement after initial plan said "do not push." Pushing to `origin/main` after each phase commit.

Next task item:
- F1: Catalog residual audit — `rg "catalog\\.db|ModelCatalog|models.toml" crates docs README.md README.zh-CN.md`, classify each hit as active library capability, planned removal, or dead residual; record findings.

Recovery or resume instruction:
- Owner record: this file. Git state: `main` at the F0 kickoff commit (to be created next). Resume by running F1 audit with the `rg` command above and classifying each hit. Do not implement before the audit table is complete.

## Checkpoint F1 - Catalog Residual Audit (2026-07-06)

Audit command:
- `rg "catalog\.db|ModelCatalog|talos_models|models\.toml" crates docs README.md README.zh-CN.md`

Findings table (classification: Active / Dead residual / Historical / This requirement):

| # | Location | Reference | Classification | Notes |
|---|---|---|---|---|
| A1 | `Cargo.toml:28` | workspace member `crates/talos-models` | **Dead residual** | No other workspace crate lists `talos-models` as a dependency or dev-dependency |
| A2 | `crates/talos-models/src/lib.rs` | `pub use store::ModelCatalog; pub use import::import_models_dev_*` | **Dead residual** | lib.rs doc comment claims CLI/TUI constructs `ModelCatalog`, but no `use talos_models` / `talos_models::` exists anywhere in `crates/` outside the crate itself |
| A3 | `crates/talos-models/src/store.rs` (1038 lines) | SQLite-backed `ModelCatalog::open(&db_path)`, `open_memory`, `seed`, `upsert_*`, `find_model`, `search_*`; test on `catalog.db` file at `dir.path().join("catalog.db")` | **Dead residual** | Entire SQLite store. No production crate depends on it. Runtime path was superseded by 2026-07-05 maintainer decision |
| A4 | `crates/talos-models/src/import.rs` (462 lines) | `import_models_dev_api` / `import_models_dev_models` parsers with test fixtures | **Dead residual** | `talos-config/build.rs` has its OWN inline `parse_api_json` rewrite — does not use `talos-models::import`. The two parsers are independent |
| A5 | `crates/talos-models/src/error.rs` (29 lines) | `CatalogError` | **Dead residual** | Only consumed by `store.rs` and `import.rs` |
| A6 | `crates/talos-cli/src/main.rs:388` | `--import-models` help text: "This flag no longer writes to catalog.db." | **Dead residual (text)** | Informative notice. No runtime logic. F2 may refine wording to clarify it never wrote to any catalog in current builds |
| A7 | `crates/talos-config/src/model.rs:33,46,65` | `include_str!("models.toml")` for `builtin_models()` and `builtin_providers()` | **Active** | Packaged runtime catalog — the accepted path |
| A8 | `crates/talos-config/build.rs` | `BUILD_MODELS=1` refresh via `curl`; inline `parse_api_json`/`generate_toml`; writes `src/models.toml` | **Active** | Build-time refresh — must be preserved per MC-002 Out Of Scope |
| A9 | `crates/talos-cli/src/mode_runners.rs:269,356,2150` | `models.toml` provider default + built-in display name/endpoint metadata | **Active** | Runtime reads packaged `models.toml` metadata, not `catalog.db` |
| A10 | `crates/talos-cli/src/model_lifecycle.rs` and `models_browser.rs` | No hits for `catalog.db`/`ModelCatalog`/`talos_models`/`catalog_snapshot` | **Active (confirmed clean)** | These files use packaged `models.toml` and user config only — DB-free |
| A11 | `docs/BOARD.md:53`, `docs/backlog/PRODUCT-BACKLOG.md:22,46` | MC-002 story descriptions | **This requirement** | Stay as-is until F2 closes; updated at F4 |
| A12 | `docs/iterations/I085-model-catalog-modernization.md` | I085 historical execution log; `ModelCatalog`, `catalog.db` pervasive | **Historical** | Immutable iteration record. Do not rewrite. F2 may add a terminal status note pointing to MC-002 |
| A13 | `docs/tasks/2026-07-03-programmer-handoff-i085-model-catalog.md` | I085 handoff; line 180 "catalog.db when available and readable" in Stage 1 plan | **Historical** | Immutable task record describing superseded Stage 2 intent. Do not rewrite. |
| A14 | `docs/tasks/2026-07-05-open-requirement-implementation-audit.md:157,158,178` | Audit confirming `--import-models` no-op, no `catalog.db` runtime path | **Active (correct)** | Already reflects current accepted behavior. Leave unchanged |
| A15 | `docs/decisions/034-reasoning-thinking-boundary.md:81,329` | `models.toml` carries `capabilities.reasoning` | **Active** | References packaged catalog correctly |
| A16 | `docs/reference/I098-I101-AUTONOMY-PERMISSION-RUNTIME-CLOSEOUT-2026-07-06.md:80` | "no `catalog.db` resurrection" prohibition statement | **Active** | Records the right prohibition — keep |
| A17 | `README.md:396-398`, `README.zh-CN.md:379` | "Talos does not create a runtime `catalog.db` for model metadata" + `/connect` uses packaged `models.toml` | **Active (correct)** | README already states the accepted behavior. F3 verifies these entries are consistent with `/model` and `/connect` docs |
| A18 | `docs/backlog/active/MC-002-*.md` | self-references in Scope and Required Reads | **This requirement** | Owner doc; updated by F2 |
| A19 | `docs/backlog/active/MC-001-*.md` | MC-001 epic description mentions `talos-models`, `catalog.db` runtime integration | **Historical** | Epic owner doc. F2 may add a short "2026-07-06 status: runtime catalog.db path superseded; see MC-002" note without rewriting history |
| A20 | `docs/backlog/PRODUCT-BACKLOG.md:44` | MC-001 epic row in the summary table | **Owner-doc state** | Updated by F2/F4 to reflect MC-002 closure |

Cross-crate dependency check (decisive):
- `rg "talos-models|talos_models" crates/*/Cargo.toml` → only `crates/talos-models/Cargo.toml` itself lists the name. No other crate depends on it.
- `rg "use talos_models|extern crate talos_models|talos_models::" crates --type rust` → 0 hits outside `crates/talos-models/`.
- `crates/talos-models/Cargo.toml` depends on `talos-core` + `rusqlite` (bundled). No other crate pulls it in.
- `crates/talos-config/build.rs` has its OWN inline `parse_api_json`/`generate_toml` — does not import `talos-models::import`.

Classification summary:
- **Active library capability** (preserve, do not touch): A7, A8, A9, A10, A14, A15, A16, A17
- **Dead residual** requiring cleanup in F2: A1 (workspace member), A2/A3/A4/A5 (crate source — quarantine, not delete, per confirmation), A6 (help text wording)
- **Historical docs** (immutable records — do not rewrite, optional short status note only): A12, A13, A19
- **This requirement** (owner docs, updated at F2/F4 closeout): A11, A18, A20

No runtime dependency on `catalog.db` was found. F2 can proceed with the quarantine + guard-test approach without risk to the running CLI/TUI.

Commands/checks and actual results:
- `rg "catalog\.db|ModelCatalog|talos_models|models\.toml" crates --type rust --type toml` → hits classified above.
- `rg "use talos_models|talos_models::" crates --type rust -g '!crates/talos-models/**'` → 0 hits.
- `rg "talos-models" crates/*/Cargo.toml` → only self.
- `cargo test -p talos-models --no-run` → Compiles; only depends on `talos-core` + `rusqlite`.

Open risks or deviations:
- None blocking. F2 quarantine approach confirmed safe by the dependency check.

Next task item:
- F2: quarantine `talos-models` as non-runtime, add `# Non-runtime` status note to its lib.rs/header, ensure `Cargo.toml` member annotation reflects quarantine, add a guard test proving `~/.talos/catalog.db` is never created at CLI startup, clean stale wording in `crates/talos-cli/src/main.rs` help text, and optionally add terminal status notes on MC-001 epic owner doc and I085 record. Do NOT delete the crate.

Recovery or resume instruction:
- Owner record: this file. Git state: `main` at the F1 audit commit (to be created next). Resume by applying F2 cleanup per the table above; quarantine (do not delete) `talos-models`; add the guard test under `crates/talos-cli/tests/` and run `cargo check --workspace`.

## Checkpoint F2 - Catalog Residual Cleanup (2026-07-06)

Completed items:
- F2: catalog residual cleanup (quarantine, not delete).

Current state and artifacts:
- `crates/talos-models/src/lib.rs` — Status header rewritten as "Quarantined (non-runtime, 2026-07-06)" with explicit invariant and MC-002/MC-001 references.
- `crates/talos-models/Cargo.toml` — description now reads "Quarantined non-runtime SQLite catalog store (historical/foundation only)…" plus comment block.
- `Cargo.toml` — workspace member list carries a 4-line quarantine comment above the `talos-models` entry.
- `crates/talos-cli/src/main.rs:385-396` — `--import-models` no-op help text clarified; it now states Talos does not create/read `~/.talos/catalog.db` at runtime.
- `crates/talos-cli/tests/no_catalog_db_guard.rs` — NEW regression guard test file (5 tests).
- `docs/backlog/active/MC-002-remove-runtime-catalog-db-residuals.md` — Status flipped to Complete; all 6 applicable acceptance criteria checked off and justified.

Commands/checks and actual results:
- `cargo check --workspace` → exit 0 (compiled `talos-models` + `talos-cli`).
- `cargo build --tests -p talos-cli` → exit 0; guard test compiles.
- `cargo test -p talos-cli --test no_catalog_db_guard` → **5 passed; 0 failed**: `import_models_does_not_create_catalog_db`, `available_models_does_not_create_catalog_db`, `available_models_filter_does_not_create_catalog_db`, `available_models_all_does_not_create_catalog_db`, `config_list_does_not_create_catalog_db`.
- `cargo test -p talos-cli model` → exit 0 (no regression in catalog tests).
- `cargo test -p talos-cli connect` → exit 0 (no regression in connect tests).
- `cargo test -p talos-models` → 37 passed (quarantined crate still compiles and tests).
- `rg "catalog.db|ModelCatalog|talos_models" crates docs` → only allowed references (quarantined crate internals, owner docs, immutable iteration logs).

Open risks or deviations:
- The `--available-models-browser` path cannot be exercised in CI (interactive TTY required); `--available-models` is the bounded sibling used by the guard test. MODEL-006 viewport-windowed rendering acceptance is closed workspace-wide already.
- MC-001 epic Status field cannot move below Paused because of the open MC107 residual (manual TUI `/connect` walkthrough). The 2026-07-05 maintainer decision in the MC-001 owner doc is unchanged.

Residual owner:
- MC-001/MC107 real terminal `/connect` walkthrough remains under MC-001 owner doc — not in scope for this frontline plan.

Next task item:
- F3: `/model` and `/connect` docs sync. Verify `README.md` / `README.zh-CN.md` / active docs describe `/model` as credential-present models only and `/connect` as the provider setup owner; record any behavior gap instead of documenting false behavior.

Recovery or resume instruction:
- Owner record: this file. Git state: `main` at the F2 cleanup commit (to be created next). Resume by running F3's `rg "/model|/connect" README.md README.zh-CN.md docs -n` to inventory documentation references, then close any text drift.
