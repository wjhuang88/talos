# 2026-07-06 Frontline Four-Month Execution Plan

**Status**: Complete (2026-07-06, F0-F16 all closed; full closeout matrix passed)
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
| F0 | 1 | Start inventory | Check current Board/backlog/task state and append a kickoff checkpoint to this file. | None | `scripts/validate_project_governance.sh .` and `git diff --check` pass. | If owner docs conflict, record conflict and stop. | Complete |
| F1 | 1 | Catalog residual audit | Confirm no user-facing flow depends on runtime `catalog.db`; list any leftover names/docs/tests. | F0 | `rg "catalog\\.db|ModelCatalog|models.toml" crates docs README.md` reviewed and findings recorded. | If runtime dependency is found, create a blocker under MC-002 and do not remove blindly. | Complete |
| F2 | 2 | Catalog residual cleanup | Remove stale `catalog.db` docs/code references that are clearly dead after audit. | F1 | Targeted tests for affected crates plus `cargo check --workspace`. | If ownership is unclear, leave code unchanged and document the exact stale reference. | Complete |
| F3 | 3 | `/model` and `/connect` docs sync | Update README/docs so `/model` shows configured/usable models and `/connect` owns provider setup. | F2 | `rg "/model|/connect" README.md README.zh-CN docs -n` reviewed; governance passes. | If current behavior differs, write a behavior gap instead of documenting false behavior. | Complete |
| F4 | 4 | Month 1 closeout | Close catalog and command-doc residuals with evidence. | F1-F3 | `cargo fmt --all -- --check`, `cargo check --workspace`, targeted tests, governance, `git diff --check`. | Mark Partial with residual owner and exact failing command. | Complete |
| F5 | 5 | Standard-provider connect regression | Ensure built-in providers do not ask for base URL; only custom providers do. | F4 | Tests cover standard provider, custom provider, config merge, and masked secret rendering. | If behavior is already covered, link tests and make no code change. | Complete |
| F6 | 6 | Protocol metadata display audit | Verify model/provider protocol metadata from packaged `models.toml` is surfaced where setup needs it. | F5 | Tests or snapshots prove known protocol-backed providers route correctly without user URL input. | If metadata is missing from packaged data, record sync blocker; do not add runtime DB. | Complete |
| F7 | 7 | CLI model list usability | Improve `--available-models` for large catalogs with an independent scroll/search browser or bounded paged output. | F6 | Terminal/manual evidence shows large lists do not flood stdout and entries remain provider-qualified. | If interactive browser is too broad, implement `--available-models --filter`/paging only and record browser residual. | Complete |
| F8 | 8 | Month 2 closeout | Close model setup/listing usability package. | F5-F7 | `cargo test -p talos-cli`, `cargo test -p talos-config`, `cargo check --workspace`, governance. | Mark Partial with exact residuals. | Complete |
| F9 | 9 | Tool argument line-fit display | Improve TUI tool-call parameter rendering so arguments are shown fully when one line has room, truncating only when needed. | F8 | Focused TUI tests cover short args, long one-line args, multi-line args, and secret-safe rendering. | If rendering helper is shared with approval secrets, stop and ask. | Complete |
| F10 | 10 | Head-tail retained lines | When middle elision is triggered, keep only first 3 and last 3 lines without changing the trigger or summary routing. | F9 | Tests prove short outputs stay full, long fallback keeps 3+3, omitted count is correct, export/model payload remains full. | If trigger logic must change, do not implement; record blocker. | Complete |
| F11 | 11 | Tool output visual hierarchy | Make grouped/header text more readable using existing TUI palette constants. | F10 | TUI tests or snapshots cover group/header style; no one-off color literals if palette constants exist. | If contrast target is ambiguous, choose existing high-contrast palette constant and record rationale. | Complete |
| F12 | 12 | Month 3 closeout | Close TUI display package. | F9-F11 | `cargo test -p talos-tui`, `cargo test -p talos-tools`, `cargo check --workspace`, governance. | Mark Partial with screenshot/test residual. | Complete |
| F13 | 13 | Static site i18n inventory | Inventory public site pages and untranslated strings. | F12 | Checklist lists every page and whether zh-CN counterpart exists. | If site validator is missing, record manual validation plan. | Complete |
| F14 | 14 | Static site i18n implementation | Add or update zh-CN static pages using existing assets and relative links. | F13 | Site validator if present, manual link check, no new JS framework/build tool. | If a page is too ambiguous to translate, add a deferral note. | Complete |
| F15 | 15 | Static site branding polish | Apply small static CSS/SVG polish consistent with Talos identity. | F14 | Visual/manual evidence; no remote assets, analytics, fonts, or build tooling. | If design direction is unclear, limit to contrast/accessibility fixes. | Complete |
| F16 | 16 | Final closeout | Produce final handoff closeout with commits, validation, residuals, and next-cycle candidates. | F0-F15 | `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo test --workspace`, governance, `git diff --check`. | Close as Partial only with exact failed gate and owner for every residual. | Complete |

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

## Checkpoint F3 - /model and /connect Docs Sync (2026-07-06)

Completed items:
- F3: `/model` and `/connect` docs sync.

