# Open Requirement Implementation Audit — 2026-07-05

## Long-Running Task Contract

Outcome:

- Turn the newly raised open requirements into an ordered, owner-backed implementation path.
- Close the first high-risk design gate for permission approvals before any broad permission change.
- Keep runtime behavior conservative while user-facing CLI/model/catalog fixes already landed are
  verified and committed.

In scope:

- Central audit of newly raised or reopened requirements.
- `PERM-003` reference study and Talos permission taxonomy.
- `VALIDATION-001` internal validation/project-type-adapter planning.
- `MODEL-006` independent CLI browser planning and interim `--available-models` mitigation.
- Linking `TOOL-017` and `GIT-001` residuals to the permission/validation path.
- Stage commits and pushes after validation gates.

Out of scope:

- No broad `bash = allow` default.
- No runtime permission default relaxation.
- No implementation of multi-command/pipe exec before the permission taxonomy lands.
- No main-session TUI coupling for the independent `--available-models` browser.
- No release tag, crate publish, destructive cleanup, credential migration, or remote deployment.

Ordered task items:

| ID | Task | Expected Output | Depends On | Completion Gate | Fallback | Status |
|---|---|---|---|---|---|---|
| A1 | Inventory open requirements and current mitigations | Audit table with owners and order | None | Owner docs linked and board updated | Record unknowns as residuals | Complete |
| A2 | Permission reference study | Reference study and taxonomy for `PERM-003` | A1 | Study cites current sources and updates acceptance | Keep implementation blocked if sources unavailable | Complete |
| A3 | Commit/push current verified remediation and audit baseline | Stage commit on `main` | A1-A2 | fmt, focused tests, governance, clippy/workspace evidence recorded | Stop before push only on validation failure | Complete |
| A4 | Validation internal-service design slice | Implementation plan or first internal API slice | A3 | Tests or owner-doc gate for no-host governance path | Defer to next iteration with explicit owner | Complete |
| A5 | Model browser implementation path | Independent CLI browser plan or first slice | A3 | CLI smoke/headless navigation evidence | Keep bounded/filterable print mode as interim | Complete |
| A6 | Git host-fallback cleanup path | Remove or gate runtime host `git status` leak | A3 | Smoke evidence without host git where feasible | Omit dirty status when internal path unavailable | Complete |

Dependencies and prerequisites:

- Follow `docs/sop/START-ITERATION.md`, `docs/sop/ITERATION-WORKFLOW.md`, and
  `docs/sop/GIT-WORKFLOW.md`.
- Preserve published iteration baselines; append execution facts instead of replacing plans.
- Use reference docs before designing permission behavior.

Artifacts and state owners to update:

- `docs/backlog/active/PERM-003-permission-experience-reference-study.md`
- `docs/backlog/active/VALIDATION-001-internal-validation-service.md`
- `docs/backlog/active/MODEL-006-interactive-model-catalog-browser.md`
- `docs/backlog/active/TOOL-017-exec-multi-parallel-pipe.md`
- `docs/backlog/active/GIT-001-embedded-git-tools.md`
- `docs/reference/PERMISSION-EXPERIENCE-REFERENCE-STUDY-2026-07-05.md`
- `docs/BOARD.md`
- `docs/iterations/README.md` if a new iteration is activated

Validation and acceptance evidence:

- Required for each code stage: `cargo fmt --all -- --check`, focused tests for touched crates,
  `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` where practical,
  `scripts/validate_project_governance.sh .`, and `git diff --check`.
- Required for documentation-only gates: source links/evidence, governance validation, and diff
  review.

Branch, worktree and checkpoint plan:

- Work on current `main` branch.
- Commit by logical phase and push each phase after validation.
- Append a checkpoint here before moving to the next implementation phase.

Allowed permissions and external actions:

- Edit repository files.
- Run local Cargo/governance validation.
- Use web browsing for current reference-project documentation.
- Commit and push current branch after validation.

Destructive or irreversible operations:

- None authorized.
- No force push, tag, publish, release, database migration, credential write, or remote deployment.

Time, cost and resource limits:

- Prefer bounded local validation and focused tests during intermediate phases.
- Full workspace validation is required before committing code stages.

Failure, retry and fallback policy:

- If a validation command fails, fix and rerun the smallest relevant gate first, then rerun the
  broader gate before commit.
- If a reference source is unavailable, record the missing source and keep dependent implementation
  blocked.
- If a host tool is unavailable, record unavailable-tool behavior rather than adding an unbounded
  shell fallback.

Default decisions for foreseeable ambiguity:

- Safety wins over approval reduction.
- Exact command approval is the default reusable bash scope.
- Directory write approvals are directory-scoped.
- Host-tool adapters are project-type-gated.
- Internal Talos tools are preferred over host commands when a typed implementation exists.

