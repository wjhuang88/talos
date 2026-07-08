# Iteration I102: Provider Runtime Reliability Gate

> Document status: Planned
> Published plan date: 2026-07-07
> Planned objective: Execute Month 1 of the 2026-07-07 four-month developer operating plan by
> closing provider/tool-use stuck-processing risks before additional trial-facing work.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: malformed provider/tool-use streams become complete tool calls or visible
> terminal errors, with runtime evidence proving no silent processing tail.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| D100 | Developer operating plan | Planned | Current owner-doc inventory | Start-gate inventory and regression baseline are recorded. |
| D101 | PROVIDER-002/RUNTIME-002 | Planned | D100 | OpenAI-compatible SSE fixture matrix covers known malformed tool-call paths. |
| D102 | RUNTIME-002 | Planned | D101 | Agent turn-loop invariants reject malformed provider event sequences. |
| D103 | RUNTIME-002/TUI-028 | Planned | D102 | Runtime/TUI status distinguishes provider wait, tool wait, timeout, failed, cancelled. |
| D104 | Developer operating plan | Planned | D100-D103 | Month-1 closeout evidence and residuals are synchronized. |

### Scope

- Add deterministic provider fixture tests for OpenAI-compatible streaming edge cases.
- Add or verify agent invariants for malformed `ToolUse` sequences.
- Use existing conversation/TUI status plumbing for terminal failure visibility.
- Persist or document enough redacted evidence to debug malformed tool-use incidents.

### Non-Goals

- No provider schema redesign.
- No new credential flow.
- No permission-policy change.
- No background watchdog unless deterministic state transitions are proven insufficient and a
  separate review approves it.

### Acceptance

- Given a provider streams split or incomplete tool-call metadata, when parsing finishes, then Talos
  either emits complete `ToolCall` events or a terminal provider error.
- Given the agent receives `ToolUse` with zero or duplicate collected tool calls, when the turn loop
  processes it, then it fails explicitly instead of waiting forever.