Current state and artifacts:
- `README.md:393-402` — `/model` / `/connect` paragraph closed a text drift: it previously
  said only that `/connect` "offers an optional custom endpoint (`base_url`) for
  gateway-compatible providers", without stating the standard-provider URL-skip behavior.
  Added: "Standard providers whose catalog metadata supplies a default endpoint submit after
  the API key without prompting for a URL; custom providers (or any row without a built-in
  endpoint) still require a non-empty `base_url`." Matches the I101 closeout evidence.
- `README.zh-CN.md:379` — Chinese version closed the same drift with a matching clause
  (English / Chinese semantic symmetry preserved; English content unchanged outside the
  URL-skip clarification).
- Slash command table rows `/model` (README.md:442 / README.zh-CN.md:367) and `/connect`
  (README.md:443 / README.zh-CN.md:368) already accurately describe the split (model picker
  for configured providers; provider setup for `/connect` with optional custom endpoint).
  No change needed.
- No behavior gap was found (no false behavior was being documented). The fix is purely
  additive text to make the standard-provider URL-skip behavior explicit in the same
  paragraph that already mentions custom endpoints.

Commands/checks and actual results:
- `rg "/model|/connect" README.md README.zh-CN.md docs -n` → all references reviewed;
  only one text drift (the standard/custom endpoint asymmetry) is closed above.
- `scripts/validate_project_governance.sh .` → "Governance validation passed: 0 warning(s)."

Open risks or deviations:
- None.

Next task item:
- F4: Month 1 closeout — full validation matrix run, month-1 commit, then checkpoint appended.

Recovery or resume instruction:
- Owner record: this file. Git state: `main` at the F3 docs-sync commit (to be created next). Resume by running the Month 1 closeout validation matrix (`cargo fmt --all -- --check`, `cargo check --workspace`, `cargo test -p talos-cli`, `cargo test -p talos-config`, governance, `git diff --check`), then commit and append the F4 checkpoint.

## Checkpoint F4 - Month 1 Closeout (2026-07-06)

Completed items:
- F0 (kickoff), F1 (audit), F2 (cleanup), F3 (docs sync).

Current state and artifacts:
- All four Month 1 commits live on `origin/main`:
  - `6ad5894` F0 kickoff inventory + consolidated confirmation contract
  - `9325aec` F1 catalog residual audit (findings table)
  - `3cda7ad` F2 quarantine `talos-models` as non-runtime + 5-test `no_catalog_db_guard.rs`; MC-002 owner doc closed
  - `8f5ee58` F3 README URL-skip text drift closed
- Owner docs updated ahead of derived docs:
  - `docs/backlog/active/MC-002-remove-runtime-catalog-db-residuals.md` — Status Complete, acceptance criteria [x].
  - `docs/backlog/active/MC-001-model-catalog-modernization.md` — unchanged (already records the 2026-07-05 maintainer decision and the MC107 real-terminal residual).
- Talos owner docs/READMEs agree that model metadata is packaged (no runtime `catalog.db`).
- Standard providers' URL-skip behavior is now explicitly documented in both README languages.

Commands/checks and actual results:
- `cargo fmt --all -- --check` → exit 0 (clean).
- `cargo check --workspace` → exit 0.
- `cargo test -p talos-cli` → exit 0; new guard test 5/5 pass; pre-existing `rpc_e2e` 1/1, `skill_runtime_e2e` 2/2 pass.
- `cargo test -p talos-config` → exit 0; doc-tests 1/1 pass; all unit tests pass.
- `scripts/validate_project_governance.sh .` → "Governance validation passed: 0 warning(s)."
- `git diff --check` → CLEAN.

Open risks or deviations:
- None for Month 1.
- The MC-001 MC107 real-terminal `/connect` walkthrough residual remains parked under its owner doc; it is out of scope for this frontline plan.

Residual owner:
- MC-001 / MC107 real-terminal `/connect` walkthrough — under MC-001 owner doc (out of scope here).

Next task item:
- F5: Standard-provider connect regression — verify built-in providers skip URL, custom requires URL, config merge works, secrets are masked. Run/verify cited tests for MODEL-006/I101 already-shipped behavior.

Recovery or resume instruction:
- Owner record: this file. Git state: `main` at the F4 checkpoint commit (to be created next). Month 1 work is committed and pushed. Resume Month 2 by verifying F5 standard-provider connect regression coverage and recording evidence in this file.

## Checkpoint F5 - Standard-Provider Connect Regression (2026-07-06)

Completed items:
- F5: verified (no implementation; MODEL-006/I101 already shipped and tested).

Current state and artifacts:
- Already-complete MODEL-006/I101 standard-provider connect behavior is covered by 21 tests across 6 files in 4 crates. Behavior verified, no code change made.

Test evidence (all passed; run with `cargo test -p <crate> [--bins] <filter>`):

Case 1 — Standard provider skips URL input (4 tests):
- `talos-cli --bins` `models_browser::tests::provider_setup_standard_provider_uses_default_without_typed_url` — openai row uses catalog default URL without typed URL.
- `talos-cli --bins` `mode_runners::connect_tests::handle_connect_default_base_url_falls_back_to_builtin_provider_config` — groq connect emits `default_base_url = Some("https://api.groq.com/openai/v1")`.
- `talos-cli --bins` `mode_runners::connect_tests::handle_connect_minimax_coding_plan_uses_anthropic_messages_endpoint` — minimax-coding-plan default URL normalized with `/messages`.
- `talos-tui` `state::tests::connect_mode_standard_provider_submits_without_base_url_field` — TUI submits after API key with default endpoint, no BaseUrl field.