Residual-work destination:

- Residual implementation items stay in the relevant backlog owner doc and this task's checkpoint
  log. Derived board rows are updated after owner docs.

## Scope

This audit captures newly raised or reopened requirements that are not fully implemented in the
current codebase and need an explicit implementation path before execution.

## Findings

| Item | Status | Risk | Recommended Path |
|---|---|---|---|
| `PERM-003` Permission Experience Reference Study And Redesign | New P1 refinement | High: directly affects shell execution safety, unattended task stability, and approval fatigue | Do reference study first; do not broaden `bash always` semantics until acceptance matrix is reviewed. |
| `MODEL-006` Interactive CLI Model Catalog Browser | New P1 refinement | Medium: large catalog UX blocks practical model discovery but should not couple to the main session TUI | Keep `--available-models` bounded/filterable now; implement a separate CLI browser mode later. |
| `TOOL-017` exec multi-command/pipe support | Existing refinement, now linked to permissions | High: can reduce bash usage, but multi-step execution changes permission semantics | Re-evaluate after `PERM-003`; align multi-step approval scope with the new permission taxonomy. |
| `VALIDATION-001` Internal Validation Service | Planned P0 | High: currently validation evidence is useful but still host-command oriented | Split into internal governance validation first, then project-type detection and host-tool adapters. |
| `GIT-001` runtime host-git leak | Existing P2 with concrete bug | Medium-high: Talos runtime still shells out to host `git status` in governance status | Replace governance status dirty-tree detection with internal/gix path or omit when unavailable. |

## Implementation Order

1. **PERM-003 design gate**
   - Compare Claude Code, Codex CLI, OpenCode, and Aider command/permission behavior.
   - Produce a Talos permission taxonomy for exact command, command template, directory write,
     network/remote, and long-task preflight scopes.
   - Define which scopes are session-only and which may become persistent config.

2. **VALIDATION-001 internal governance slice**
   - Move governance validation behind an in-process service API.
   - Keep host tools as explicit adapters with unavailable-tool behavior.
   - Add project-type detection before injecting adapter instructions.

3. **MODEL-006 independent CLI browser**
   - Build a CLI-local terminal browser for the packaged catalog.
   - Do not depend on main conversation TUI state.
   - Reuse config merge helpers for provider credential/base URL changes.

4. **TOOL-017 after permission taxonomy**
   - Extend `exec` only after multi-step permission scope is specified.
   - Use it to reduce normal bash usage without making shell a broad escape hatch.

5. **GIT-001 host fallback cleanup**
   - Remove direct host `git status` from runtime governance status.
   - Update prompt guidance so built-in gix-backed Git tools are preferred over `bash git`.

## Current Mitigations Already Landed

- `--available-models` prints `provider/model`.
- `--available-models` default output is bounded and filterable.
- Full catalog output requires `--available-models-all`.
- `--import-models` is no-op compatibility and does not create `catalog.db`.
- Runtime model metadata uses packaged `models.toml`, not `~/.talos/catalog.db`.
- Bash `always` approval no longer shares one broad resource across unrelated subcommands.

## Checkpoints

### 2026-07-05 — A1/A2 Reference Gate

Completed task items:

- A1 open-requirement audit drafted with owner docs for `PERM-003`, `MODEL-006`, `VALIDATION-001`,
  `TOOL-017`, and `GIT-001`.
- A2 permission reference study completed at
  `docs/reference/PERMISSION-EXPERIENCE-REFERENCE-STUDY-2026-07-05.md`.
- `PERM-003` acceptance updated to mark reference study and taxonomy complete.

Current state and artifacts:

- Runtime permission defaults remain unchanged.
- `--available-models` interim mitigation is present: bounded default output, filtering, optional
  full output, and `provider/model` names.
- Runtime model catalog remains packaged `models.toml`; no `catalog.db` runtime path.

Commands/checks and actual results:

- `cargo fmt --all -- --check` passed.
- `scripts/validate_project_governance.sh .` passed with 0 warnings.
- `git diff --check` passed.
- `cargo test -p talos-cli available_model` passed: 3 tests.
- `cargo test -p talos-tools bash_permission_profile` passed: 4 tests.
- `cargo test -p talos-cli connect_tests` passed: 6 tests.
- `cargo clippy --workspace -- -D warnings` passed.
- `cargo test --workspace` passed.

Open risks or deviations:

- Permission UX implementation is intentionally blocked until prompt copy, deny-precedence tests,
  and measured repeated-approval trace are added.
- `VALIDATION-001` still needs implementation work to move governance validation off host-command
  scripts.
