# PROVIDER-006: Bounded Retry Progress Contract

| Field | Value |
|---|---|
| Story ID | PROVIDER-006 |
| Type | Provider / Runtime Observability Story |
| Priority | P1 |
| Status | Review / Claimed |
| Source | [GitHub Issue #278](https://github.com/wjhuang88/talos/issues/278) |
| Selected Iteration | I210 - Review / Claimed |
| Depends On | I209 disposition; ADR for a semver-compatible provider progress contract |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline session |
| Work Slice | I210/PROVIDER-006 only: ADR-062 typed progress contract; additive defaulted provider progress entrypoint; built-in OpenAI-compatible and Anthropic dispatch/backoff/first-packet facts; ordered Agent/session/conversation projection; Connecting/Reconnecting activity presentation; deterministic compatibility, retry, cancellation and UI tests; directly affected docs. Excludes retry/timeout/backoff policy changes, error-text/timer inference, persistence, dependencies, release/publication, Desktop/Dashboard and unrelated I198/I211 work. |
| Claimed At | 2026-08-20 |
| Source Issue | #278 |
| Governance Claim PR | #321 |
| Authorization Mode | Independent review |
| Authorization Evidence | PR #321 exact head `4d45f1ba890fa7cb1ea6f6f058ecb0f0916eb639` passed CI `32322271343`, independent governance/architecture review `5350249740`, both governance validators and merge-time CAS, then merged to `main` as `e58fbd399a7071aad7ad8fd846a82f2745611fa0`. |
| Implementation PR | #323 |
| Last Updated | 2026-08-20 |
| Handoff / Release Condition | Claim is effective on `main@e58fbd39`. Create implementation work only from this merge or later exact `main`; implementation still requires the separately governed I210 scope and final review gates. |

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
PR overlaps Issue #278. Governance PR #321 is now effective on `main@e58fbd39`; it authorizes only
the bounded I210 implementation scope recorded above, not release, publication or unrelated work.

## 2026-08-20 Implementation Evidence Checkpoint

Implementation commit `6efee2b8` is the pre-existing source evidence for the I210 implementation PR.
It adds typed initial/retry/backoff/first-packet progress, ordered Agent/session projection,
`Connecting...`/`Reconnecting... (attempt n/m)` presentation, cancellation coverage at all three
wait boundaries, and focused provider/runtime/TUI tests. Retry policy, dependencies, persistence,
versions and release/publication scope are unchanged.

Locked validation passed with an isolated writable `HOME`: affected crates and the complete
`talos-cli` suite passed; strict affected-crate Clippy, formatting and `git diff --check` passed.
The implementation is still Review/Claimed pending exact-head CI, independent review, merge-time
CAS and the deferred live retry-status row in Issue #302. `Completion Commit` remains pending.

## 2026-08-20 Implementation Merge Disposition

PR #323 final head `c984ec483aaba5f6d4d1e96d288cfcb874b0f239` passed exact-head CI
`32333116774` (five jobs, including Windows), independent Agent technical re-review `5351610613`
with shared-account identity limits disclosed, both governance validators and merge-time CAS. It
merged to `main` as `9d5c8a71718b44d424092a45a75d3da0d593547d`; implementation commit
`6efee2b8b257f5c5fde5754d9cd4f211bf9474c7` is an ancestor of that merge.

The remaining natural-person live retry-status row is recorded in Issue #302 comment `5351796088`.
PROVIDER-006 stays Review/Claimed with `Completion Commit: Pending`; the implementation disposition
allows the non-overlapping I198 claim to proceed but does not claim human acceptance or transfer
release authority.