- Given a provider/tool failure occurs after a turn starts, when the TUI receives status updates,
  then processing ends with a visible failed/timed-out/cancelled state.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo test -p talos-provider openai::tests::parse_sse_stream`
- `cargo test -p talos-agent tool_use`
- `cargo test -p talos-cli conversation_loop`
- `cargo test -p talos-tui processing`
- `cargo check --workspace`
- `cargo test --workspace` at closeout
- `cargo clippy --workspace -- -D warnings` at closeout
- `scripts/validate_project_governance.sh .`

### Documentation To Update

- `docs/tasks/2026-07-07-four-month-developer-operating-plan.md`
- `docs/backlog/active/RUNTIME-002-turn-health-and-stuck-processing.md`
- `docs/backlog/active/PROVIDER-002-response-reliability-timeout-retry.md`
- `docs/BOARD.md` after owner docs

### Risks And Rollback

- Risk: fixture fixes accidentally reject valid text-based tool-call behavior.
- Rollback: keep native streaming invariants distinct from text-based tool-call parsing and retain
  regression tests for valid multi-tool turns.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-07 | Planning | Created as Month 1 shell for the four-month developer operating plan. |
| 2026-07-07 | D100 Start-gate | Repo surveyed clean at commit `96dc2fc` (`fix(tools): skip rg test when ripgrep is not installed (#RUNTIME-002) [model:deepseek-v4-flash-free]`). Branch `main`, no working-tree changes. Owner docs (`I102`, `I103`, `I104`, `I105`, `RUNTIME-002`, `PROVIDER-002`, `AGENTS.md`, `BOARD.md`, `PRODUCT-BACKLOG.md`) read before D100 ran. Four-month developer operating plan (`2026-07-07-four-month-developer-operating-plan.md`) and handoff (`2026-07-07-programmer-handoff-four-month-developer-operating-plan.md`) exist and are consistent. D101/D102/D103 dependencies satisfied: `RUNTIME-002` is Complete (FS04 MaxTokens fix + integration tests), `PROVIDER-002` is Complete (UX103-UX105), and prior FP1-FP2 (`bf79b39`, `c26b79a`) already shipped SSE `[DONE]`-after-tool-call handling + duplicate-id guard. Hard boundaries re-confirmed: no tag/push/publish, no permission/sandbox/credential/storage-default change, no runtime catalog.db resurrection, no new dependency, every behavior change must carry runtime evidence. Baseline verification snapshot recorded in `## Verification Evidence` and the only residual (`bash_tool.rs` format drift) recorded in `## Variance And Residuals`. |
| 2026-07-07 | D101 Fixtures | Extended the OpenAI-compatible SSE fixture matrix in `crates/talos-provider/src/openai.rs::tests` with six new `parse_sse_stream_*` cases that lock already-implemented parser paths not covered by FP1-FP2: `finish_reason="length" → StopReason::MaxTokens`, role-only first chunk consumed without spurious emit, SSE `: keepalive` / `retry:` / `data: ` empty passthrough, mixed content + tool_calls in one delta, and multi-byte UTF-8 round-trip. No production parser change was needed — these are deterministic regression guards. D102 must now extend the agent-layer invariant (no-silent-stuck-on-`ToolUse`-without-tool-calls) so any future malformed path the fixtures do not yet exercise is still bounded. |

## Verification Evidence

- `git status --short`: clean (empty output).
- `git rev-parse HEAD`: `96dc2fcaa66d78be1f8a293d8ccb4e3d46cacb23`.
- `git log --oneline -3`: `96dc2fc fix(tools): skip rg test when ripgrep is not installed (#RUNTIME-002)` → `b05f71e docs(workspace): add four-month developer operating plan (#D100)` → `27626d9 chore(tools): replace iter().any() with contains() in clippy fix (#RUNTIME-002)`.
- `cargo check --workspace`: passed (exit 0). 19 crates checked (`talos-core`, `talos-agent`, `talos-cli`, `talos-runtime`, `talos-tools`, plus all dependents).
- `cargo test --workspace`: passed, 1778 passed / 0 failed / 0 ignored across 61 active test binaries; doctests embedded (`talos-provider` x2, `talos-sandbox` x2, `talos-skill` x2, `talos-tui` x2, `talos-permission` x1) all green.
- `cargo clippy --workspace -- -D warnings`: passed (exit 0).
- `scripts/validate_project_governance.sh .`: passed, 0 governance warnings; `manifest.yaml` profile still `high-risk`, `status` still `conformant`.
- `cargo fmt --all -- --check`: reports a single pre-existing diff in `crates/talos-tools/src/bash_tool.rs:583` (joined-line variant of `args.first().copied() == Some("fmt") && args.contains(&"--check") && exit_code == 1` introduced by `27626d9`). Existing test suite stays green because rustfmt is presentation-only, but the diff exits non-zero (exit 1) — see `## Variance And Residuals` for ownership.
- Owner-doc cross-check: `I102` references `RUNTIME-002` and `PROVIDER-002` as `Planned` parents under `Selected Stories`, but the active backlog owner docs mark both as `Complete`. The component-level work for D101/D102/D103 is already landed (FP1-FP2 commit pair `bf79b39` + `c26b79a`), so D101/D102 are now *coverage-deliberate* reopens of the same vectors (extra fixtures, dedicated invariant tests, evidence trails), not first-pass bug fixes. I will not rewrite the planned baseline — the starting gate is the executed parent stories + their recorded residuals, and D101/D102/D103 will strengthen/extend rather than re-do them.

### D101 verification evidence

- `cargo test -p talos-provider openai::tests::parse_sse_stream`: 14 passed / 0 failed / 0 ignored (8 inherited FP1-FP2 fixtures + 6 new D101 fixtures). The six new ones: `parse_sse_stream_finish_reason_length_emits_max_tokens`, `parse_sse_stream_role_only_first_chunk_does_not_emit_or_hang`, `parse_sse_stream_keepalive_comment_lines_pass_through`, `parse_sse_stream_empty_data_event_is_skipped`, `parse_sse_stream_mixed_text_and_tool_call_in_same_delta_emits_both`, `parse_sse_stream_utf8_multibyte_content_round_trips`.
- `cargo test -p talos-provider`: 86 passed / 0 failed / 0 ignored (incl. 4 anthropic tests). No regression.
- `cargo clippy -p talos-provider -- -D warnings`: passed (exit 0).
- `cargo test --workspace`: 1784 passed / 0 failed / 0 ignored across 61 active test binaries (was 1778 at D100 baseline → +6 from D101 fixtures). Doctests for `talos-provider` (x2) still green.
- `cargo fmt --all -- --check`: still only the pre-existing `bash_tool.rs:583` drift recorded at D100. No new drift introduced by D101.
- `scripts/validate_project_governance.sh .`: passed, 0 governance warnings.
- No production parser code was changed. Per the runtime-evidence hard boundary, D101 is a deterministic-fixture extension that locks existing parser behavior; the fixtures themselves are the evidence. D102 will add the agent-layer invariant that closes any remaining silent-stuck tail.

## Variance And Residuals

### Pre-existing `bash_tool.rs` format drift (out of I102 scope)

- `crates/talos-tools/src/bash_tool.rs:583` carries one rustfmt diff (multi-line `&&` chain collapsed by `27626d9`). D100 must not silently fix unrelated code; this stays a residual for the next owner whose scope touches `talos-tools`. `cargo fmt` will resolve it in one line. Tests still pass.
- This is the only baseline drift found by the start-gate sweep.

### Owner-doc status disagreements inherited by I102

- `RUNTIME-002` and `PROVIDER-002` are recorded as `Complete` in their owner docs, but `I102` `## Selected Stories` lists D101/D102/D103 as `Planned`. Per the iteration published-baseline rule, I will not rewrite the plan; D101-D103 become *coverage-extension* packets on the same provider/agent invariants, not first-pass bug fixes. Acceptance criteria in `I102` still hold (see `## Acceptance`).
- The 2026-07-07 provider-runtime hardening next-phase task and the 2026-07-07 frontline runtime/UX stability plan both fed I102. `I102` does not supersede their closeouts; it inherits their residuals and packetises the remaining work.

### Residuals exposed by D104 / month-1 closeout to track

- Nothing D100 itself uncovered. D104 will produce the month-1 closeout residual set.
