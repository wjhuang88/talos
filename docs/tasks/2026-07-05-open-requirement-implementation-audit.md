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
| A3 | Commit/push current verified remediation and audit baseline | Stage commit on `main` | A1-A2 | fmt, focused tests, governance, clippy/workspace evidence recorded | Stop before push only on validation failure | In Progress |
| A4 | Validation internal-service design slice | Implementation plan or first internal API slice | A3 | Tests or owner-doc gate for no-host governance path | Defer to next iteration with explicit owner | Planned |
| A5 | Model browser implementation path | Independent CLI browser plan or first slice | A3 | CLI smoke/headless navigation evidence | Keep bounded/filterable print mode as interim | Planned |
| A6 | Git host-fallback cleanup path | Remove or gate runtime host `git status` leak | A3 | Smoke evidence without host git where feasible | Omit dirty status when internal path unavailable | Planned |

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
