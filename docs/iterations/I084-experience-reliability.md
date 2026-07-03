# Iteration I084: Experience Reliability — Thinking, Timeout, Retry, And Status

> Document status: Complete
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
| 2026-07-03 | UX100 | ADR-034 drafted: per-model `reasoning` options in `ModelConfig`, provider-specific mapping (Anthropic thinking block, OpenAI reasoning_effort + max_completion_tokens, OpenAI-compatible reasoning_content stream), existing `AgentEvent::ThinkingDelta` variant, transient-only persistence, existing TUI preview, optional reasoning token count in `Usage`. Awaiting maintainer acceptance before UX101 implementation. |
| 2026-07-03 | UX100 | Cross-project reasoning token usage research completed (OpenCode, Codex, Claude Code, Cline, Aider, Continue, omp.sh, Pi). Findings recorded in `docs/reference/REFERENCE-PROJECTS.md` section 20. ADR-034 cost model refined: `reasoning_tokens` as informational subset of `output_tokens` (Cline/omp.sh precedent), priced at normal output rate. omp.sh added to reference project repositories. Pre-existing bugs (Anthropic hardcoded `max_tokens`, cache/reasoning tokens not surfaced in TUI) folded into UX101-UX105 execution scope. |
| 2026-07-03 | UX100 | Thinking-in-context-history research completed. Major finding: ADR-034 "transient only, do not persist" was insufficient. Anthropic requires thinking blocks in request history for tool conversations; omp.sh/Pi/Cline/Claude Code all replay thinking; local servers need it for KV cache stability. ADR-034 persistence section revised: `reasoning: Option<String>` on durable `Message` for request-history replay; display stays transient; `replay_reasoning` config flag (default true). OpenCode outlier pattern (strip thinking from requests) explicitly rejected. REFERENCE-PROJECTS.md section 20 expanded with full cross-project evidence. |
| 2026-07-03 | UX100 | Architecture review conducted (Oracle consultation + codebase fact verification + provider ground-truth verification against official docs/SDKs). Verdicts: Q1 persistence data model REJECT (`Option<String>` cannot carry Anthropic Required `signature` or `redacted_thinking` data — internally contradictory with the ADR's own replay guardrail), Q6 scope REJECT (signature storage not deferrable while Anthropic replay is in scope), Q2/Q3/Q4/Q5/Q7 RISK (no cross-provider replay policy for `/model`; no compaction boundary; JSONL `#[serde(default)]` premise factually wrong — JSONL flattens messages and never serializes `Message` structs; export boundary incidental not intentional; Gemini `thoughtSignature` and DeepSeek native API missing from research). New ground truth: signatures are model-bound not key-bound; DeepSeek native requires `reasoning_content` on tool-call turns (400 without) but ignores it otherwise; official OpenAI Chat Completions never streams `reasoning_content`. |
| 2026-07-03 | UX100 | ADR-034 revised (v3) per review. Persistence redesigned: structured `ReasoningBlock` (`Thinking { text, signature }` / `Redacted { data }` / `Plain { text }`) wrapped in origin-stamped `AssistantReasoning { provider, model, blocks }`; new `AgentEvent::ReasoningComplete` durable-payload event (display keeps `ThinkingDelta`); JSONL round-trip via `SessionMetadata.reasoning` implemented symmetrically inside talos-session (`is_empty()` trap documented); origin-gated replay enforced in talos-agent on request copies; Anthropic trailing-tool_use degradation guardrail (omit `thinking` param instead of guaranteed 400); reasoning excluded from `/copy`/`/export` by design; compaction minimal boundary now, age-based trimming deferred to MEM-007; per-model `replay` flag with config-load consequence warning. REFERENCE-PROJECTS.md §20 corrected (model-bound signatures, DeepSeek/Gemini/OpenAI official evidence subsection) and decisions/README.md entry 34 synced. Scope implication for UX101-UX102: structured storage and persistence round-trip are in-scope for the first slice (not deferrable); fallback if the slice must shrink is cutting the entire Anthropic replay path, never unsigned persistence. |

## Verification Evidence

- UX100 complete: ADR-034 v3 accepted (2026-07-03) after architecture review. Cross-project research recorded in REFERENCE-PROJECTS.md §20. Owner docs (MODEL-003 ADR gate, UX-001 acceptance criterion) synced.
- UX101 complete: Reasoning implementation across 4 phases (data model, Anthropic path, OpenAI path, agent replay gate). 20+ tests added. Committed: 2ddae35..6e1dea4.
- UX102 complete: Config validation (capability gating + replay-disable warnings), config.reference.toml documentation. Committed: 29a7750..2d9e8a9.
- UX103 complete: Provider first-packet and stream-idle timeout detection with structured errors. 6 tests added. Committed: db9f0a2..eeafc1a.
- UX104 complete: Retry classifier with exponential backoff and jitter, replacing ad-hoc retry logic. 8 tests added. Committed: 9362e6d..9fe4b83.
- UX105 complete: TurnPhase status states (Connecting/Thinking/Generating/TimedOut/Failed/Cancelled) in conversation engine and TUI. 6 tests added. Committed: 225a1ab..0b8fbbb.
- UX106 complete: Owner docs synced, validation run. 1497 workspace tests pass. Clippy clean. Fmt clean. Governance validation passed (0 warnings).
- Full validation: `cargo test --workspace` = 1497 tests pass; `cargo clippy --workspace -- -D warnings` = clean; `cargo fmt --all -- --check` = clean; `scripts/validate_project_governance.sh .` = 0 warnings.

## Variance And Residuals

- Age-based reasoning compaction deferred to MEM-007 (active context compression).
- Richer thinking TUI (collapsible section, scrollable panel) deferred to a separate design decision.
- Per-gateway compatibility overrides (requires_reasoning_replay / forbids_reasoning_replay) deferred until evidence demands.
- Gemini native adapter (thoughtSignature) deferred to a future Gemini protocol decision.
- OpenAI Responses API (encrypted reasoning items) deferred to a separate ADR when Responses support lands.
- Provider retry status events (Retrying phase in TUI) not yet emitted — retry happens inside send_request before streaming starts. Future enhancement could surface retry attempts.
- StatusSnapshot.usage is last-turn only (pre-existing latent issue); cumulative usage via TokenEstimator is not plumbed to the TUI.

## Retrospective

- Completed as the release-facing UX reliability closeout for 2026-07-03.
- The high-risk reasoning persistence decision was correctly forced through ADR-034 before code:
  the initial transient-only design was rejected after provider-ground-truth review, and the final
  implementation persisted structured, origin-gated reasoning blocks instead.
- Timeout/retry/status work landed as a coherent user-facing slice rather than isolated provider
  internals; this is the right shape for release readiness because users see bounded progress and
  failure states.
- Residuals are explicitly non-release-blocking follow-ups: richer thinking UI, age-based reasoning
  compaction, gateway-specific replay overrides, Gemini native support, OpenAI Responses API, retry
  attempt status events, and cumulative TUI usage plumbing.