- `GIT-001` still needs runtime cleanup for host `git status` fallback.

Next task item:

- A3: commit and push the current verified remediation and audit baseline.

Recovery or resume instruction:

- Resume from this task file, run `git status --short --branch`, verify the A3 validation gates,
  then commit logical changes using the required conventional commit format.

### 2026-07-05 — A4 Internal Governance And Project-Type Detection Slice

Completed task items:

- A3 committed and pushed:
  - `459e214` — model catalog/runtime permission gaps.
  - `9c1004a` — open requirement audit gate and PERM-003 reference study.
- A4 first implementation slice started:
  - `talos validate plan/run --profile governance` now uses an internal governance validation check
    instead of `scripts/validate_project_governance.sh`.
  - Validation evidence distinguishes `execution_mode: "internal"` and
    `execution_mode: "host_tool"`.
  - Project type detection recognizes Talos governance, Rust, Node.js, Python, Go, and Java
    workspaces.
  - Project type detection is implemented through a `ProjectTypeDetector` strategy registry, so
    future project/governance types can be added by registering detectors.
  - Cargo checks are treated as Rust host-tool adapters and are blocked when Rust is not detected.

Current state and artifacts:

- A4 code lives in `crates/talos-cli/src/validation.rs`.
- A4 owner doc update lives in
  `docs/backlog/active/VALIDATION-001-internal-validation-service.md`.
- This is still a CLI-local service slice; the acceptance item for a shared API outside `talos-cli`
  remains open.

Commands/checks and actual results:

- `cargo test -p talos-cli validation` passed: 10 tests.
- `cargo run -p talos-cli -- validate plan --profile governance --json` printed
  `project_types:["talos_governance","rust"]` and an internal governance check.
- `cargo run -p talos-cli -- validate run --profile governance --json` printed a passed
  `execution_mode:"internal"` governance record.
- `cargo fmt --all -- --check` passed.
- `scripts/validate_project_governance.sh .` passed with 0 warnings.
- `git diff --check` passed.
- `cargo clippy --workspace -- -D warnings` passed.
- `cargo test --workspace` passed.

Open risks or deviations:

- The validation service has not yet moved to a shared crate.
- TUI/runtime invocation of internal validation remains open.
- Host-tool adapter instruction injection is still planned, not implemented.

Next task item:

- Commit/push this internal governance/project-type detection slice.

Recovery or resume instruction:

- Resume from `git status --short --branch`; run fmt, `cargo test -p talos-cli validation`, clippy,
  workspace tests if code changed further, governance validation, and diff check before committing.

### 2026-07-05 — A6 Governance Git Runtime Leak Slice

Completed task items:

- `talos_tools::git_dirty_count()` added as a narrow public helper backed by the native `gix`
  status API.
- `talos --governance-status` now calls `git_dirty_count()` instead of spawning
  `git status --porcelain`.
- Governance Git state degrades to an explicit unavailable message if a repository cannot be
  discovered; it does not silently call host Git.
- Identity prompt now prefers built-in Git tools (`git_status`, `git_diff`, `git_log`,
  `git_branch_list`) for read-only inspection and treats host shell Git as an explicit approved
  fallback only.
- `GIT-001` updated to close the runtime dirty-tree and prompt-guidance findings.

Current state and artifacts:

- A6 code touches `crates/talos-tools/src/git.rs`, `crates/talos-tools/src/lib.rs`,
  `crates/talos-cli/src/governance.rs`, `crates/talos-agent/prompts/identity.txt`, and
  `crates/talos-agent/src/prompt/tests.rs`.

Commands/checks and actual results:

- `cargo fmt --all -- --check` passed.
- `cargo test -p talos-cli governance::tests` passed: 8 tests.
- `cargo test -p talos-tools git::tests` passed: 3 tests.
- `cargo test -p talos-agent test_identity_prompt_prefers_builtin_git_tools_over_bash_git` passed:
  1 test.
- `cargo run -p talos-cli -- --governance-status` passed and printed a Git status section via the
  internal/gix path.
- `rg 'Command::new\("git"\)|status --porcelain' crates/talos-cli/src/governance.rs` has no
  matches.

Final validation gates:

- `cargo fmt --all -- --check` passed.
- `cargo test -p talos-cli governance::tests` passed: 8 tests.
- `cargo test -p talos-tools git::tests` passed: 3 tests.
- `cargo test -p talos-agent test_identity_prompt_prefers_builtin_git_tools_over_bash_git` passed:
  1 test.
- `cargo clippy --workspace -- -D warnings` passed.
- `cargo test --workspace` passed.
- `scripts/validate_project_governance.sh .` passed with 0 warnings.
- `git diff --check` passed.

Open risks or deviations:

