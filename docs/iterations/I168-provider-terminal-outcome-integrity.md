# Iteration I168: Provider Terminal Outcome Integrity

> Document status: Active
> Published plan date: 2026-07-29
> Planned objective: Stop treating unknown or missing provider terminal signals as normal success, preserve a bounded terminal-cause diagnostic, and show truncation/failure accurately.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: A rebuilt Talos binary distinguishes explicit completion, output-limit truncation, unsupported finish reasons, and terminal-frame-less stream closure in the UI and retained interactive TLOG evidence.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `RUNTIME-003` | None | Ready | ADR-039/042 Accepted; RUNTIME-002, PROVIDER-002, and SESSION-006 Complete | False-success terminal fallbacks are removed and terminal cause becomes visible and reconstructable. |

### Baseline And Inventory

- Starting HEAD: `19d0f2e4074fe3356a911f8a5a21eff7847173b7`.
- Branch: `main`.
- Execution mode: direct `main`; no parallel implementation task exists.
- Active/Review inventory: none before activation.
- I156, I157, I163, I165, I166, and I167: Complete.
- I164: Paused with its superseded baseline retained.
- I158-I162: remain Blocked; ADR-053 remains Proposed.
- Existing Ready reserve work, including PROVIDER-004 and TOOL-023-A/B/C, remains deferred.
- OBS-002 remains Refinement because its broader live pipeline timeline still requires an ADR.
- Priority decision: the maintainer classified silent false-success and missing terminal-cause
  evidence as serious. I168 preempts ADR-053 review/G1 as a bounded P0 correction; it does not
  supersede or modify I158-I162.

### Scope

1. Write deterministic failing fixtures for parser false-success paths.
2. Make unknown finish/stop reasons and missing terminal frames terminal errors in both provider
   adapters.
3. Preserve `MaxTokens` partial text while classifying it visibly and durably as truncation.
4. Persist bounded interactive terminal diagnostics correlated to the current turn/provider
   response, excluded from transcript/model/export surfaces.
5. Prove valid completed tool prefixes remain intact when the following provider continuation
   fails.
6. Drive the result through the canonical session/conversation/TUI path and rebuilt binary.

### Non-Goals

- No heuristic assessment of whether prose “looks incomplete”.
- No automatic continuation, provider failover, new retry behavior, or resumable stream.
- No broader OBS-002 request/header/first-packet/tool-correlation timeline.
- No PROVIDER-004 or TOOL-023 implementation.
- No public breaking API, new dependency, `unsafe`, global event bus, transcript-format migration,
  permission/sandbox change, version change, tag, publish, or release.
- No I158-I162 implementation and no ADR-053 status change.

### Acceptance

- Given partial provider text followed by transport EOF without an explicit terminal frame, when
  the provider task ends, then Talos reports an error rather than `EndTurn`, clears processing,
  excludes the trailing fragment from completed assistant facts, and retains the normalized cause.
- Given an unsupported finish/stop reason, when parsed, then Talos reports that bounded reason and
  never treats it as normal completion.
- Given `MaxTokens`, when the provider stops, then partial text remains visible but truncation is
  explicit in UI and retained diagnostics.
- Given a successful tool result and a failing continuation, when the turn ends, then the tool
  result remains preserved and the provider failure is separately attributable.
- Given explicit normal terminal signals, when the turn ends, then existing success behavior is
  unchanged.
- Given TLOG reopen, compaction, transcript hydration, copy, export, and provider-history rebuild,
  when terminal diagnostics exist, then they remain available as bounded operational evidence but
  never enter conversation facts or model context.

### Required Failing Tests Before Production Changes

- `openai_eof_after_partial_text_is_terminal_error`
- `anthropic_eof_after_partial_text_is_terminal_error`
- `openai_unknown_finish_reason_is_not_end_turn`
- `anthropic_unknown_stop_reason_is_not_end_turn`
- `provider_byte_stream_error_is_terminal_error`
- `max_tokens_is_visible_and_durable_as_truncated`
- `failed_continuation_preserves_completed_tool_prefix_without_trailing_fragment`
- `interactive_tlog_terminal_diagnostic_round_trips_with_turn_correlation`
- `terminal_diagnostic_is_excluded_from_messages_copy_export_and_provider_history`
- `recent_terminal_diagnostic_survives_compaction`
- `explicit_provider_completion_regression_is_unchanged`

