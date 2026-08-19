# PROVIDER-006: Bounded Retry Progress Contract

| Field | Value |
|---|---|
| Story ID | PROVIDER-006 |
| Type | Provider / Runtime Observability Story |
| Priority | P1 |
| Status | Planned / Unclaimed |
| Source | [GitHub Issue #278](https://github.com/wjhuang88/talos/issues/278) |
| Selected Iteration | I210 |
| Depends On | I209 disposition; ADR for a semver-compatible provider progress contract |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #278 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-19 |
| Handoff / Release Condition | Accept an ADR defining the semver-compatible progress contract, then establish an effective claim before implementation. |

## Identity / Goal / Value

Users waiting on provider dispatch or retry backoff need truthful bounded progress based on actual
provider attempt and wait facts. The runtime and TUI must not infer progress from elapsed timers or
present a static indefinite connection label when the provider knows the current retry state.

## Confirmed Current Boundary

- Retry attempt and backoff facts currently exist only in `talos-provider` structured tracing.
- `TurnPhase::Retrying` has no production producer.
- `LanguageModel::stream()` cannot expose dispatch/backoff progress before it returns a stream.
- A truthful cross-crate projection therefore needs an explicit public progress contract or an
  equivalent protocol decision; I209 excludes that API change.

## Scope

- Decide and document a semver-compatible provider progress contract in a new ADR.
- Emit actual current attempt and bounded backoff/dispatch facts without changing retry policy.
- Project those facts through runtime status to CLI/TUI consumers.
- Keep cancellation responsive during dispatch, retry backoff and first-packet wait.
- Add deterministic contract, cancellation and presentation tests.

## Additional Acceptance - 2026-08-19 Connecting/Reconnecting Presentation

- The model-request activity surface starts at `Connecting…` for the initial provider dispatch.
- After a structured retryable provider error or timeout, it changes to `Reconnecting… (attempt
  n/m)` and remains visible through the corresponding bounded backoff/next-attempt wait.
- Success, terminal failure and cancellation clear the activity state.
- `n/m` and every state transition come from typed provider progress events; the TUI must not parse
  error text or infer retry counts from elapsed time.
- This is a projection requirement only: retry policy, max attempts, timeout, backoff and
  cancellation semantics remain unchanged.

## Exclusions

- No retry count, timeout default, jitter, deadline or provider-selection policy redesign.
- No new dependency, persistence format, Desktop/Dashboard behavior or release work.
- No fabricated attempt counters based on wall-clock time and no use of terminal error events as
  progress notifications.

## Acceptance For Future Implementation

- Given initial provider dispatch, retry backoff or first-packet wait, status is driven by actual
  provider facts and identifies a bounded current wait without exposing secrets.
- Cancellation during each wait boundary promptly terminates the bound turn and preserves the
  durable terminal-cancelled outcome.
- Existing provider implementations retain source compatibility or follow the accepted migration
  contract; any compatibility tradeoff is explicit in the ADR.
- Provider retry policy and dependency closure remain unchanged.
- Focused locked tests, workspace validation and user-facing status documentation pass at the
  reviewed exact head.

## Required Reads

- `docs/sop/CHANGE-CONTROL.md`
- `docs/decisions/README.md`
- `docs/backlog/active/TUI-051-resumed-session-interactivity.md`
- `docs/iterations/I209-resumed-session-interactivity.md`
- `crates/talos-provider/src/lib.rs`
- `crates/talos-provider/src/openai.rs`
- `crates/talos-agent/src/session/turn.rs`
