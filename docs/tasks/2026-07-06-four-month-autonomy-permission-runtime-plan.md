# 2026-07-06 Four-Month Autonomy, Permission, Runtime Hardening Plan

> Status: In Progress — I099 active
> Created: 2026-07-06
> Timebox: 16 weeks / roughly 4 months
> Owner boundary: senior-agent owned; permission/runtime/governance slices require direct review
> Trigger: maintainer requested a new four-month long-running task plan.
> Baseline rule: this file is the execution contract. Append checkpoints instead of replacing the
> plan. Changed objectives use a new task or iteration ID.

## Outcome

Make Talos materially more usable for long-running self-development without weakening the safety
boundary. The plan focuses on the current high-risk gaps: noisy shell approvals, incomplete
structured execution, project-type-aware validation, independent model-catalog browsing, native Git
fallback reduction, and evidence needed for future self-bootstrap qualification.

This plan is not execution authorization. Activation, commits, pushes, dependency upgrades, or
permission-boundary changes still follow the phase gates below and any explicit maintainer
confirmation required by the active iteration.

## In Scope

- Four planned iteration shells:
  - I098: Permission preflight and low-noise execution policy.
  - I099: Structured exec parity and bash fallback reduction.
  - I100: Project intelligence, validation adapters, and governance routing.
  - I101: Model catalog browser closeout, Git fallback tracking, and self-bootstrap evidence.
- Convert recent permission fixes into a more inspectable preflight model for long tasks.
- Finish remaining `exec` M2-M4 design slices only where permission facets remain explicit.
- Extend project-type detection through a registry-style strategy surface, not ad hoc conditionals.
- Keep validation internal-first and language-neutral; host tools remain adapters.
- Close human-facing model catalog residuals without reintroducing `catalog.db`.
- Tighten model setup UX: built-in providers must not ask users for a base URL during connect;
  only custom providers require a URL.
- Make large model-list rendering incremental/scroll-loaded so the full packaged catalog does not
  freeze the terminal by rendering every row at once.
- Continue `gix` tracking with replacement gates for host-Git fallbacks.

## Out Of Scope

- No blanket `bash` allow, model self-approval, Guardian auto-approval, or permission-default
  relaxation.
- No tag, release, crate publish, GitHub Release, force push, or destructive Git cleanup.
- No native Git dependency such as `git2`/libgit2.
- No remote plugin marketplace, auto-discovery, browser automation, cookies, profile reuse, or
  write-capable plugin expansion.
- No arbitrary user-supplied validation command execution.
- No `v1.0.0` or REL-002 qualification claim unless the evidence gate is independently satisfied.

## Ordered Task Items

