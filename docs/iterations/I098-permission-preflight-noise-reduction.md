# Iteration I098: Permission Preflight And Low-Noise Execution Policy

> Document status: Active
> Published plan date: 2026-07-06
> Planned objective: make long-running permission needs inspectable up front while preserving deny
> precedence and avoiding broad bash approval.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a test-backed long-task permission preflight packet plus approval trace evidence
> showing lower repeated-prompt noise without weakening write/execute gates.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `PERM-003` | Permission experience | Complete; refinement allowed | PERM-001, PERM-002 | Preflight model and measured prompt-noise trace. |
| `PERM-001` | Guardian/exec approval policy | In Progress; autonomy matrix complete | ADR-011, ADR-012 | No self-approval or permission-default relaxation. |

### Scope

- Add or document a long-task permission preflight packet made of normal scoped permission facets.
- Preserve current exact/template bash behavior and directory write `always` behavior.
- Improve traceability of why an approval is reused or remains exact.
- Add tests for deny precedence, reusable low-risk template scopes, and high-risk exact fallback.

### Non-Goals

- No blanket `bash` allow.
- No persistent user config allowlist unless separately designed and approved.
- No model self-approval, Guardian approval, timeout approval, or scheduled direct tool execution.
- No expansion of write-capable tool defaults.

### Acceptance

- Given a long task has expected tool operations,
  When Talos builds a preflight packet,
  Then the user can see each reusable permission scope before execution.
- Given a configured deny rule conflicts with a runtime/session allow,
  When permission evaluation runs,
  Then the deny rule wins.
- Given a low-risk bash template such as `cat` in one cwd was approved as `always`,
  When the same command family targets another simple relative file in that cwd,
  Then it does not prompt again.
- Given a high-risk, mutating, complex shell, absolute path, parent traversal, or network command,
  When permission evaluation runs,
  Then it stays exact or asks according to the existing policy.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo test -p talos-tools bash_tool`
- `cargo test -p talos-cli approval::tests`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `scripts/validate_project_governance.sh .`
- `git diff --check`

### Documentation To Update

- `docs/backlog/active/PERM-003-permission-experience-reference-study.md`
- `docs/reference/PERMISSION-LONG-TASK-TRACE-2026-07-05.md` or a successor trace file
- `docs/BOARD.md`
- `docs/backlog/PRODUCT-BACKLOG.md`

### Risks And Rollback

- Risk: preflight UI implies broader authority than the underlying permission facets.
- Rollback: keep runtime permission behavior unchanged and ship only documentation/trace evidence.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-06 | Planning | Created as Month 1 of the 2026-07-06 autonomy/permission/runtime hardening plan. Not active until explicit activation. |
| 2026-07-06 | Activation | Activated after maintainer direction to start the long-running task. Inventory disposition: I085 remains Paused; I086-I089 remain Planned; I099-I101 remain Planned and ordered after I098/I099/I100 respectively; MODEL-006 remains In Progress for I101 only; PERM-003 is complete but selected for refinement; PERM-001 remains In Progress with Guardian auto-approval and exec DSL disabled. Activation authorizes permission preflight/traceability only, not broad bash allow, permission-default relaxation, release action, tag, publish, or runtime `catalog.db` behavior. |

## Verification Evidence

- Pending.

## Variance And Residuals

- Pending.

## Retrospective

- Pending.
