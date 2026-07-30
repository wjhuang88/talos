# RUNTIME-003: Provider Terminal Outcome Integrity

| Field | Value |
| --- | --- |
| Story ID | RUNTIME-003 |
| Type | Technical Story (P0 reliability bug fix) |
| Priority | P0 |
| Status | Complete — I168 accepted (2026-07-30) |
| Source | Maintainer-provided TLOG analysis on 2026-07-29: three assistant responses ended as successful turns after dangling pre-tool text, without durable terminal-cause evidence |
| Depends On | ADR-039; ADR-042; RUNTIME-002/PROVIDER-002 timeout foundations; SESSION-006 completed-prefix persistence |
| Selected Iteration | I168 (Complete) |

## Identity / Goal / Value

An interactive Talos user and a post-incident maintainer must be able to distinguish a normal
provider completion from output-limit truncation, an unknown provider finish reason, a stream that
closed without an explicit terminal frame, timeout, cancellation, or another provider error.

Today both OpenAI-compatible and Anthropic-compatible stream parsers normalize unknown finish
reasons and terminal-frame-less EOF to `StopReason::EndTurn`. The agent then commits any text-only
response as a successful turn, including text such as `现在看……：` that clearly precedes an intended
tool call. The interactive TLOG stores the assistant message but not the terminal event or mapped
stop reason, so the original cause cannot be reconstructed.

The goal is not to infer model intent from punctuation. The goal is to preserve and expose the
actual protocol outcome, and never manufacture a normal-success boundary when the transport did
not provide one.

## Confirmed Evidence

The supplied TLOG retained 51 recent entries after compaction and contained three independently
observable dangling success records:

1. `添加 import 和去掉 scoped：` followed by no tool call; the user resumed 6m10s later.
2. `让我启动它：` followed by no tool call; the next user message arrived about 1h15m later.
3. A successful `read` result followed by
   `现在看 app1 当前的前端 API 调用和 store 持久化逻辑，确认缺口：`, then no tool call; the user sent
   `继续` 4m32s later.

Every adjacent tool result was `__OK__`. No persisted `AgentEvent::TurnEnd`,
`AgentEvent::Error`, raw finish reason, timeout, or cancellation record existed. The entries could
only reach the ordinary completed-turn persistence path after the agent observed a `TurnEnd`;
failed-turn persistence excludes a trailing partial assistant fragment.

The exact provider cause for those three responses is unrecoverable from the current TLOG. It may
have been an explicit normal stop, `max_tokens`, an unknown finish reason normalized to `EndTurn`,
or a transport EOF normalized to `EndTurn`. This Story must make future cases distinguishable.

## Scope

### Provider terminal integrity

- OpenAI-compatible `[DONE]` and known finish reasons retain their current protocol meaning.
- Anthropic-compatible known stop reasons retain their current protocol meaning.
- A stream EOF without the protocol's explicit terminal signal emits a terminal provider error;
  it must not synthesize `EndTurn`.
- An unknown OpenAI `finish_reason` or Anthropic `stop_reason` emits a bounded, redacted terminal
  provider error that names the unsupported reason; it must not synthesize `EndTurn`.
- `MaxTokens` remains a distinct existing `StopReason`; generated text is preserved, but the
  outcome is visibly and durably classified as truncated rather than indistinguishable from a
  normal completion.
- Byte-stream decode failures and transport read errors must become observable terminal errors
  rather than silently falling through to EOF success.

### Agent/session classification

- The agent must not return whole-turn success for parser/transport terminal errors.
- A valid completed tool-call/result prefix remains recoverable under SESSION-006 when a following
  provider continuation fails.
- A trailing half-streamed assistant fragment on an error path is not committed as a completed
  assistant fact.
- Use the existing ADR-039 ordered session seam. Do not add a global bus or a second lifecycle
  authority.

### Durable diagnostic evidence

- Persist one bounded, redacted terminal diagnostic for each provider-response boundary in the
  ordinary interactive Session/TLOG path, correlated with the enclosing `turn_id` and response
  ordinal.
- Reuse the existing system-event/session-record lane or an additive internal record. Do not add a
  breaking public enum variant.
- Terminal diagnostics are excluded from `read_messages`, provider request history, transcript
  projection, copy, and export. They are operational evidence, not conversation facts.
- The record contains only normalized outcome, explicit-versus-error source, bounded unknown
  reason/error category, provider/model identity already permitted by Session metadata, and
  correlation identifiers. It must not contain credentials, headers, request/response bodies,
  hidden reasoning, or raw tool output.
- Reopening the TLOG and retaining recent entries through current compaction must preserve enough
  diagnostic evidence to identify the terminal cause for a retained assistant entry.
- ADR-042 durable embedded transcripts retain their existing successful-turn atomicity and
  filtering. If sharing this diagnostic lane with embedded durable sessions would change that
  contract, keep the first implementation limited to interactive Session persistence and record a
  separate ADR-gated residual.