- Write-oriented Git operations still have documented structured host-`git` fallbacks.
- A6 intentionally closes only the runtime dirty-tree leak and prompt guidance. Write-oriented Git
  fallback policy remains tracked in `GIT-001`.

Next task item:

- Commit/push the runtime Git leak slice, then resume A5.

Recovery or resume instruction:

- Resume from `git status --short --branch`; if A6 has not been committed, review/stage this slice
  and commit after re-running any stale validation gate.

### 2026-07-05 — A5 Independent Model Browser Slice

Completed task items:

- Added `talos --available-models-browser` as an independent command-line terminal browser.
- Kept `--available-models` as bounded/filterable script output; the browser is a separate opt-in
  command.
- Browser state lives in `crates/talos-cli/src/models_browser.rs` and does not depend on the main
  conversation TUI state machine.
- Browser supports `j/k`, arrows, PageUp/PageDown, `g/G`, `/` search, `Enter` selection/setup,
  `c` provider setup, and `q`/Esc quit.
- Authenticated model rows can be selected to save the active provider/model.
- Unauthenticated rows route to a CLI-local API key/base URL prompt and save provider credentials
  without printing existing API key values.
- README and `MODEL-006` updated with usage, scope, and residual manual walkthrough.

Current state and artifacts:

- Code lives in `crates/talos-cli/src/models_browser.rs` and `crates/talos-cli/src/main.rs`.
- `MODEL-006` is `In Progress` because the first implementation slice is complete, but a
  real-terminal manual walkthrough remains before final Complete.

Commands/checks and actual results:

- `cargo fmt --all -- --check` passed.
- `cargo test -p talos-cli models_browser::tests` passed: 5 tests.
- `cargo check -p talos-cli` passed.
- `cargo run -p talos-cli -- --help` passed and listed `--available-models-browser`.
- `cargo run -p talos-cli -- --available-models-browser` in non-TTY mode failed as expected with:
  `--available-models-browser requires an interactive terminal; use --available-models for script output`.

Open risks or deviations:

- Real terminal walkthrough is still required to verify raw-mode UX, alternate-screen cleanup, and
  credential prompt ergonomics.
- No main TUI coupling was introduced.

Next task item:

- Run full validation gates, commit/push A5, then close the concentrated audit task with residuals.

Recovery or resume instruction:

- Resume from `git status --short --branch`; if A5 is not committed, re-run focused tests plus
  full gates before staging.

### 2026-07-05 — Concentrated Audit Closeout

Completed task items:

- A1-A6 are complete.
- User correction on project information detection was incorporated into the `VALIDATION-001`
  owner path: project/governance type detection must remain an extensible detector/strategy
  registry, not a monolithic hardcoded matcher.
- `PERM-003`, `VALIDATION-001`, `MODEL-006`, `TOOL-017`, and `GIT-001` all have owner-backed
  implementation or residual paths.
- Runtime behavior stayed conservative: no broad bash allow, no permission-default relaxation, no
  release tag, no publish, no destructive cleanup, and no main-session TUI coupling for the model
  browser.

Final validation gates:

- `cargo fmt --all -- --check` passed.
- `cargo clippy --workspace -- -D warnings` passed.
- `cargo test --workspace` passed.
- `scripts/validate_project_governance.sh .` passed with 0 warnings.
- `git diff --check` passed.

Residual owner work:

- `PERM-003`: permission UX implementation remains blocked until prompt copy, deny-precedence tests,
  and measured repeated-approval evidence are added.
- `VALIDATION-001`: shared service extraction, TUI/runtime caller, and adapter instruction
  injection remain open; project type detection must stay strategy-registry based.
- `MODEL-006`: real-terminal manual walkthrough remains before Complete.
- `GIT-001`: write-oriented Git fallback policy remains tracked.

Recovery or resume instruction:

- Resume residuals from their owner docs, not from this concentrated audit task. Use a new iteration
  ID if objectives or acceptance targets change.

## Verification Expectations For Future Work

- Permission changes require focused tests proving deny rules override all runtime allow rules.
- CLI browser work requires headless navigation/filtering tests and no-secret rendering tests.
- Validation service work requires tests proving internal profiles do not spawn host commands.
- Git cleanup requires tests or smoke evidence proving `talos governance status` works without host
  `git` when the internal path is available.

## Linked Owner Docs

- `docs/backlog/active/PERM-003-permission-experience-reference-study.md`
- `docs/backlog/active/MODEL-006-interactive-model-catalog-browser.md`
- `docs/backlog/active/TOOL-017-exec-multi-parallel-pipe.md`
- `docs/backlog/active/VALIDATION-001-internal-validation-service.md`
- `docs/backlog/active/GIT-001-embedded-git-tools.md`
