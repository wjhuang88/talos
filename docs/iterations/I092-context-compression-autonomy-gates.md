# Iteration I092: Context Compression And Autonomy Gates

> Document status: Active
> Published plan date: 2026-07-04
> Planned objective: prove or reject active context compression and split autonomous execution
> features into non-bypass permission slices.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: deterministic compression evidence or rejection plus a permission matrix for
> scheduled/batch/exec-style autonomy.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `MEM-007` | Active context compression | Research | MEM-005/ARCH-006 | Prototype or reject cache-safe deterministic pre-entry compression. |
| `SCHED-001` split | Delayed/scheduled tasks | Planned/In Progress historical | Permission gate | Safe slices named; no hidden scheduled write/exec behavior. |
| `PERM-001` / `TOOL-010` | Permission/autonomy | Blocked/refinement | Permission gate | Deny/ask/allow matrix for batch/autonomous paths. |

### Scope

- Compression applies only to dynamic suffix/tool results, never stable prefix.
- Raw output preservation for export/debug remains mandatory.
- Scheduled/batch/exec behavior must prove non-bypass before runtime expansion.

### Non-Goals

- No ML/ONNX compression model.
- No retroactive compression of cached history.
- No automatic scheduled direct tool execution.
- No permission-default relaxation.

### Acceptance

- Given compression is enabled or disabled,
  When stable prefix is built,
  Then stable-prefix bytes/hash are unchanged.
- Given identical tool output and config,
  When compression runs twice,
  Then compressed output is byte-identical.
- Given raw transcript export,
  When compressed model-facing output exists,
  Then raw full output remains available.
- Given scheduled/batch/autonomous actions,
  When permission checks run,
  Then deny/ask/allow behavior cannot be bypassed.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- Compression determinism/cache tests if implemented.
- Permission regression matrix if autonomy slices are touched.
- `scripts/validate_project_governance.sh .`

### Documentation To Update

- `docs/backlog/active/MEM-007-active-context-compression.md`
- `docs/backlog/active/PERM-001-guardian-exec-policy.md`
- `docs/backlog/active/SCHED-001-delayed-scheduled-tasks.md`
- `docs/backlog/active/TOOL-010-batch-file-operations.md`
- `docs/BOARD.md`

### Risks And Rollback

- Risk: compression corrupts context or hides data needed for export/debug.
- Rollback: reject active compression and preserve MEM-005 only.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-04 | Activation | Activated after I091 completed `/hooks` diagnostics, hook manifest declaration validation, and optional asset distribution policy. Non-terminal inventory disposition: I085 remains Paused with MC107 real terminal `/connect` walkthrough residual; I086-I089 remain planned product-hardening shells; I093 remains the planned next direct-owner shell. I092 starts with cache-stability proof/rejection before autonomy implementation, then a permission matrix for scheduled/batch/exec-style paths. |
| 2026-07-04 | A10 execution | Closed the first MEM-007 evidence slice for bash-only active compression. The existing compressor is deterministic and default-off; added regression tests proving enabling compression does not change stable-prefix bytes, and long bash output is compressed only in the model-facing tool result while the UI event/export surface keeps the full raw output. Corrected docs/comments that previously over-claimed durable JSONL raw-output preservation. |

## Verification Evidence

- `scripts/validate_project_governance.sh .`: passed, 0 warnings.
- `git diff --check`: clean.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p talos-agent`: passed.
- `cargo clippy -p talos-agent -- -D warnings`: passed.
- `cargo test -p talos-agent bash_compression`: 2 matching regression tests passed.
- `cargo test -p talos-agent`: 196 unit tests and 12 doctests passed.
- `cargo test --workspace`: passed.

## Variance And Residuals

- No compression or autonomy runtime behavior has changed at activation.
- A10 is complete for the bash-only evidence slice, not for all MEM-007 strategies. `read`,
  `grep`, `git_diff`, cross-turn dedup, and durable JSONL dual-track raw-output storage remain
  deferred.
- A11 remains the autonomy permission packet and must not ship scheduled direct tool execution,
  Guardian auto-approval, exec DSL implementation, or batch writes before deny/ask/allow matrix
  coverage is explicit.

## Retrospective

- Activation intentionally keeps implementation disabled until cache-stability and permission
  non-bypass evidence exist.
- The existing bash compressor was already a reasonable minimal slice. The main A10 correction was
  adding the missing cache/export regression proof and narrowing documentation to match the actual
  storage boundary.
