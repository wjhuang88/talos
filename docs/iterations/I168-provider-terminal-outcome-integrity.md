# Iteration I168: Provider Terminal Outcome Integrity

> Document status: Complete (2026-07-30)
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
| 2026-07-29 | Maintainer pause | The maintainer requested that I168 stop before implementation while they handle the underlying repair context. No implementation work is authorized while paused, no replacement iteration is activated, and resumption requires an explicit maintainer instruction followed by owner/Board/program synchronization. The published objective, scope, acceptance criteria, and P0 priority remain unchanged. |
| 2026-07-30 | Maintainer resumption | The maintainer explicitly requested I168 activation after the I157 stale-snapshot correction completed at `5aac6756`. Inventory confirms I157/MODEL-010 and I167 are Complete, I164 remains Paused, I158-I162 remain Blocked, ADR-053 remains Proposed, and no competing Active/Review implementation iteration exists. I168/RUNTIME-003 became the sole Active implementation authority. |
| 2026-07-30 | Branch variance | After implementation PR #63 merged at `b5fcaaf3`, the maintainer authorized branch/PR closeout. PR #67 used `agent/i168-terminal-closeout`; no direct `main` mutation, force-push, tag, publish, release, or deployment occurred. |
| 2026-07-30 | Completion | Red-first print-mode proof failed with exit 101 before `dda2170f`; deterministic fixtures and the complete locked matrix passed at validation harness commit `62ae098d`. Completion Commit is the already-existing implementation/fixture commit `86262d02`. |
| 2026-07-30 | Review rejection | Maintainer review rejected the first closeout because standard `content_filter` / `pause_turn` values were mislabeled unknown, legacy/known stop policies were incomplete, and merged stdout/stderr could concatenate the MaxTokens warning. I168 returned to corrective Review. |
| 2026-07-30 | Review correction accepted for re-review | `c570991b` added explicit known policies and consumer tests; `86262d02` removed execution artifacts and is the Completion Commit. Workflow `30558757429`, job `90925992796`, passed the full review matrix and rebuilt-binary fixture. |

## Verification Evidence

- Red-first print/headless proof: workflow run `30551626267` checked out `b5fcaaf3` and the new MaxTokens projection assertion failed before production code with exit 101 (`E0425`: missing `terminal_notice_for_stop_reason`).
- Implementation chain: PR #63 merge `b5fcaaf3`; print/headless truncation projection `dda2170f`; deterministic two-protocol fixture packet `86262d02`.
- Final validation: workflow run `30552762936`, job `90905349923`, validation harness commit `62ae098d`, `OVERALL_EXIT=0`. Standard PR CI run `30552762362` also passed.
- Runtime fixture command: `scripts/verify_i168_provider_terminal.sh target/debug/talos`. Binary path: `target/debug/talos`. The harness commit differs from Completion Commit only by the temporary validation workflow; production and fixture content are owned by `86262d02`.
- Warning accounting: commands that build `talos-config` emitted one informational build-script line (`models.toml compressed`); Rust check and Clippy produced no code warning and `clippy -D warnings` exited 0.

| Validation command | Exit | Passed | Failed | Ignored | Warning lines |
|---|---:|---:|---:|---:|---:|
| `cargo test --locked -p talos-provider terminal` | 0 | 13 | 0 | 0 | 1 |
| `cargo test --locked -p talos-provider eof` | 0 | 2 | 0 | 0 | 1 |
| `cargo test --locked -p talos-agent terminal` | 0 | 2 | 0 | 0 | 0 |
| `cargo test --locked -p talos-session diagnostic` | 0 | 4 | 0 | 0 | 0 |
| `cargo test --locked -p talos-conversation terminal` | 0 | 2 | 0 | 0 | 0 |
| `cargo test --locked -p talos-cli terminal` | 0 | 7 | 0 | 0 | 1 |
| `cargo test --locked -p talos-tui terminal` | 0 | 29 | 0 | 0 | 0 |
| `cargo fmt --all -- --check` | 0 | 0 | 0 | 0 | 0 |
| `cargo check --workspace --locked` | 0 | 0 | 0 | 0 | 1 |
| `cargo clippy --workspace --locked -- -D warnings` | 0 | 0 | 0 | 0 | 1 |
| `cargo test --workspace --locked` | 0 | 2681 | 0 | 0 | 1 |
| `scripts/validate_project_governance.sh .` | 0 | 0 | 0 | 0 | 0 |
| `git diff --check` | 0 | 0 | 0 | 0 | 0 |
| `cargo build --locked -p talos-cli` | 0 | 0 | 0 | 0 | 1 |
| provider terminal source scan | 0 | 0 | 0 | 0 | 0 |
| rebuilt-binary two-protocol fixture | 0 | 0 | 0 | 0 | 0 |