### User-visible outcome

- Stream EOF, unsupported finish reason, timeout, and provider errors end processing and render a
  specific bounded failure explanation.
- `MaxTokens` preserves visible partial text and renders a specific truncation explanation.
- Normal explicit completion remains visually quiet.
- The diagnostic wording distinguishes provider/transport completion from tool execution; it must
  not claim that a tool failed when the tool result was already successful.

## Exclusions

- No punctuation, trailing-colon, language-model-intent, or “looks incomplete” heuristic.
- No automatic continuation, retry beyond existing provider retry policy, provider failover, or
  resumable streaming.
- No new public event bus, observer API, renderer, dependency, `unsafe`, or protocol plugin.
- No raw provider payload persistence and no provider-context growth.
- No change to tool execution, permission, sandbox, session export, transcript facts, or
  completion of PROVIDER-004.
- The broader per-boundary live timeline
  (`tool completed -> result accepted -> dispatch -> headers -> first packet`) remains OBS-002 and
  retains its ADR gate. RUNTIME-003 only fixes terminal-outcome integrity and post-incident
  evidence.

## Decision Links And Constraints

- ADR-039: session-level `TurnEventPayload::Completed` remains the authoritative user-turn
  lifecycle; provider `TurnEnd` is a response boundary and diagnostic input only.
- ADR-042: failed embedded durable turns remain absent from model-visible durable transcripts.
- SESSION-006: valid completed tool exchanges may persist after a later provider failure; no
  fabricated tool result or half-streamed assistant completion.
- Public APIs are semver-bound. The implementation must use additive/non-breaking APIs. If a
  breaking `StopReason`, `TurnCompletionStatus`, or Session schema change appears necessary, stop,
  draft an ADR and migration plan, and request maintainer approval.

## Uncertainty And Validation Path

- The supplied TLOG cannot prove which wire protocol the custom `baosight/glm-5.2-global`
  provider used. Both OpenAI-compatible and Anthropic-compatible adapters therefore require the
  same terminal-integrity matrix.
- The supplied TLOG cannot recover the original raw finish reason. Deterministic fixture streams
  must reproduce explicit completion, unknown reason, missing terminal frame, transport error,
  and `MaxTokens`.
- Before production changes, write failing fixtures that demonstrate the current false-success
  behavior.

## State / Status Owners

- Story status and acceptance: this file.
- Iteration execution and evidence: `docs/iterations/I168-provider-terminal-outcome-integrity.md`.
- Program priority/disposition: v0.6 program owner and four-month execution package.
- Current operating view: `docs/BOARD.md`.
- Broader boundary timeline: OBS-002; this Story does not silently complete it.

## User-Facing Documentation

- Add a troubleshooting/reference section explaining the visible terminal outcomes and what a
  retained partial answer means.
- If an existing session inspection command can expose the diagnostic without scope expansion,
  document it. Otherwise the TLOG reopen/integration test is the mandatory post-incident evidence
  and an inspection command remains a separately owned residual.

## Required Reads

- `AGENTS.md`
- `docs/sop/ITERATION-WORKFLOW.md`
- `docs/sop/TESTING.md`
- `docs/decisions/039-runtime-event-semantic-single-flow.md`
- `docs/decisions/042-embedded-durable-runtime-session-boundary.md`
- `docs/backlog/active/RUNTIME-002-turn-health-and-stuck-processing.md`
- `docs/backlog/active/PROVIDER-002-response-reliability-timeout-retry.md`
- `docs/backlog/active/SESSION-006-session-error-path-persistence.md`
- `docs/backlog/active/OBS-002-turn-pipeline-boundary-observability.md`
- `crates/talos-core/src/message.rs`
- `crates/talos-core/src/session.rs`
- `crates/talos-provider/src/openai_sse.rs`
- `crates/talos-provider/src/anthropic_stream.rs`
- `crates/talos-agent/src/lib.rs`
- `crates/talos-agent/src/session/turn.rs`
- `crates/talos-conversation/src/engine.rs`
- `crates/talos-session/src/jsonl.rs`
- `crates/talos-session/src/durable.rs`
- `crates/talos-session/src/types.rs`
- `crates/talos-cli/src/tui_bridge.rs`
- `crates/talos-tui/src/app.rs`

## Acceptance For Behavior

- Given an OpenAI-compatible stream that emits partial text and closes without `[DONE]` or
  `finish_reason`, when Talos consumes it, then the turn ends as a visible provider/stream error,
  the partial text is not committed as a completed assistant message, and the TLOG diagnostic
  identifies missing explicit termination.
- Given an Anthropic-compatible stream that emits partial text and closes without a terminal
  message delta, when Talos consumes it, then the same error and persistence invariants hold.
- Given either adapter returns an unknown finish/stop reason, when Talos consumes it, then the
  bounded reason is observable and is never mapped to normal `EndTurn`.
- Given `MaxTokens`, when partial text exists, then the text remains visible and recoverable while
  the UI and TLOG classify it as truncated.
