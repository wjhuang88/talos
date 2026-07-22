# Issue / Document / Code Status Audit — 2026-07-15

> Status: Complete — the 2026-07-15 remote-state refresh and owner synchronization are recorded
> below. It is a historical status index, not an ongoing product iteration.
>
> Completion Commit: `a1e9985` — `docs(backlog): sync open issue owners`.

This audit reconciles GitHub issues, local owner docs, and implementation evidence after the
I106-I109 self-bootstrap closeout review and the 2026-07-15 remote-state refresh. It is a status
index only; executable requirements remain in the owner docs listed below.

## Summary

- Fixed and synchronized: #18, #24, #25, #26, #31, #35, #39.
- Correctly open after refresh: #22, #29, #30, #32, #36, #37, #38, #40.
- Correctly closed from the earlier issue batch: #19, #20, #21, #23, #27, #33, #34.
- Proposal or refinement work still needs separate activation: #29, #30, #32, #36, #37, #38, #40.

## Matrix

| Issue | GitHub state after audit | Owner doc | Code / evidence state | Action |
|---|---|---|---|---|
| #18 stuck processing after tool/provider failure | Closed | `RUNTIME-002`, `PROVIDER-002` | Fixed by I107: `dispatch_timeout_secs`, OpenAI/Anthropic `send().await` timeout, provider tests, `AgentEvent::Error` bridge test, conversation-loop `TimedOut` terminal status test. | Closed with status comment. |
| #19 todo batch/update + UUID hiding | Closed | `TODO-002` | Implemented and registered in print/TUI tool registries. | No action. |
| #20 edit diff coloring | Closed | `TOOL-018` | Diff rendering implemented in TUI/tool display path. | No action. |
| #21 git diff tool | Closed | `TOOL-018`, `TOOL-020` | Unified/staged/path-filtered git diff implemented; ref-to-ref comparison intentionally deferred to `TOOL-020`. | No action unless ref-to-ref is reselected. |
| #22 workspace trust sandbox | Open | `PERM-004`, `PERM-005` | Design updated: non-Git strict mode; Git repo-root sandbox only after approval; bash/exec broadening waits for touched-path evidence/enforcement. | Keep open. |
| #23 bash exit code classification | Closed | `TOOL-019` | Expected non-zero exit codes classified without false tool error. | No action. |
| #24 processing animation cadence | Closed | `TUI-028` | I114 native Alacritty PTY evidence accepted stable runtime cadence after `c68fd08`; closeout `072c726`. | Closed with maintainer evidence comment. |
| #25 thinking ripple animation | Closed | `TUI-028` | Native Alacritty review accepted the two-color, three-segment center-out ripple after `c68fd08`; visual-only semantics preserved. | Closed with maintainer evidence comment. |
| #26 thinking content history | Closed | `TUI-029` | Typed reasoning history, safe resume projection, explicit `--include-thinking` export, and cancellation/error exclusion are implemented; owner record cites `6970af9`, `4ebf73e`, and `26211d3` plus workspace validation. | Closed during the 2026-07-15 issue refresh. |
| #27 stale preview clear | Closed | `TUI-028` | Engine clears preview on cancel/error/turn lifecycle; conversation-loop tests cover terminal cleanup paths. | No action. |
| #28 dashboard message format | Closed, superseded by #39 | `TUI-028` | #39 replaced the original requirement; its transient `UiOutput::Tip` behavior was later accepted with native PTY evidence. | No action. |
| #29 talos desktop | Open | `docs/proposals/talos-desktop.md` | Proposal only; no implementation task is selected. | Keep open until a bounded feature intake is selected. |
| #30 multi-agent architecture | Open | `docs/proposals/multi-agent-architecture.md` | Proposal only; no implementation task is selected. | Keep open until a bounded, ADR-aligned feature intake is selected. |
| #31 model switch status-bar jump | Closed | `TUI-028` | Native Alacritty verification accepted compact model/provider status rendering after `823a8e0`; display-width truncation remains bounded. | Closed with maintainer evidence comment. |
| #32 health-check thread proposal | Open | `RUNTIME-002` | RUNTIME-002 keeps health-check as optional future work; no background task added. | Keep open unless a separately scoped safety design is selected. |
| #33 `/todo delete` | Closed | `TODO-002` | Implemented with confirmation and short-ID handling. | No action. |
| #34 todo create idempotency | Closed | `TODO-002` | Implemented with idempotent create/batch behavior. | No action. |
| #35 single data-flow audit | Closed | `ARCH-032` | I108 audit complete; zero `broadcast::channel` usages; channel topology documented. | Closed with status comment. |
| #36 tool error propagation audit | Open | `TOOL-021` | Refinement audit only; no tool/provider behavior change selected. | Trace and review data-flow before activation. |
| #37 input history up/down | Open | `TUI-030` | Refinement owner scopes an in-memory composer-history slice only. | Refine and select independently. |
| #38 long-running task engine | Open | `TASK-001` | ADR-gated architecture/security spike; no engine or scheduler selected. | Produce reviewed decision before implementation. |
| #39 dashboard transient notification | Closed | `TUI-028` | Native Alacritty evidence accepted `UiOutput::Tip` routing with no stale output or blank startup row after `823a8e0`; closeout `072c726`. | Closed with maintainer evidence comment. |
| #40 multi-Talos discovery/communication | Open | `A2A-001` | ADR-gated architecture/security spike; no discovery/protocol selected. | Produce threat-modelled decision before implementation. |

## Verification Performed

- GitHub issue states checked with `gh issue list --state all --limit 100` on 2026-07-15.
- GitHub status sync performed:
  - Closed #18.
  - Closed #35.
  - Reopened #24.
  - Reopened #31.
  - On 2026-07-15, closed #26 and confirmed #24, #25, #31, and #39 have subsequent accepted
    closure evidence.
- Local targeted tests added and run:
  - `cargo test -p talos-agent run_streaming_emits_error_event_on_provider_dispatch_timeout`
  - `cargo test -p talos-cli conversation_loop_clears_processing_on_dispatch_timeout_error`
- Follow-up closeout validation passed:
  - `cargo fmt --all -- --check`
  - `cargo check --workspace`
  - `cargo test --workspace`
  - `cargo clippy --workspace -- -D warnings`
  - `scripts/validate_project_governance.sh .`
  - `git diff --check`
  - `scripts/talos_smoke.sh` (9/9)

## Residual Rule

Keep #22, #29, #30, #32, #36, #37, #38, and #40 open until their owner docs record an activated,
validated implementation or an explicit reviewed disposition. Do not reopen #24, #25, #26, #31, or
#39 without contrary runtime/visual evidence or a new requirement.
