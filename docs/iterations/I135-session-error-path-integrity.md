# Iteration I135: Session Error-Path Integrity

> Document status: Planned
> Published plan date: 2026-07-16
> Planned objective: close SESSION-006 without weakening I128 durable-turn atomicity.
> Baseline rule: preserve this target; changed persistence semantics use change control or a new iteration ID.
> MVP deliverable: after a provider fails following a completed tool execution, interactive session resume retains the valid completed tool exchange while durable Runtime abort semantics remain unchanged.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `SESSION-006` | `TOOL-021` finding 2 | Open / P1 | I131 audit; I128 atomic-turn boundary | Preserve a valid completed tool exchange across provider failure and resume. |

### Scope

- Trace the canonical interactive session and I128 durable Runtime paths before editing.
- Define the persistable prefix as completed normalized messages only; never synthesize a tool result or persist an incomplete streamed assistant fragment.
- Persist that prefix before returning a provider error on the canonical interactive session path.
- Make retry/idempotency behavior explicit and prevent duplicate user/tool entries.
- Add a regression proving the durable Runtime still aborts a failed turn without committing a half turn.
- Synchronize SESSION-006 and GitHub Issue #36 after evidence passes.

### Non-Goals

- No provider retry, resumable stream, session/TLOG format, public API, permission, approval, or event-order change.
- No weakening of ADR-042 successful-turn-only durable commit semantics.
- No fabricated tool result after denial, cancellation, timeout, or tool failure.

### Acceptance

- Given a completed tool call/result followed by provider failure, when the interactive session resumes, then the exact completed exchange is present once.
- Given a provider failure before a completed tool exchange, then no partial assistant/tool result is persisted.
- Given a durable Runtime-bound session failure, then the durable turn aborts under ADR-042 and emits no committed-entries success signal.
- Given a retry with the same logical turn, then persisted entries are not duplicated.
- The original TOOL-021 fixture matrix remains green.

### Planned Validation

- Focused `talos-agent`, `talos-session`, and `talos-runtime` error-path tests.
- Runtime fixture: tool success followed by deterministic provider error, process reconstruction, and transcript/resume inspection.
- `cargo fmt --all -- --check`
- `cargo check --workspace --locked`
- `cargo clippy --workspace --locked -- -D warnings`
- `cargo test --workspace --locked`
- `./scripts/release_preflight.sh`
- `scripts/validate_project_governance.sh .`
- `git diff --check`

### Documentation To Update

- `docs/backlog/active/SESSION-006-session-error-path-persistence.md`
- `docs/reference/TOOL-021-ERROR-PROPAGATION-AUDIT-2026-07-16.md`
- `docs/iterations/README.md`, `docs/BOARD.md`, and the execution package
- GitHub Issue #36

### Risks And Rollback

- Risk: confusing interactive prefix preservation with durable failed-turn commit.
- Rollback: retain current error return and fixtures; do not alter ADR-042. Stop for maintainer review if the paths cannot be separated without a format or public-API change.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|

## Verification Evidence

- Pending activation.

## Variance And Residuals

- None at publication.