| ID | Task | Expected Output | Depends On | Completion Gate | Fallback | Status |
|---|---|---|---|---|---|---|
| C0 | Establish execution baseline | This task record, I098-I101 planned shells, Board, backlog, and iteration index name the new track. | Maintainer request | Governance validation and `git diff --check` pass. | Keep task Planned with exact blocker. | Planned |
| C1 | Activate I098 | Start permission preflight/noise-reduction iteration after inventory. | C0 | I098 Active and non-terminal inventory disposition recorded. | Keep Planned if conflicting active work blocks activation. | Complete |
| C2 | Long-task permission preflight | A readable preflight packet lists expected scoped approvals before unattended work. | C1 | Tests prove deny precedence, directory write scope, bash template scope, and no broad bash allow. | Keep current runtime rules and ship docs-only preflight guidance. | Complete |
| C3 | Permission trace and UX evidence | Approval prompts and trace output explain why a repeated approval is or is not reused. | C2 | Recorded trace shows prompt count reduction and high-risk exact fallback behavior. | Record unresolved noisy command families in PERM-003. | Complete |
| C4 | Activate I099 | Start structured exec parity iteration. | C3 | I098 closed or paused with exact residuals. | Keep I099 Planned. | Complete |
| C5 | Exec parallel and pipe slices | `exec` supports approved parallel and pipe workflows without shell parsing. | C4 | `talos-tools` tests prove timeout/cancel/failure behavior and per-step permission facets. | Ship only parallel or only pipe if the other lacks safe semantics. | Planned |
| C6 | Bash fallback reduction audit | Identify bash calls that should become typed tools/adapters or remain exact shell. | C5 | Audit matrix updated; no permission broadening. | Keep bash exact/template behavior and record blockers. | Planned |
| C7 | Activate I100 | Start project-intelligence and validation-adapter iteration. | C6 | I099 closed or paused with exact residuals. | Keep I100 Planned. | Planned |
| C8 | Detector/adapters hardening | Project detectors and host-tool adapter guidance are extensible and test covered. | C7 | Tests cover Rust/Node/Python/Go/Java/mixed/governance and no unrelated adapter injection. | Keep existing detector registry and record missing ecosystem. | Planned |
| C9 | Governance routing evidence | Talos can recognize governance tasks and use internal validation/mutation gates. | C8 | `/validate governance` and governance preview/write paths remain internal-first and tested. | Keep governance read-only for any risky mutation class. | Planned |
| C10 | Activate I101 | Start model/Git/self-bootstrap evidence closeout. | C9 | I100 closed or paused with exact residuals. | Keep I101 Planned. | Planned |
| C11 | Model catalog browser closeout | Independent CLI browser walkthrough and docs close MODEL-006 residuals. | C10 | Real-terminal evidence, no-secret rendering, `/model` vs `/connect` separation confirmed. | Record terminal blocker without faking walkthrough. | Planned |
| C12 | Standard-provider connect cleanup | Built-in catalog providers use catalog-defined API endpoint metadata and do not prompt for URL; custom providers still require URL. | C11 | Tests cover standard provider setup, custom provider setup, config merge, and no-secret rendering. | Keep existing prompt only with an explicit MODEL-006 blocker. | Planned |
| C13 | Incremental model-list rendering | Model browser/listing renders only the visible/search window or scroll-loaded chunks instead of the full catalog at once. | C11 | Tests or terminal smoke evidence prove large catalogs remain responsive and selection/search state stays correct. | Keep bounded `--available-models` output and record browser performance blocker. | Planned |
| C14 | Git fallback tracking | Re-check `gix` capability and reduce host fallback only when safe. | C10 | GIT-001 matrix updated; any dependency update has full workspace validation. | Keep host fallback and record replacement trigger. | Planned |
| C15 | Final self-bootstrap evidence packet | REL-002 evidence says exactly what is and is not Talos-primary. | C11, C12, C13, C14 | Full validation and closeout docs pass; no release overclaim. | Mark Partial with residual owners. | Planned |

## Dependencies And Prerequisites

- PERM-003 is complete as the current permission taxonomy and trace baseline.
- TOOL-017 M1 is complete; M2-M4 remain planned.
- VALIDATION-001 is complete for the first internal validation service slice.
- MODEL-006 is in progress and requires real-terminal walkthrough evidence.
- GIT-001 remains the owner for continuing `gix` tracking and host fallback decisions.
- REL-002 remains No-go until Talos is primary for planning, implementation, validation, docs, and
  evidence capture.

## Artifacts And State Owners To Update

- This task record.
- Iterations I098-I101.
- `docs/iterations/README.md`.
- `docs/backlog/active/PERM-003-permission-experience-reference-study.md`.
- `docs/backlog/active/TOOL-017-exec-multi-parallel-pipe.md`.
- `docs/backlog/active/VALIDATION-001-internal-validation-service.md`.
- `docs/backlog/active/MODEL-006-interactive-model-catalog-browser.md`.
- `docs/backlog/active/GIT-001-embedded-git-tools.md`.
- `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md`.
- `docs/backlog/PRODUCT-BACKLOG.md`.
- `docs/BOARD.md`.
- ADRs only if permission semantics, dependency policy, or runtime public API boundaries change.

## Validation And Acceptance Evidence

Planning-only baseline:

```sh
scripts/validate_project_governance.sh .
git diff --check
```

Every implementation phase must run:

```sh
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace -- -D warnings
cargo test --workspace
scripts/validate_project_governance.sh .
git diff --check
```