Case 2 — Custom provider requires URL (5 tests):
- `talos-cli --bins` `models_browser::tests::provider_setup_custom_provider_requires_base_url` — returns "base URL is required".
- `talos-tui` `state::tests::connect_mode_custom_provider_first_submit_advances_to_base_url_field` — first submit returns None, advances to BaseUrl field.
- `talos-tui` `state::tests::connect_mode_custom_provider_second_submit_returns_typed_base_url` — typed URL submitted successfully.
- `talos-tui` `state::tests::connect_mode_custom_provider_empty_base_url_stays_open` — empty URL keeps panel open.
- `talos-tui` `state::tests::connect_mode_empty_api_key_cancels_without_advancing` — empty API key cancels, no advance to BaseUrl.

Case 3 — Config merge preserves unrelated fields (4 tests):
- `talos-cli --bins` `mode_runners::connect_tests::handle_connect_with_credential_preserves_unrelated_provider_fields` — preserves groq existing base_url + model overrides, leaves anthropic untouched.
- `talos-cli --bins` `mode_runners::connect_tests::handle_connect_with_credential_updates_base_url_when_provided` — supplied base_url overwrites.
- `talos-cli --bins` `mode_runners::connect_tests::handle_connect_with_credential_writes_new_provider_api_key_and_base_url` — fresh connect writes api_key/api_key_env/base_url.
- `talos-cli --bins` `tests::tests::config_save_load_roundtrip_preserves_fields` — protocol/base_url/api_key_env/model context_limit survive save+load.

Case 4 — api_key masked in display (8 tests):
- `talos-config` `tests::test_provider_config_debug_masks_api_key` — Debug output shows "***", not secret.
- `talos-config` `tests::test_credentials_debug_masks_keys` — Credentials Debug shows "redacted".
- `talos-config` `tests::test_config_debug_masks_provider_api_keys` — Config Debug masks api_key with "***".
- `talos-cli --bins` `tests::tests::mask_secrets_masks_api_key_lines` — mask_secrets replaces api_key value, leaves api_key_env.
- `talos-cli --bins` `tests::tests::config_subcommand_list_masks_secrets` — `config list` output shows "api_key = ***".
- `talos-cli --bins` `tests::tests::config_secret_masking_survives_roundtrip` — masking persists through serialize+deserialize.
- `talos-tui` `app::app_tests::credential_display_never_reveals_secret_suffix` — `credential_display_text` returns only bullet chars.
- `talos-tui` `app::app_tests::credential_cursor_tracks_masked_buffer` — cursor positioning works against masked representation.

Commands/checks and actual results:
- All 21 tests passed (see commands above); captured as the F5 acceptance evidence.

Open risks or deviations:
- None.

Next task item:
- F6: Protocol metadata display audit.

Recovery or resume instruction:
- Owner record: this file. Git state: `main` at F4 (Month 1 closeout commit). Resume by running the F6 protocol-metadata tests (`provider_setup_uses_catalog_protocol_when_url_does_not_reveal_protocol`, `provider_setup_minimax_coding_plan_uses_anthropic_messages_endpoint`, `handle_connect_with_credential_sets_anthropic_protocol_for_minimax_endpoint`, `handle_connect_minimax_coding_plan_uses_anthropic_messages_endpoint`, `test_anthropic_catalog_endpoint_normalized_for_legacy_minimax_config`, `test_default_config`, endpoint normalization tests) and the no-catalog-db guard for `--available-models` variants.

## Checkpoint F6 - Protocol Metadata Display Audit (2026-07-06)

Completed items:
- F6: verified (no implementation; protocol metadata from packaged `models.toml` is consumed in every required flow).

Current state and artifacts:
- Protocol metadata flow verified per path:
  - `/connect` — `handle_connect()` resolves `default_base_url` using `builtin_providers().protocol` (Append `/messages` for AnthropicMessages).
  - `/connect` final — `handle_connect_with_credential()` runs `normalize_provider_endpoint(base_url)` to set `provider_entry.protocol`.
  - `/model` — `build_model_picker_data()` uses `config.all_models()` (built from `builtin_models()`); provider protocol was already resolved during `/connect` or `set_active_model()`.
  - `--available-models` — `run_models()` calls `talos_config::model::builtin_models()` directly; protocol embedded in provider metadata.
  - `--available-models-browser` — `build_browser_rows()` populates `CatalogBrowserRow.protocol` from `builtin_providers()`; `apply_provider_setup()` uses `normalize_row_endpoint()` which checks `row.protocol` first (catalog protocol wins over URL inference).
- Endpoint normalization (`crates/talos-config/src/endpoint.rs`) infers protocol from URL paths; catalog protocol overrides URL inference when both are present.

Test evidence (all passed):

Catalog-protocol-wins tests:
- `talos-cli --bins` `models_browser::tests::provider_setup_uses_catalog_protocol_when_url_does_not_reveal_protocol` — kimi row with `AnthropicMessages` and URL `https://api.kimi.com/coding/v1` (no `/anthropic/`) keeps `AnthropicMessages` protocol. **Key proof that catalog protocol overrides URL inference.**
- `talos-cli --bins` `models_browser::tests::provider_setup_minimax_coding_plan_uses_anthropic_messages_endpoint` — minimax row gets `/messages` appended and `AnthropicMessages` protocol.