### Planned Validation

```bash
cargo test --locked -p talos-provider terminal
cargo test --locked -p talos-provider eof
cargo test --locked -p talos-agent terminal
cargo test --locked -p talos-session diagnostic
cargo test --locked -p talos-conversation terminal
cargo test --locked -p talos-cli terminal
cargo test --locked -p talos-tui terminal
cargo fmt --all -- --check
cargo check --workspace --locked
cargo clippy --workspace --locked -- -D warnings
cargo test --workspace --locked
scripts/validate_project_governance.sh .
git diff --check
cargo build --locked -p talos-cli
```

Record every command, exit code, actual test count, warning count, binary path, and implementation
commit. “Green” alone is not evidence.

### Runtime Evidence

Use deterministic local fixture providers for both compatible protocols and the rebuilt
`target/debug/talos`:

1. explicit normal completion;
2. output followed by `MaxTokens`;
3. output followed by unsupported finish/stop reason;
4. output followed by connection close without terminal signal;
5. successful tool result followed by a continuation that closes without a terminal signal;
6. reopen the generated TLOG and verify the normalized cause while `/copy all` and export remain
   free of diagnostics.

Real paid/provider credentials are not required and must not be introduced.

### Documentation To Update

- `docs/backlog/active/RUNTIME-003-provider-terminal-outcome-integrity.md`
- `docs/backlog/active/OBS-002-turn-pipeline-boundary-observability.md`
- this iteration
- `docs/iterations/README.md`
- `docs/tasks/2026-07-26-v0.6-runtime-productization-program.md`
- `docs/tasks/2026-07-28-four-month-v06-execution-package.md`
- `docs/backlog/PRODUCT-BACKLOG.md`
- `docs/BOARD.md`
- relevant README/reference troubleshooting documentation
- `EVOLUTION.md`

### Risks And Rollback

- Risk: a compatibility gateway intentionally closes after complete data without `[DONE]` or a
  finish reason. Correctness requires treating this as malformed, but the error must explain the
  compatibility requirement instead of hanging.
- Risk: diagnostics accidentally become model-visible or export-visible. Prevent with storage
  round-trip and all projection/filter regression tests before enabling production persistence.
- Risk: valid tool prefixes are lost when the continuation fails. Preserve SESSION-006 semantics
  and test the exact sequence.
- Rollback: revert the I168 implementation commits. Existing explicit terminal signals and timeout
  handling remain the known runnable baseline; do not restore false-success fallback selectively.

### Stop And Escalate

- Any RUNTIME-003 Stop And Escalate condition fires.
- A public breaking API or ADR-042 persistence change is unavoidable.
- A compatibility fixture demonstrates that the proposed explicit-terminal rule rejects a
  documented, supported provider contract that has no alternative signal.
- Three bounded approaches fail or baseline provider/session tests fail without explanation.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-29 | Activation | Inventory at `19d0f2e4` found no Active or Review implementation iteration; I164 remains Paused, I158-I162 remain Blocked, ADR-053 remains Proposed, and existing reserve work remains deferred. Maintainer classified silent provider false-success and lost stop-cause evidence as serious. RUNTIME-003 was refined to Ready, selected into I168, then moved to In Progress. I168 is the sole Active implementation authority. |

## Verification Evidence

- Planning/governance validation: pending.
- Implementation and runtime evidence: pending.

## Completion Evidence

- Completion Commit: pending. I168/RUNTIME-003 must not become Complete without already-existing
  implementation SHA evidence and rebuilt-binary acceptance.

## Variance And Residuals

- OBS-002 remains a separate broader observability Story. I168 does not add a general
  request/tool/header progress event contract.
- The supplied TLOG proves evidence loss and false-success acceptance but cannot recover its
  original provider wire protocol or raw finish reasons. Both current compatible adapters are in
  the required fixture matrix.
- PROVIDER-004 remains a separately owned Ready Story; repeated `call_0` identifiers in the TLOG
  are not treated as the proven cause of all three dangling completions.

## Retrospective

- Pending.
