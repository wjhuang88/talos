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

## Claim Preparation - 2026-08-20

| Field | Value |
|---|---|
| Claim State | Unclaimed; governance proposal not yet effective |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline session |
| Work Slice | I210/PROVIDER-006 only: ADR-062 typed progress contract; additive defaulted provider progress entrypoint; built-in OpenAI-compatible and Anthropic dispatch/backoff/first-packet facts; ordered Agent/session/conversation projection; Connecting/Reconnecting activity presentation; deterministic compatibility, retry, cancellation and UI tests; directly affected docs. Excludes retry/timeout/backoff policy changes, error-text/timer inference, persistence, dependencies, release/publication, Desktop/Dashboard and unrelated I198/I211 work. |
| Claimed At | Not effective until target-branch merge |
| Source Issue | #278 |
| Governance Claim PR | Pending |
| Authorization Mode | Independent review proposed |
| Authorization Evidence | Pending exact-head review, CI, both governance validators and merge-time CAS. |
| Implementation PR | Not started |
| Last Updated | 2026-08-20 |
| Handoff / Release Condition | This preparation record grants no implementation authority. Finalize the actual claim PR number and accepted ADR, merge the claim to `main`, then create implementation work only from that merge or later exact `main`. |

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

## 2026-08-20 Architecture And Claim Preparation Checkpoint

ADR-062 selects a request-local typed progress channel and a defaulted additive `LanguageModel`
method so existing third-party provider implementations retain their current source contract. The
existing non-exhaustive Agent/session progress path carries the facts; a new reconnecting phase owns
the visible `n/m` presentation while the legacy retry phase remains available. The ADR records the
pre-1.0 minor-release requirement for the new public conversation enum variant; I210 itself does
not change versions or authorize release.

The exact-main inventory and overlap audit are recorded in I210. No active implementation or claim
PR overlaps Issue #278. This governance proposal remains ineffective until its actual PR number,
accepted decision evidence, exact-head checks and merge-time CAS are complete on `main`.
