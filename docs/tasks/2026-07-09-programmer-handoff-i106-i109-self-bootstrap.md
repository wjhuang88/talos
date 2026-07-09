# Programmer Handoff: I106-I109 Talos Self-Bootstrap Closeout

**Date**: 2026-07-09
**Current audited HEAD**: `b4eb430` plus this follow-up status/evidence closeout
**Plan doc**: `docs/tasks/2026-07-08-four-month-talos-self-bootstrap-plan.md`
**Runtime classification**: glm-5.2 via zai-coding-plan was the primary executor for I106-I109, not
the `talos` binary. All I106-I109 evidence is therefore useful but **non-qualifying** for REL-002.

## Outcome

I106-I109 were executed and closed to Review, but the self-bootstrap objective was not achieved.

| Iteration | Tasks | Result | REL-002 |
|---|---|---|---|
| I106 | SBT100-SBT104 | Execution contract, evidence schema, smoke harness, governance rehearsal, month-1 closeout. | Non-qualifying |
| I107 | SBT110-SBT113 | #18 request-dispatch timeout fixed for OpenAI-compatible and Anthropic providers. | Non-qualifying |
| I108 | SBT120-SBT123 | ARCH-032 Single Data Flow Audit complete; no ADR-006 deviations found. | Non-qualifying |
| I109 | SBT130-SBT133 | REL-002 evidence audit and NO-GO v1.0 readiness report complete. | Non-qualifying |

## Final Validation Claims

The I109 closeout records a green validation matrix at the time of `b4eb430`:

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- `scripts/validate_project_governance.sh .`
- `git diff --check`
- `scripts/talos_smoke.sh`

The follow-up audit after `b4eb430` found missing status sync and missing #18 bridge evidence. The
corrective follow-up adds:

- `run_streaming_emits_error_event_on_provider_dispatch_timeout`
- `conversation_loop_clears_processing_on_dispatch_timeout_error`

These prove provider dispatch timeout propagates to the streaming event channel and the conversation
loop emits terminal `Status { is_processing: false, phase: TimedOut }`.

Follow-up validation after the corrective sync:

- `cargo fmt --all -- --check` passed.
- `cargo check --workspace` passed.
- `cargo test --workspace` passed.
- `cargo clippy --workspace -- -D warnings` passed.
- `scripts/validate_project_governance.sh .` passed with 0 warnings.
- `git diff --check` passed.
- `scripts/talos_smoke.sh` passed 9/9.

## GitHub Issue / Local Owner Status

Detailed matrix: `docs/reference/ISSUE-DOC-CODE-STATUS-2026-07-09.md`.

| Issue | GitHub State After Audit | Local Owner | Current Truth |
|---|---|---|---|
| #18 | Closed | `RUNTIME-002`, `PROVIDER-002` | Fixed by I107 plus follow-up bridge tests. |
| #24 | Reopened | `TUI-028` | Still needs runtime/visual cadence evidence under heavy rendering/load. |
| #25 | Open | `TUI-028` | Still not implemented: current code animates a gradient label, not the requested two-color three-segment ripple. |
| #26 | Open | `TUI-029` | Planned; requires ADR-034/TUI-020 policy decision before implementation. |
| #28/#39 | #28 closed, #39 open | `TUI-028` | Still open through #39: dashboard availability must be transient `UiOutput::Tip`, not persistent scrollback. |
| #31 | Reopened | `TUI-028` | Still needs runtime/visual evidence for model-switch layout stability. |
| #35 | Closed | `ARCH-032` | Audit complete; no ADR-006 deviations found. |

## Remaining Work

1. TUI-028:
   - #25 two-color three-segment center-out ripple thinking animation.
   - #39 transient dashboard notification.
   - #24 runtime/visual animation cadence evidence.
   - #31 runtime/visual model-switch layout evidence.
2. TUI-029:
   - #26 thinking history archive policy decision before implementation.
3. REL-002:
   - Start a new Talos-primary attempt only when the `talos` binary is the actual primary executor.
   - Do not claim v1.0 readiness until REL-002 criteria have fully qualifying evidence.

## Operating Rules For The Next Attempt

- Start from a clean worktree and record exact HEAD.
- Run `scripts/talos_smoke.sh` before selecting work.
- Use owner docs as source of truth; update `docs/BOARD.md` only after owner docs.
- Close or reopen GitHub issues only when owner docs and code evidence agree.
- Classify any external-runtime implementation as non-qualifying for REL-002.
- No tag, release, publish, permission/sandbox relaxation, dependency addition, credential change,
  storage-default migration, or external trial invitation without a separate gate.
