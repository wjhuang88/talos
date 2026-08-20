# ADR-062: Typed Provider Retry Progress Boundary

**Status**: Accepted when governance PR #321 reaches `main`; Proposed while the PR is open

**Date**: 2026-08-20

**Owners**: PROVIDER-006 / I210

**Acceptance Gate**: independent exact-head architecture review, CI, both governance validators,
merge-time CAS and PR #321 target-branch merge. No implementation authority exists while the PR is
open.

## Context

The built-in OpenAI-compatible and Anthropic adapters already know the configured retry ceiling,
the current retry ordinal and the bounded backoff delay. Those facts are currently emitted only as
structured tracing fields inside `talos-provider`. `LanguageModel::stream_with_tools()` does not
provide progress before response headers arrive, so the Agent and TUI can only retain a static
`Connecting...` state while dispatch retries are occurring.

The UI must show `Connecting...` for the initial dispatch and then truthful
`Reconnecting... (attempt n/m)` state after a retryable provider error or dispatch timeout. It must
not derive `n/m` from elapsed time, error text or a duplicated retry policy. The change also has to
preserve third-party `LanguageModel` implementations and the existing retry, timeout, cancellation
and persistence behavior.

## Decision

1. `talos-core` will define a non-exhaustive, serializable `ProviderProgress` protocol containing
   only bounded, non-secret facts. Its stages are initial/retry dispatch, scheduled backoff and
   first-packet wait. Every stage carries the zero-based retry ordinal and configured retry ceiling;
   scheduled backoff additionally carries the actual bounded delay in milliseconds. Retry ordinal
   `0` means the initial request. Retry ordinals `1..=max_retries` map directly to the existing
   `RetryDecision::Retry` result; this ADR does not reinterpret `max_attempts` or change policy.
2. `LanguageModel` will gain an additive async progress-aware method with a default implementation
   that delegates to the existing `stream_with_tools()` method and emits no progress. Existing
   third-party implementations therefore continue to compile and behave as before. The Agent uses
   the new method; built-in providers override it and send progress on a per-request unbounded
   channel so status reporting cannot block provider dispatch.
3. The Agent wraps typed progress as a new `AgentEvent::ProviderProgress` variant. `AgentEvent`,
   `SessionEvent` and `TurnEventPayload` are already `#[non_exhaustive]`, so the event travels through
   the existing ordered transient `TurnEventPayload::Progress` path without a new persistence
   record, error-message parser or side channel at the runtime boundary.
4. Conversation state maps retry ordinal `0` to the existing `Connecting` phase. A positive ordinal
   maps to a new `Reconnecting { attempt, max_attempts }` phase and renders exactly
   `Reconnecting... (attempt n/m)` in the model-request activity row. The existing
   `Retrying { attempt }` phase remains available as a compatibility variant and is not repurposed.
5. The progress phase remains visible during its reported backoff, next dispatch and first-packet
   wait. The first real content/reasoning/tool event replaces it with the existing phase; success,
   terminal failure and cancellation clear or terminalize it through the existing turn lifecycle.
   Progress events are not transcript messages, are not restored as current activity after resume
   and never contain URLs, headers, credentials, response bodies or provider error strings.
6. Cancellation continues to use the existing per-turn `CancellationToken`. Aborting dispatch,
   backoff or first-packet wait must close the request-local progress forwarder and produce the
   existing durable cancelled terminal outcome. I210 must not introduce an independent retry task
   that can outlive the turn.

## Semver And Migration Contract

- Adding the defaulted `LanguageModel` method and the non-exhaustive core progress/event variants is
  source-compatible for existing provider implementers and consumers that already follow the
  non-exhaustive matching contract.
- Adding `TurnPhase::Reconnecting` is additive but can require a new wildcard arm in downstream
  exhaustive matches. Because Talos is pre-1.0, it may ship only in the next minor release, not a
  patch release. Existing `TurnPhase::Retrying` constructors remain valid.
- Third-party providers are not required to emit progress. They retain the existing static
  `Connecting...` behavior until they explicitly override the progress-aware method.
- I210 changes no crate version, tag or publication state. The release owner must enforce the minor
  version gate when this API is next published.

## Alternatives Rejected

- Parse tracing or terminal error strings: loses typed attempt identity and couples UI behavior to
  diagnostics wording.
- Infer retries from elapsed time: fabricates state and cannot distinguish dispatch, backoff and
  first-packet wait.
- Change the return type of `LanguageModel::stream()`: breaks every provider implementation and is
  unnecessary when a defaulted additive method can carry progress.
- Put retry policy in the Agent or TUI: creates competing retry authorities and can display values
  that differ from the actual provider decision.
- Persist transient progress in session history: resume would present an obsolete wait as current
  state and would expand the durable protocol without user value.

## Validation And Acceptance

- Compile an unchanged out-of-tree-style mock that implements only the current required
  `LanguageModel::stream()` method and prove the default progress path remains usable.
- Deterministic OpenAI-compatible fixtures must cover initial dispatch, retryable response,
  dispatch timeout, bounded backoff, eventual success, exhausted failure and cancellation during
  dispatch/backoff/first-packet wait. Anthropic instrumentation must obey the same protocol.
- Contract tests must prove ordinal bounds, event order, absence of error text/secrets and no change
  to retry decisions or configured delays.
- Runtime/conversation/TUI tests must prove `Connecting...` then
  `Reconnecting... (attempt n/m)`, replacement on first meaningful stream event and clearing on all
  terminal outcomes.
- Locked focused/workspace validation, exact-head CI, independent technical review, merge-time CAS
  and the Issue #302 natural-person live retry-status row are required by I210/I211.

## Reversal

If the defaulted method cannot preserve third-party provider compatibility or the ordered event
path cannot distinguish retry progress without changing retry policy, remove the new method and
progress projection before release. Built-in providers then retain the current static
`Connecting...` behavior; existing configuration, sessions and retry semantics remain readable and
unchanged because I210 adds no durable state.
