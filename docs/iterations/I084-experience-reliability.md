# Iteration I084: Experience Reliability — Thinking, Timeout, Retry, And Status

> Document status: Planned
> Published plan date: 2026-07-03
> Planned objective: Execute the first UX reliability series: provider thinking compatibility,
> first-packet and stream-idle timeout detection, retry/backoff, and clear TUI model-call status.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: model calls become observable and bounded: users can see connecting, retrying,
> thinking, generating, timeout, failure, and cancellation states without corrupting history.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| UX100 | MODEL-003/UX-001 | ADR-needed/Planned | ADR-013, reasoning proposal | ADR for provider reasoning/thinking boundary |
| UX101 | MODEL-003/TUI-020 | Planned/Complete | UX100 | Provider thinking stream chunks normalize to preview events |
| UX102 | MODEL-003/MODEL-001 | Planned/Complete catalog | UX100 | Provider request-side reasoning config mapping |
| UX103 | PROVIDER-002 | Planned | provider stream clients | First-packet and stream-idle timeout detection |
| UX104 | PROVIDER-002 | Planned | UX103 | Retry classifier and exponential backoff |
| UX105 | UX-001/TUI | Planned | UX101-UX104 | TUI/conversation status bridge |
| UX106 | UX-001 | Planned | UX100-UX105 | Docs, validation, and residual closeout |

### Scope

- Add the ADR needed before reasoning/thinking provider request schema changes.
- Normalize provider-specific thinking stream fields into Talos preview semantics.
- Add bounded first-packet and stream-idle timeout behavior for provider streams.
- Add retry/backoff for safe, retryable provider failures.
- Surface clear status states in conversation/TUI without duplicating durable history.

### Non-Goals

- No hidden chain-of-thought exposure by default.
- No provider failover or automatic model switching.
- No retry after assistant text/tool-call output has begun unless a later ADR approves resumable
  streams.
- No plugin, distribution, release, browser, or permission-default changes.

### Acceptance

- Given a thinking-capable provider stream, when reasoning chunks arrive, then Talos displays them in
  the live preview and keeps finalized history clean.
- Given no provider packet arrives before the first-packet timeout, when the timeout fires, then the
  user sees a timeout state and the turn exits cleanly.
- Given a stream becomes idle after partial progress, when the idle timeout fires, then Talos fails
  visibly without duplicating text or hanging.
- Given a retryable failure occurs before irreversible output, when retry budget remains, then Talos
  retries with exponential backoff and shows attempt status.
- Given a non-retryable provider error occurs, when the error is classified, then Talos fails without
  retrying and shows an actionable reason.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test -p talos-provider`
- `cargo test -p talos-conversation`
- `cargo test -p talos-tui`
- `cargo clippy -p talos-provider -p talos-conversation -p talos-tui -- -D warnings`
- `cargo test --workspace` at closeout
- `cargo clippy --workspace -- -D warnings` at closeout
- `scripts/validate_project_governance.sh .`

### Documentation To Update

- `docs/backlog/active/UX-001-experience-reliability-program.md`
- `docs/backlog/active/MODEL-003-reasoning-thinking-support.md`
- `docs/backlog/active/PROVIDER-002-response-reliability-timeout-retry.md`
- ADR-034 reasoning/thinking boundary
- README/reference config if user-visible config fields land
- `docs/BOARD.md` after owner docs

### Risks And Rollback

- Risk: reasoning implementation exposes hidden chain-of-thought. Rollback: emit only provider-marked
  visible thinking preview and strip hidden reasoning by default.
- Risk: retry duplicates output. Rollback: allow retries only before assistant text/tool-call output.
- Risk: timeout defaults are too aggressive. Rollback: keep defaults conservative and configurable
  after the first implementation evidence.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-03 | Planning | Created from maintainer feedback that thinking compatibility, timeout, and retry behavior should move ahead of lower-impact extension work. |

## Verification Evidence

- Planned.

## Variance And Residuals

- Planned.

## Retrospective

- Pending.