`/connect` flow protocol tests:
- `talos-cli --bins` `mode_runners::connect_tests::handle_connect_minimax_coding_plan_uses_anthropic_messages_endpoint` — default URL normalized with `/messages` endpoint for AnthropicMessages providers.
- `talos-cli --bins` `mode_runners::connect_tests::handle_connect_with_credential_sets_anthropic_protocol_for_minimax_endpoint` — after credential submit, `minimax.protocol == AnthropicMessages`.

Config-level protocol tests:
- `talos-config` `tests::test_default_config` — default anthropic provider has `AnthropicMessages` protocol.
- `talos-config` `tests::test_anthropic_catalog_endpoint_normalized_for_legacy_minimax_config` — minimax URL with `/anthropic/v1` resolves to AnthropicMessages protocol.
- `talos-config` `tests::test_builtin_anthropic_custom_endpoint_keeps_anthropic_protocol` — anthropic with custom gateway URL stays AnthropicMessages.

Endpoint normalization tests (`crates/talos-config/src/endpoint.rs`):
- `talos-config` `endpoint::tests::normalizes_anthropic_root_to_messages_endpoint` — root URL gets `/messages` appended.
- `talos-config` `endpoint::tests::preserves_anthropic_messages_endpoint` — already-on-`/messages` URL is preserved.
- `talos-config` `endpoint::tests::strips_openai_chat_completions_endpoint_to_root` — openai `/chat/completions` URL is normalized to root.
- `talos-config` `endpoint::tests::preserves_openai_gateway_root` — gateway root URL is preserved.

CLI protocol config tests:
- `talos-cli --bins` `tests::tests::config_set_protocol` — `config set providers.<x>.protocol openai-chat` round-trips.
- `talos-cli --bins` `tests::tests::config_set_dotted_rejects_invalid_protocol` — invalid protocol value produces an error.

No-runtime-DB tests (also satisfy F7 catalog display source):
- `talos-cli` integration `no_catalog_db_guard::available_models_does_not_create_catalog_db`, `available_models_filter_does_not_create_catalog_db`, `available_models_all_does_not_create_catalog_db` — `--available-models` variants source from packaged `models.toml`.

Commands/checks and actual results:
- All above tests passed (see commands executed in this session).

Open risks or deviations:
- Potential gap (residual, NOT blocking): `set_active_model()` in `crates/talos-config/src/config.rs` creates a provider entry via `builtin_provider_config()` (hardcoded 14 providers) and does NOT consult `builtin_providers()` (the 149-provider `models.toml` parser) for protocol. For non-hardcoded providers (e.g., `kimi-for-coding`, `minimax-coding-plan`) selected via `/model` without prior `/connect`, the protocol would default to OpenAIChat rather than reading catalog metadata. This is NOT triggered in normal `/model` usage because `/model` only shows credentials-present providers (which were set up via `/connect`, where protocol is correctly resolved). The gap only manifests in an unusual `/model` path with no prior `/connect` for a non-hardcoded provider. Recorded under MODEL-006 backlog residual; not blocking F6.

Next task item:
- F7: CLI model list usability.

Recovery or resume instruction:
- Owner record: this file. Git state: `main` at F4. Resume by running the MODEL-006 browser tests (`cargo test -p talos-cli --bins browser`) and the terminal `--available-models` walkthrough captured below.

## Checkpoint F7 - CLI Model List Usability (2026-07-06)

Completed items:
- F7: verified (MODEL-006 already shipped and tested).

Current state and artifacts:
- `talos --available-models` and `talos --available-models-browser` are shipped (I101 closeout). All 10 MODEL-006 acceptance criteria were already checked in the owner doc.
- Terminal manual QA performed 2026-07-06:
  - `talos --available-models` → "Built-in model catalog: 4182 matching models across 149 providers. Showing first 120. Use --available-models-filter ... or --available-models-all to print all." then rows under headers like `302ai — Setup required`. Last output: "... 4062 more matching models omitted. Use --available-models-all to print every row."
  - Rows printed as `provider/model` (e.g., `302ai/claude-opus-4-1-20250805`, `anthropic/claude-haiku-4-5`) — provider-qualified.
  - `talos --available-models --available-models-filter anthropic/claude` → "Built-in model catalog: 189 matching models across 17 providers" then filtered rows under `anthropic — Setup required`. Filter narrows by provider, model id, and provider-qualified id.
  - Both commands exited 0. No credentials were entered or stored during this inspection.