Phase-specific evidence:

- I098: approval trace before/after, deny-precedence tests, repeated-object and different-object
  bash template tests, directory write-scope tests.
- I099: direct `exec` tests for sequential/parallel/pipe behavior, timeout/cancel behavior, and
  permission facets for every spawned step.
- I100: project detector tests, adapter-instruction injection tests, internal governance validation
  tests, and TUI-safe validation command tests.
- I101: real terminal model-browser walkthrough, standard-provider connect URL-prompt regression,
  custom-provider URL requirement, incremental/scroll-loaded large-catalog rendering evidence,
  no-secret rendering checks, GIT-001 capability matrix, and REL-002 self-bootstrap evidence
  classification.

## Branch, Worktree And Checkpoint Plan

- Work in the current worktree unless the maintainer requests a branch.
- Use one logical commit per phase; push only when the user asks or when the active phase contract
  explicitly authorizes phase push.
- Append checkpoints before moving from C3 to C4, C6 to C7, C9 to C10, and C15 final closeout.
- Each checkpoint must include completed item IDs, actual commands/results, open deviations, next
  item, and recovery instructions.

## Allowed Permissions And External Actions

Allowed by this plan after activation:

- Edit repository files in the workspace.
- Run local build, lint, tests, governance checks, and deterministic terminal/runtime smoke tests.
- Inspect public crate/source metadata for `gix` only when needed for capability tracking.

Not allowed without separate explicit approval:

- Push commits, tags, or release artifacts.
- Publish crates or GitHub Releases.
- Use credentials, paid services, remote plugin install, marketplace behavior, browser profiles, or
  authenticated browser state.
- Force push, reset, clean, rebase, destructive migration, or broad dependency changes.

## Destructive Or Irreversible Operations

No destructive or irreversible production operation is authorized. Destructive behavior is limited
to temporary test fixtures and must be covered by tests.

## Time, Cost And Resource Limits

- Timebox: 16 weeks.
- Monetary spend: zero.
- Network: public metadata only when explicitly needed.
- Retry deterministic validation failures at most twice after concrete fixes before recording a
  blocker.

## Failure, Retry And Fallback Policy

- If a permission change cannot prove deny precedence, do not ship it.
- If an `exec` feature requires shell parsing or hidden command expansion, defer it.
- If a project detector becomes a monolithic hardcoded branch, stop and redesign around registered
  detector strategies.
- If `gix` does not safely replace a host fallback, retain the fallback and record the replacement
  trigger.
- If self-bootstrap evidence remains Codex-primary, record it as non-qualifying evidence.

## Default Decisions For Foreseeable Ambiguity

- Prefer internal typed tools over bash.
- Prefer explicit scoped preflight over global approval modes.
- Prefer read-only/preview governance before write-capable behavior.
- Prefer adapter-specific guidance after project detection over generic language assumptions.
- Prefer honest No-go release posture over optimistic qualification.

## Residual-Work Destination

- Permission UX and policy residuals: PERM-003 and PERM-001.
- Structured execution residuals: TOOL-017 and TOOL-016.
- Validation/project-intelligence residuals: VALIDATION-001 and GOV-003.
- Model catalog residuals: MODEL-006 and MC-001.
- Git residuals: GIT-001 and ADR-010.
- Self-bootstrap residuals: REL-002 and RUNTIME-001.

## Checkpoints

### C0 — Planning Baseline Drafted (2026-07-06)

Completed task items:

- Drafted this four-month long-running task plan.
- Created planned iteration shells I098-I101.
- Synchronized Board, Product Backlog, and iteration index entries.

Commands/checks and actual results:

- `scripts/validate_project_governance.sh .`: passed, 0 warnings.
- `git diff --check`: clean.

Next task item:

- C1: activate I098 only after explicit maintainer direction.

Recovery or resume instruction:

- Run `git status --short --branch`.
- Read this task file, I098, PERM-003, TOOL-017, VALIDATION-001, MODEL-006, GIT-001, and REL-002
  before activating work.

### C1 — I098 Activated (2026-07-06)

Completed task items:

- Activated I098 after maintainer direction to start executing the long-running task.
- Recorded non-terminal inventory disposition:
  - I085 remains Paused with MC107 real-terminal `/connect` walkthrough residual and is not
    reopened.
  - I086-I089 remain planned product-hardening shells.
  - I099-I101 remain planned and depend on I098/I099/I100 completion or explicit pause.
  - MODEL-006 remains In Progress and is selected only for I101.
  - PERM-003 remains Complete but is selected for I098 refinement under its existing taxonomy.
  - PERM-001 remains In Progress with Guardian auto-approval and exec DSL still disabled.
  - No permission-default relaxation, broad bash allow, release action, tag, publish, or runtime
    `catalog.db` path is authorized by this activation.

Current state and artifacts:

- I098 is Active.
- This task is In Progress.
- Implementation starts with permission preflight and traceability only.

Commands/checks and actual results:

- Pending activation-document validation.

Next task item:

- C2: implement the long-task permission preflight packet without weakening permission facets.

Recovery or resume instruction:

- Run `git status --short --branch`.
- Read I098, PERM-003, PERM-001, `crates/talos-cli/src/approval.rs`,
  `crates/talos-permission/src/lib.rs`, and `crates/talos-tools/src/bash_tool.rs`.

### C2/C3 — Permission Preflight And Traceability Closed (2026-07-06)

Completed task items:

- Added `talos permissions preflight`.
- The preflight command accepts repeated `--operation 'tool={"json":"input"}'` entries and uses
  the real registered tool permission profile.
- The command prints current permission decision, reusable `always` scopes, and explicit notes that
  preflight is read-only and does not execute tools or install allow rules.
- Added JSON output for machine-readable long-task planning.
- Kept configured deny precedence and existing bash exact/template policy unchanged.
- Updated README, PERM-003, and the permission long-task trace with preflight evidence.

Current state and artifacts:

- I098 is complete.
- I099 remains planned and is the next implementation phase.

Commands/checks and actual results:

- `cargo fmt --all -- --check`: passed.
- `cargo test -p talos-cli permissions`: passed, 4 tests.
- `cargo test -p talos-cli approval::tests`: passed, 13 tests.
- `cargo test -p talos-tools bash_tool`: passed, 32 tests.
- `cargo run -p talos-cli -- permissions preflight --operation 'bash={"command":"cat Cargo.toml"}' --operation 'bash={"command":"rm generated.txt"}'`: passed; printed read-only packet with reusable `cat` template and exact mutating `rm` scope.
- `cargo run -p talos-cli -- permissions preflight --json --operation 'bash={"command":"cargo test approval"}'`: passed; printed `bash:validation_build:template:<cwd>:cargo:test`.
- `cargo check --workspace`: passed.
- `cargo clippy --workspace -- -D warnings`: passed.
- `cargo test --workspace`: passed.

Open risks or deviations:

- I098 does not install preflight approvals. It only makes scoped permission needs inspectable before
  a long task. This is intentional to avoid hidden authorization.

Next task item:

- C4: activate I099 for structured exec parity.

Recovery or resume instruction:

- Run `git status --short --branch`.
- Read I099 and TOOL-017 before activating the next phase.

### C4 — I099 Activated (2026-07-06)

Completed task items:

- Activated I099 after I098 completed and was pushed.
- Confirmed TOOL-017 status: M1 sequential steps complete; M2 parallel, M3 pipe chains, and M4
  permission strategy alignment remain planned.
- Preserved the no-shell-parser boundary: I099 may add direct argv parallel and pipe behavior only;
  it may not add glob expansion, redirection, background jobs, arbitrary scripts, or broad bash
  permission changes.

Current state and artifacts:

- I099 is Active.
- I100-I101 remain Planned.

Commands/checks and actual results:

- Pending activation-document validation.

Next task item:

- C5: implement or safely defer structured exec parallel/pipe slices with tests.

Recovery or resume instruction:

- Run `git status --short --branch`.
- Read I099, TOOL-017, `docs/proposals/exec-multi-command-parallel-pipe.md`, and
  `crates/talos-tools/src/exec_tool.rs`.