- Given a successful tool result followed by a provider continuation that terminates abnormally,
  when the turn ends, then the tool result remains preserved, the continuation failure is
  separately visible, and no dangling assistant preamble is marked complete.
- Given explicit normal completion, when the turn ends, then current success behavior and
  transcript output remain unchanged.
- Given a recent terminal diagnostic and a TLOG reopen/compaction cycle, when a maintainer inspects
  the retained turn, then its normalized terminal cause remains attributable without entering
  model context or exported conversation text.

## Acceptance For Technical Work

- [x] Failing-before-fix fixtures cover OpenAI EOF after partial text, Anthropic EOF after partial
      text, unknown finish/stop reasons, byte-stream error, and known normal terminal events.
- [x] Agent/session tests cover false-success rejection, `MaxTokens` partial classification, valid
      completed-prefix recovery, and trailing-fragment exclusion.
- [x] Session tests prove terminal-diagnostic TLOG round trip, turn/response correlation,
      compaction retention, redaction, and exclusion from model-visible messages/copy/export.
- [x] Conversation/CLI/TUI integration tests drive the canonical session bridge and prove
      processing clears with the correct visible outcome.
- [x] Existing explicit `[DONE]`, `end_turn`, `tool_use`, timeout, cancellation, retry, tool-call,
      SESSION-006, ADR-039 ordering, and ADR-042 durable tests remain green.
- [x] Source scan proves unknown reason and terminal-frame-less EOF no longer map to
      `StopReason::EndTurn`.
- [x] `cargo fmt --all -- --check` exits 0.
- [x] `cargo check --workspace --locked` exits 0 with 0 warnings.
- [x] `cargo clippy --workspace --locked -- -D warnings` exits 0 with 0 warnings.
- [x] `cargo test --workspace --locked` exits 0; actual test counts are recorded.
- [x] `scripts/validate_project_governance.sh .` exits 0 with 0 warnings.
- [x] A rebuilt `target/debug/talos` fixture/manual walkthrough demonstrates normal completion,
      `MaxTokens`, terminal-frame-less EOF, and tool-success-then-continuation-failure.
- [x] Story, I168, iteration index, program, execution package, Product Backlog, Board, user docs,
      and residuals are synchronized.

## Stop And Escalate

- A correct fix requires a breaking public API or persisted-format migration.
- Provider compatibility requires preserving terminal-frame-less EOF as normal success.
- Terminal diagnostics cannot be excluded from model context, transcript, copy, and export.
- ADR-042 successful-turn atomicity or SESSION-006 completed-prefix semantics would change.
- A new dependency, `unsafe`, global event bus, permission change, or raw provider payload
  persistence is required.
- Baseline tests reveal an unexplained provider/session regression.

## Completion Evidence

- Completion Commit: `86262d0290d821b7e3518a0e6371f0b2d3185e95`.
- Provider/session/TUI foundation merged through PR #63 at `b5fcaaf3`; print/headless MaxTokens projection landed at `dda2170f`; deterministic fixture scripts landed at `86262d02`.
- Red-before-green evidence: workflow run `30551626267`, exit 101 before print-mode production implementation.
- Initial closeout evidence: workflow `30552762936` passed the first matrix. PR #67 review then required explicit known-terminal policies and merged-output ordering.
- Final review evidence: workflow `30558757429`, job `90925992796`, passed the expanded parser/session/CLI/TUI/workspace and rebuilt-binary matrix; clean-HEAD CI `30558599777`, job `90926266628`, passed Release preflight with 2681 workspace tests and zero failures.
- Rebuilt `target/debug/talos` passed normal completion, MaxTokens, OpenAI `content_filter` and legacy `function_call`, Anthropic `stop_sequence`, `pause_turn`, and `refusal`, synthetic unknown reasons, EOF, invalid UTF-8, transport failure, and tool continuation. The fixture asserts empty normal stderr, exactly one truncation warning, absence of stale labels, and merged-fd line ordering.
- Terminal diagnostics remain bounded operational evidence and are excluded from model context, transcript, copy, and export.
- OBS-002 remains Refinement and is not completed by this Story.
- Review correction `c570991b` defines the known protocol policy matrix; clean Completion Commit `86262d02` removes transient execution artifacts. Revalidation workflow `30558757429` passed parser/session/CLI/TUI/workspace gates and the exact/negative/merged-fd rebuilt-binary fixture.

### Review-Correction Policy

- Known terminal values are never described as unknown.
- OpenAI `content_filter` and deprecated `function_call` are bounded provider-error policies.
- Anthropic `stop_sequence` maps to explicit normal completion; `pause_turn` and `refusal` are bounded provider-error policies because automatic pause continuation is outside this Story.
- Only unrecognized values such as `fixture_unknown_reason` use `UnsupportedReason`.
- MaxTokens partial output is newline-terminated and flushed before the single stderr warning, including when descriptors are merged.