- Reviewed fixture outcomes: OpenAI normal and MaxTokens exit 0; `content_filter`, legacy `function_call`, synthetic unknown, EOF, invalid UTF-8, transport failure, and continuation EOF exit 1 with distinct bounded causes. Anthropic normal, `stop_sequence`, and MaxTokens exit 0; `pause_turn`, `refusal`, synthetic unknown, EOF, invalid UTF-8, transport failure, and continuation EOF exit 1. Normal stderr is empty, MaxTokens emits exactly one warning, error paths exclude stale truncation text, merged descriptors preserve `partial\nwarning\n` ordering, and both continuation requests contain the completed tool result.
- Session tests prove TLOG round trip, compaction retention, turn/response correlation, bounded redaction, and exclusion from messages, provider history, transcript, copy, and export.

## Completion Evidence

- Completion Commit: `86262d0290d821b7e3518a0e6371f0b2d3185e95`. This clean implementation state existed before final review-evidence synchronization and contains corrected implementation `c570991b` as an ancestor.
- Rebuilt-binary review acceptance: PASS in workflow `30558757429`, job `90925992796`. The expanded known-policy/synthetic-unknown matrix, exact and negative stdout/stderr assertions, merged-fd ordering, and tool-continuation prefix checks all passed.
- Final clean-HEAD acceptance: CI run `30558599777`, rerun job `90926266628`, checked the PR merge containing `86262d02`; Release preflight passed with 2681 workspace tests and zero failures. Windows fixture job `90926267367` passed.
- Governance acceptance: owner and derived documents synchronized; I164 remains Paused, I158-I162 remain Blocked, ADR-053 remains Proposed, and OBS-002 remains Refinement.
- Review correction evidence: OpenAI `content_filter` and deprecated `function_call` now have explicit bounded non-success policies; Anthropic `stop_sequence` is explicit normal completion while `pause_turn` and `refusal` are explicit bounded non-success policies; only `fixture_unknown_reason` uses the unknown path. Merged-fd fixtures prove partial output and warning/error are line-separated, normal paths are quiet, and no stale truncation warning appears on errors.

## Variance And Residuals

- The 2026-07-29 pause is closed by explicit maintainer resumption on 2026-07-30. It remains
  historical scheduling evidence and did not change the published objective or acceptance.
- OBS-002 remains a separate broader observability Story. I168 does not add a general
  request/tool/header progress event contract.
- The supplied TLOG proves evidence loss and false-success acceptance but cannot recover its
  original provider wire protocol or raw finish reasons. Both current compatible adapters are in
  the required fixture matrix.
- PROVIDER-004 remains a separately owned Ready Story; repeated `call_0` identifiers in the TLOG
  are not treated as the proven cause of all three dangling completions.

## Retrospective

- The provider adapters must never infer normal success from transport exhaustion; explicit protocol terminal evidence is the authority.
- A shared conversation projection test did not prove the separate print-mode consumer. Behavior-facing terminal outcomes require fixture coverage through every product consumer, including headless output.
- Completion documents must be part of the implementation sequence rather than deferred until after merge; PR #63 was code-complete but not governance-complete.
- Deterministic local SSE fixtures provide stronger, credential-free evidence than provider-paid smoke tests and are now retained under `scripts/fixtures/`.

## Review Correction Evidence

- PR #67 review required known protocol terminal values to have explicit policies rather than being used as unknown fixtures.
- OpenAI `content_filter` is a known filtered error and deprecated `function_call` is a known legacy error with `tool_calls` migration guidance. Anthropic `stop_sequence` is normal completion; `pause_turn` and `refusal` are known bounded non-success outcomes. Only `fixture_unknown_reason` uses `UnsupportedReason`.
- Red evidence recorded parser exit 101 and rebuilt fixture exit 1 before correction, including the rejected merged shape `fixture partialWarning...`.
- Corrected implementation `c570991b` plus cleanup `86262d02` passed workflow `30558757429`; standard clean-HEAD Release preflight and Windows fixture passed in run `30558599777`.
- Superseded initial Completion Commit: `2eac5b0523f6d8006318456b631c72cdb5bf9bed`; PR review replaced it with clean Completion Commit `86262d0290d821b7e3518a0e6371f0b2d3185e95`.
