# Iteration I099: Structured Exec And Bash Fallback Reduction

> Document status: Complete
> Published plan date: 2026-07-06
> Planned objective: reduce shell fallback pressure by completing safe structured `exec` workflows.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: parallel and pipe-capable `exec` slices, or explicit deferral evidence, with
> permission facets for every spawned process.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `TOOL-017` | Exec multi-command/parallel/pipe | M1 Complete; M2-M4 Planned | TOOL-016, PERM-003 | Structured parallel/pipe execution or safe deferral. |
| `PERM-003` | Permission experience | Complete | I098 | `exec` permission semantics align with scoped preflight. |

### Scope

- Implement `exec` parallel execution only with direct argv spawning and bounded output.
- Implement pipe chains only when stdin/stdout ownership, timeout, cancellation, and failure
  propagation are deterministic and tested.
- Align multi-step permission profiles with per-step command and cwd facets.
- Audit common bash usages and classify them as typed tool, `exec`, host-tool adapter, or exact
  bash fallback.

### Non-Goals

- No shell parsing, glob expansion, redirection, command substitution, background jobs, or shell
  condition syntax.
- No arbitrary script runner.
- No change to bash policy beyond documentation/audit unless separately accepted through I098.

### Acceptance

- Given `exec` receives parallel steps,
  When it runs,
  Then each step has independent timeout/cancel/failure evidence and permission facets.
- Given `exec` receives a pipe chain,
  When it runs,
  Then stdout-to-stdin flow is deterministic, bounded, and does not invoke a shell.
- Given one step is denied,
  When permission evaluation runs,
  Then no denied step is spawned.
- Given a workflow still requires bash,
  When the audit matrix is updated,
  Then the reason is explicit.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo test -p talos-tools exec_tool`
- `cargo check -p talos-tools`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `scripts/validate_project_governance.sh .`
- `git diff --check`

### Documentation To Update

- `docs/backlog/active/TOOL-017-exec-multi-parallel-pipe.md`
- `docs/reference/EXEC-TOOL-PERMISSION-POLICY-2026-07-02.md`
- `docs/backlog/active/PERM-003-permission-experience-reference-study.md`
- `docs/BOARD.md`

### Risks And Rollback

- Risk: pipe/parallel semantics hide process execution behind one approval.
- Rollback: keep M1 sequential behavior and record unsafe M2/M3 blockers in TOOL-017.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-06 | Planning | Created as Month 2 of the 2026-07-06 autonomy/permission/runtime hardening plan. Not active until I098 closes or is explicitly paused. |
| 2026-07-06 | Activation | Activated after I098 completed and was pushed. TOOL-017 remains the owner: M1 sequential steps are complete; M2 parallel, M3 pipe chains, and M4 permission strategy alignment are selected. Activation does not authorize shell parsing, glob expansion, redirection, background jobs, arbitrary scripts, broad bash permission changes, release actions, or permission-default relaxation. |
| 2026-07-06 | Completion | Closed the bounded direct-argv slice: `exec` now supports parallel steps, bounded pipe chains, parallel pipe chains, and per-step permission facets without shell parsing. Bash fallback matrix recorded in TOOL-017. |

## Verification Evidence

- `cargo fmt --all -- --check`: passed.
- `cargo test -p talos-tools exec_tool`: passed, 20 tests.
- `cargo check -p talos-tools`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy --workspace -- -D warnings`: passed.
- `cargo test --workspace`: passed.

## Variance And Residuals

- `steps` and `pipes` are mutually exclusive in this slice. The mixed shape in the proposal remains
  deferred because it adds scheduling semantics without improving the immediate bash fallback
  reduction gate.
- Pipe chains use bounded in-memory stdout-to-stdin transfer rather than unbounded streaming pipes.
  This preserves output/resource bounds.

## Retrospective

- Structured `exec` can now absorb common multi-command and stdout-to-stdin workflows that
  previously pushed users toward `bash`, while still exposing every spawned process to permission
  evaluation. The remaining shell fallback cases are explicit shell-language features, not hidden
  gaps in the direct execution path.