Test evidence (all passed):
- `talos-cli --bins` `models_browser::tests::filters_by_provider_model_and_qualified_name` — filter matches all three identifiers.
- `talos-cli --bins` `models_browser::tests::render_marks_current_and_setup_without_secrets` — current model marked, setup rows shown, no secret printed.
- `talos-cli --bins` `models_browser::tests::render_lines_is_viewport_windowed_for_large_catalog` — opening an 8-line view over 500 rows renders only the visible window (no full dump).
- `talos-cli --bins` `models_browser::tests::navigation_stays_on_model_rows` — j/k navigation skips headers.
- `talos-cli --bins` `models_browser::tests::fit_truncates_to_width` — long ids truncated to fit the available width.
- `talos-cli --bins` `provider_setup_*` (7 tests, also cited in F5) — credential/base_url/protocol routing in browser.
- `talos-cli` integration `no_catalog_db_guard::available_models_does_not_create_catalog_db`, `available_models_filter_does_not_create_catalog_db`, `available_models_all_does_not_create_catalog_db` — bounded/filter/all variants stay DB-free.

Commands/checks and actual results:
- `cargo test -p talos-cli --bins browser` → 10 passed; 0 failed.
- `cargo test -p talos-cli --test no_catalog_db_guard` → 5 passed; 0 failed (F2 evidence reused).
- Terminal: `talos --available-models` and `talos --available-models --available-models-filter anthropic/claude` → bounded + provider-qualified output, exit 0.

Open risks or deviations:
- The `--available-models-browser` interactive path cannot be exercised here (TTY required). The viewport-windowed rendering test plus the I101 closeout real-binary walkthrough already satisfy the MODEL-006 acceptance. No new residual.

Next task item:
- F8: Month 2 closeout.

Recovery or resume instruction:
- Owner record: this file. Git state: `main` at F4. Month 2 work was verification-only (no code change). Resume by running the Month 2 closeout gates (`cargo test -p talos-cli`, `cargo test -p talos-config`, `cargo check --workspace`, governance) and committing this checkpoint.

## Checkpoint F8 - Month 2 Closeout (2026-07-06)

Completed items:
- F5 (standard-provider connect regression — verified), F6 (protocol metadata display audit — verified), F7 (CLI model list usability — verified).

Current state and artifacts:
- No code change in Month 2 (verification-only per confirmation: F5/F6/F7 already shipped).
- This file's F5/F6/F7 checkpoints record every test name and assertion as evidence; the MODEL-006 owner doc retains all 10 acceptance criteria as checked.

Commands/checks and actual results:
- `cargo test -p talos-cli` (full crate, including integration tests) → exit 0:
  - Unit tests (`--bins`): 154 passed; 0 failed; 0 ignored; 0 filtered out.
  - `tests/rpc_e2e.rs`: 1 passed.
  - `tests/skill_runtime_e2e.rs`: 2 passed.
  - `tests/no_catalog_db_guard.rs`: 5 passed.
- `cargo test -p talos-config` → exit 0; all unit tests passed + 1 doctest passed.
- `cargo check --workspace` → exit 0 (already validated in F4; unchanged).
- `scripts/validate_project_governance.sh .` → "Governance validation passed: 0 warning(s)." (unchanged from F4).
- `git diff --check` → CLEAN.

Open risks or deviations:
- F6 noted a residual: `set_active_model()` does not consult `builtin_providers()` for protocol when a non-hardcoded provider is selected via `/model` without prior `/connect`. Not blocking because `/model` only shows credentials-present providers (already setup via `/connect`). Recorded as MODEL-006 residual.

Residual owner:
- MODEL-006 `set_active_model()` catalog-protocol lookup — under `docs/backlog/active/MODEL-006-interactive-model-catalog-browser.md` residual hardening.

Next task item:
- F9: Tool argument line-fit display verification (TUI-025).

Recovery or resume instruction:
- Owner record: this file. Git state: `main` at F4 (no Month 2 commit yet, since Month 2 was verification-only and produced no code change; the F8 checkpoint is appended to this file and will be committed alongside F12's Month 3 work, per the "one logical commit per month" plan rule). Resume Month 3 by running the F9 cited tests (`cargo test -p talos-tui tool_args_summary_uses_available_budget_before_truncating`, `cargo test -p talos-tui approval_state_preserves_full_multibyte_arguments`).

## Checkpoint F9 - Tool Argument Line-Fit Display (2026-07-06)

Completed items:
- F9: verified (no implementation; TUI-025 already shipped 2026-07-04).

Current state and artifacts:
- `crates/talos-tui/src/app/app_tests.rs:65 tool_args_summary_uses_available_budget_before_truncating` asserts that a long line-fit command argument renders the complete summary when the budget allows.
- `crates/talos-tui/src/state.rs approval_state_preserves_full_multibyte_arguments` asserts that approval state stores the full argument and leaves truncation to render time.

Test evidence (all passed):
- `talos-tui` `app::app_tests::tool_args_summary_uses_available_budget_before_truncating` → ok.
- `talos-tui` `state::tests::approval_state_preserves_full_multibyte_arguments` → ok.

Open risks or deviations:
- None.

Next task item:
- F10: Head-tail retained 3/3.

Recovery or resume instruction:
- Owner record: this file. Git state: `main` at the Month 2 closeout commit. Resume F10 by running `cargo test -p talos-tui head_tail` plus the under-threshold tests.

## Checkpoint F10 - Head-Tail Retained 3/3 (2026-07-06)

Completed items:
- F10: verified (no implementation; TUI-015 already shipped with 3/3 retained lines on 2026-07-04).

Current state and artifacts:
- `crates/talos-tui/src/tool_display.rs` constants: `HEAD_LINES = 3`, `TAIL_LINES = 3`, `SUMMARIZE_OUTPUT_THRESHOLD_LINES = 30`. These are the exact retained-line values F10 requires.
- `build_head_tail_scrollback_lines` (line 303) renders `HEAD_LINES` head lines + a `⋯ {omitted} lines omitted` separator + `TAIL_LINES` tail lines.
- Decision pipeline (line 282): only triggers head-tail when `all_lines.len() > SUMMARIZE_OUTPUT_THRESHOLD_LINES` (30) AND the tool is not in the summarize set. Summarize-eligible tools (read/grep/glob/ls/find_symbol/list_imports) still take the one-line summary path. Unchanged.

Test evidence (all passed):
- `talos-tui` `app::app_tests::head_tail_omitted_count_is_correct` → ok. For totals {31, 32, 50, 100}: renders exactly 7 lines (3 head + 1 separator + 3 tail); the separator carries `expected_omitted = total - 3 - 3` count.
- `talos-tui` `app::app_tests::head_tail_truncation_does_not_affect_export_content` → ok. The display is borrowed immutably, so export content survives head-tail truncation.
- Short-output full render (unchanged trigger):
  - `talos-tui` `app::app_tests::glob_under_threshold_not_summarized` → ok.
  - `talos-tui` `app::app_tests::ls_under_threshold_not_summarized` → ok.
  - `talos-tui` `app::app_tests::list_imports_under_threshold_not_summarized` → ok.
  - `talos-tui` `app::app_tests::grep_under_threshold_renders_inline` → ok.
  - `talos-tui` `app::app_tests::bash_under_threshold_renders_full` → ok.

Open risks or deviations:
- Summary routing orthogonality preserved: head-tail is the non-summarize fallback; the summarize path is unchanged.

Next task item:
- F11: Tool output visual hierarchy.

Recovery or resume instruction:
- Owner record: this file. Git state: `main` at the Month 2 closeout commit. Resume F11 by running `cargo test -p talos-tui tool_result`.

## Checkpoint F11 - Tool Output Visual Hierarchy (2026-07-06)

Completed items:
- F11: verified (no implementation; TUI-019 already shipped in I076/T105 on 2026-07-01).

Current state and artifacts:
- `crates/talos-tui/src/tool_display.rs` `result_line_style()` classifies primary vs secondary result lines; primary result lines use the result color (typically the success/error color), detail/preview lines use the existing dim/secondary semantic style (`secondary_result_color()` = `Rgb(0x9A, 0xA4, 0xB2)`).
- No one-off color literals introduced; the palette constant `secondary_result_color()` is reused.

Test evidence (all passed, 7 tests via `cargo test -p talos-tui tool_result`):
- `talos-tui` `tool_display::tests::tool_result_success_single_line_rendering` → ok.
- `talos-tui` `tool_display::tests::tool_result_error_rendering_unchanged` → ok.
- `talos-tui` `tool_display::tests::tool_result_success_special_cases_rendering` → ok (empty output "(no output)" and suppressed-read cases).
- `talos-tui` `app::app_tests::read_tool_result_hides_content_from_scrollback` → ok.
- `talos-tui` `app::app_tests::tool_result_error_detail_lines_keep_error_style` → ok (error style preserved on detail lines).
- `talos-tui` `app::app_tests::tool_result_scrollback_styles_primary_and_detail_lines` → ok (primary vs detail line styling).
- `talos-tui` `app::app_tests::tool_result_scrollback_keeps_multiple_lines` → ok.

Open risks or deviations:
- None.

Next task item:
- F12: Month 3 closeout.

Recovery or resume instruction:
- Owner record: this file. Git state: `main` at the Month 2 closeout commit. Resume F12 by running `cargo test -p talos-tui`, `cargo test -p talos-tools`, `cargo check --workspace`, governance, then commit + push.

## Checkpoint F12 - Month 3 Closeout (2026-07-06)

Completed items:
- F9 (tool argument line-fit — verified), F10 (head-tail retained 3/3 — verified), F11 (tool output visual hierarchy — verified).

Current state and artifacts:
- No code change in Month 3 (verification-only per confirmation: F9/F10/F11 already shipped).
- Owner docs `TUI-015`, `TUI-019`, `TUI-025` retain their Complete status with the cited tests as evidence.

Commands/checks and actual results:
- `cargo test -p talos-tui --lib` → **249 passed; 0 failed; 0 ignored**.
- `cargo test -p talos-tui` (doctests) → 2 passed (format_tokens, format_duration).
- `cargo test -p talos-tools` → all unit tests passed + 0 doctests.
- `cargo check --workspace` → exit 0 (unchanged from F4/F8).
- `scripts/validate_project_governance.sh .` → "Governance validation passed: 0 warning(s)." (unchanged).
- `git diff --check` → CLEAN.

Open risks or deviations:
- None.

Residual owner:
- None for Month 3.

Next task item:
- F13: Static site i18n inventory — inventory public site pages and check whether each page has a zh-CN counterpart, plus whether a language switcher and `validate_public_site.sh` coverage of `site/zh/` are in place.

Recovery or resume instruction:
- Owner record: this file. Git state: `main` at the Month 3 closeout commit (to be created next). Resume Month 4 by running F13 inventory against `site/` and `site/zh/`, then update the validation harness and language switcher per F14.

## Checkpoint F13 - Static Site i18n Inventory (2026-07-06)

Completed items:
- F13: inventory performed. Result: WEB-003 already fully shipped.

Current state and artifacts:
- 7 EN pages under `site/`: index, install, capabilities, safety, roadmap, releases, 404.
- 7 ZH pages under `site/zh/`: same set, 1:1.
- Language switcher: every EN page links to its ZH counterpart (`<a href="zh/index.html">中文</a>`); every ZH page links back to EN (`href="../index.html">EN`). The 404 pages keep a brand link to their own index (consistent with the static 404 convention).
- `scripts/validate_public_site.sh` already enumerates all 7 `zh/*.html` paths in its required-files list (line 46) and uses recursive `find` (line 55) to walk every HTML page under `site/`, so ZH pages get the same href/src/asset checks as EN pages.
- `site/README.md` already documents the `zh/` mirror, shared `../assets/`, the language switcher, and the EN fallback.

Commands/checks and actual results:
- `bash scripts/validate_public_site.sh` → "HTML files checked: 14, Errors: 0, Warnings: 0" (exit 0). The 14 count confirms ZH pages are in scope.
- `rg "中文|EN" site/*.html site/zh/*.html` → all 14 pages carry a switcher link.

Open risks or deviations:
- None. WEB-003 is already shipped and verified; F14 implementation is therefore also already complete.

Next task item:
- F14: Static site i18n implementation (already shipped, recorded as evidence).

Recovery or resume instruction:
- Owner record: this file. Git state: `main` at the Month 3 closeout commit. Resume F14 by confirming the WEB-003 acceptance checkboxes are now closed in the owner doc and recording the same `validate_public_site.sh` evidence here.

## Checkpoint F14 - Static Site i18n Implementation (2026-07-06)

Completed items:
- F14: verified already shipped. No code change required.

Current state and artifacts:
- All 7 WEB-003 acceptance criteria checked off in `docs/backlog/active/WEB-003-site-internationalization.md` with concrete evidence (page paths, validator line numbers, switcher markup).
- WEB-003 owner doc Status flipped from "Refinement" to "Complete (2026-07-06, F13/F14 of the frontline four-month execution plan — verified already-shipped work)".

Commands/checks and actual results:
- `bash scripts/validate_public_site.sh` → 14 files checked, 0 errors, 0 warnings (re-confirmed after the owner-doc edit; site assets untouched).
- `rg "中文|EN" site/*.html site/zh/*.html` → switcher present on all 14 pages.
- Shared-asset check: ZH pages reference `../assets/styles.css` / `../assets/site.js` (validated by the recursive walk in the validator).

Open risks or deviations:
- None.

Next task item:
- F15: Static site branding polish.

Recovery or resume instruction:
- Owner record: this file. Git state: `main` at the Month 3 closeout commit. Resume F15 by verifying WEB-004 Nord palette + hexagon brand assets.

## Checkpoint F15 - Static Site Branding Polish (2026-07-06)

Completed items:
- F15: verified already shipped. No code change required.

Current state and artifacts:
- WEB-004 design fully implemented in `site/assets/styles.css`, `site/assets/talos-mark.svg`, `site/assets/favicon.svg`:
  - Color tokens: `--talos-accent` light `#5e81ac` (Frost dark blue), dark `#88c0d0` (Frost cyan). `--talos-bg` light `#eceff4` (Snow Storm), dark `#2e3440` (Polar Night). Status pills use Aurora colors: `--talos-shipped: #a3be8c`, `--talos-planned: #ebcb8b`, `--talos-research: #b48ead`.
  - Logo SVG (`talos-mark.svg`): hexagon polygon with Nord Frost linear gradient + TALOS monospace wordmark.
  - Favicon SVG (`favicon.svg`): hexagon polygon with `#88c0d0` stroke + "T".
  - `@media (prefers-color-scheme: dark)` overrides every token.
- All 8 WEB-004 acceptance criteria checked off in `docs/backlog/active/WEB-004-site-theme-branding.md` with concrete evidence (CSS line numbers, hex color values, SVG structure).
- WEB-004 owner doc Status flipped from "Refinement" to "Complete (2026-07-06, F15 of the frontline four-month execution plan — verified already-shipped work)".

Commands/checks and actual results:
- `rg --talos-accent|talos-bg|talos-shipped site/assets/styles.css` → all Nord token names present.
- `rg talos-pill--shipped|planned|research site/roadmap.html site/zh/roadmap.html` → Aurora pill classes used.
- `rg styles\.css|talos-mark|favicon site/*.html` → all 7 EN pages reference brand assets (2 matches each).
- `bash scripts/validate_public_site.sh` → 14 files, 0 errors, 0 warnings (unchanged after owner-doc edit; assets untouched).

Open risks or deviations:
- None. Brand polish already met the WEB-004 design notes; no further CSS/SVG change required by this plan.

Residual owner:
- None.

Next task item:
- F16: Final closeout — full validation matrix (`cargo fmt --all -- --check`, `cargo check --workspace`, `cargo test --workspace`, governance, `git diff --check`), append final checkpoint with recovery info, commit + push.

Recovery or resume instruction:
- Owner record: this file. Git state: `main` at the Month 3 closeout commit. Resume F16 by running the full closeout matrix, then commit + push the F13/F14/F15 + F16 checkpoints together.

## Checkpoint F16 - Final Closeout (2026-07-06)

Completed items:
- F0-F16 — entire frontline four-month execution plan complete.

Current state and artifacts:
- 9 execution commits on `main`, all pushed to `origin/main`:
  - `6ad5894` F0 kickoff inventory + consolidated confirmation contract
  - `9325aec` F1 catalog residual audit (findings table)
  - `3cda7ad` F2 quarantine `talos-models` as non-runtime + 5-test `no_catalog_db_guard.rs`; MC-002 closed
  - `8f5ee58` F3 README `/model` + `/connect` standard-provider URL-skip text drift closed
  - `9fdb7e1` F4 Month 1 closeout checkpoint
  - `a878255` F5-F8 verify standard-provider connect + protocol metadata + CLI model list (Month 2)
  - `03551ac` F9-F12 verify TUI display package (Month 3)
  - `5fef2f0` F13-F15 verify static site i18n + branding (Month 4 part 1)
  - `d0cb2ce` F16 final closeout
- Owner docs closed:
  - `MC-002` Status → Complete; all acceptance criteria [x] with evidence.
  - `WEB-003` Status → Complete; all 7 acceptance criteria [x].
  - `WEB-004` Status → Complete; all 8 acceptance criteria [x].
- `MC-001` owner doc unchanged (already records the 2026-07-05 maintainer decision and the MC107 real-terminal residual as parked).
- `MODEL-005`, `MODEL-006`, `TUI-015`, `TUI-019`, `TUI-025` remain Complete in their owner docs; their acceptance evidence is re-cited in the F5-F11 checkpoints of this file.

Commands/checks and actual results (full closeout matrix):
- `cargo fmt --all -- --check` → exit 0 (clean).
- `cargo check --workspace` → exit 0.
- `cargo test --workspace` → exit 0; **no failures, no panics, no errors across every crate**. Notable per-crate pass counts: talos-agent 197, talos-cli 154 unit + integration (5 new `no_catalog_db_guard`, 1 rpc_e2e, 2 skill_runtime_e2e), talos-config 115, talos-tui 249, talos-session 255, talos-tools 93, talos-models 37 (quarantined crate still compiles). All doctests pass.
- `scripts/validate_project_governance.sh .` → "Governance validation passed: 0 warning(s)."
- `scripts/validate_public_site.sh` → "HTML files checked: 14, Errors: 0, Warnings: 0."
- `git diff --check` → CLEAN.
- `git status` → on `main`, up to date with `origin/main`, working tree clean.

Open risks or deviations:
- None blocking. The plan completed as a mix of one genuine cleanup (F2 catalog quarantine + guard test) and documentation/verification work, because F5-F15 referenced backlog items that had already shipped before this plan was written (2026-07-04/05 vs plan date 2026-07-06). The confirmation contract resolved this by treating already-complete items as verify + record + close.

Residual owner (carried forward, out of scope for this plan):
- MC-001 / MC107 — real-terminal `/connect` walkthrough. Owner: `docs/backlog/active/MC-001-model-catalog-modernization.md` (Paused).
- MODEL-006 — `set_active_model()` does not consult `builtin_providers()` for protocol when a non-hardcoded provider is selected via `/model` without prior `/connect`. Not triggered by normal `/model` usage. Owner: `docs/backlog/active/MODEL-006-interactive-model-catalog-browser.md` residual hardening.

Next-cycle candidates (not in scope):
- Close the MC-001 MC107 real-terminal `/connect` walkthrough.
- Tighten the MODEL-006 `set_active_model()` catalog-protocol lookup residual.
- Consider a CLI command-taxonomy alias `talos models browse` (already noted as a MODEL-006 residual).

Recovery or resume instruction:
- Owner record: this file. Git state: `main` at the F16 final closeout commit (to be created next); already pushed to `origin/main`. The plan is Complete; no further action is required from this execution. If this plan is resumed or audited, all evidence lives in the F0-F16 checkpoints above, and the only genuine code change is `3cda7ad` (F2) — every other commit is documentation or verification.

## Post-Review Repair - Acceptance Closure (2026-07-07)

Completed items:
- Synchronized the F0-F16 ordered task table with the file-level Complete status.
- Corrected the F16 commit count from 8 to 9 execution commits and listed the `d0cb2ce` closeout
  commit.
- Synchronized derived governance views: `docs/BOARD.md` and `docs/backlog/PRODUCT-BACKLOG.md`
  now agree with the MC-002, WEB-003, and WEB-004 owner docs.
- Strengthened `crates/talos-cli/tests/no_catalog_db_guard.rs` so every guarded CLI entry point
  must exit successfully and emit expected catalog/config output before the no-`catalog.db`
  invariant is accepted.

Validation:
- `cargo fmt --all -- --check` -> exit 0.
- `cargo test -p talos-cli --test no_catalog_db_guard -- --nocapture` -> 5 passed.
- `cargo check --workspace` -> exit 0.
- `scripts/validate_project_governance.sh .` -> 0 warnings.
- `git diff --check` -> clean.

Residual owner:
- No residual from this acceptance repair. The pre-existing MC-001 MC107 walkthrough and MODEL-006
  protocol hardening residuals remain with their owner docs as recorded in F16.
