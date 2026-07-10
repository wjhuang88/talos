# Issue / Document / Code Status Audit — 2026-07-09

This audit reconciles GitHub issues, local owner docs, and implementation evidence after the
I106-I109 self-bootstrap closeout review. It is a status index only; executable requirements remain
in the owner docs listed below.

## Summary

- Fixed and synchronized: #18, #35.
- Correctly open after audit: #22, #24, #25, #26, #31, #39.
- Correctly closed from the earlier issue batch: #19, #20, #21, #23, #27, #33, #34.
- Proposal/open backlog still needs separate intake or activation: #29, #30, #32, #36, #37, #38,
  #40.

## Matrix

| Issue | GitHub state after audit | Owner doc | Code / evidence state | Action |
|---|---|---|---|---|
| #18 stuck processing after tool/provider failure | Closed | `RUNTIME-002`, `PROVIDER-002` | Fixed by I107: `dispatch_timeout_secs`, OpenAI/Anthropic `send().await` timeout, provider tests, `AgentEvent::Error` bridge test, conversation-loop `TimedOut` terminal status test. | Closed with status comment. |
| #19 todo batch/update + UUID hiding | Closed | `TODO-002` | Implemented and registered in print/TUI tool registries. | No action. |
| #20 edit diff coloring | Closed | `TOOL-018` | Diff rendering implemented in TUI/tool display path. | No action. |
| #21 git diff tool | Closed | `TOOL-018`, `TOOL-020` | Unified/staged/path-filtered git diff implemented; ref-to-ref comparison intentionally deferred to `TOOL-020`. | No action unless ref-to-ref is reselected. |
| #22 workspace trust sandbox | Open | `PERM-004`, `PERM-005` | Design updated: non-Git strict mode; Git repo-root sandbox only after approval; bash/exec broadening waits for touched-path evidence/enforcement. | Keep open. |
| #23 bash exit code classification | Closed | `TOOL-019` | Expected non-zero exit codes classified without false tool error. | No action. |
| #24 processing animation cadence | Open | `TUI-028` | Code has a 50ms render interval, but no runtime/visual proof under heavy rendering or long-output load. | Reopened; add evidence or implementation. |
| #25 thinking ripple animation | Open | `TUI-028` | Current code animates the `"thinking"` label gradient; it does not implement the requested two-color three-segment center-out ripple block animation. | Keep open; implement or revise requirement. |
| #26 thinking content history | Open | `TUI-029` | ADR-034 v4 approved the bounded display projection on 2026-07-10. Implementation has not started; TUI-029 is Ready for Implementation in a new iteration. | Keep open until code, runtime evidence, and owner-doc acceptance are complete. |
| #27 stale preview clear | Closed | `TUI-028` | Engine clears preview on cancel/error/turn lifecycle; conversation-loop tests cover terminal cleanup paths. | No action. |
| #28 dashboard message format | Closed, superseded by #39 | `TUI-028` | Original issue remains unimplemented as transient notification; #39 is the active reopened issue. | Track through #39. |
| #31 model switch status-bar jump | Open | `TUI-028` | Code truncates labels, but no runtime/visual evidence proves transition stability. | Reopened; add evidence or implementation. |
| #32 health-check thread proposal | Open | `RUNTIME-002` | RUNTIME-002 keeps health-check as optional future work; no background task added. | Keep open unless selected. |
| #33 `/todo delete` | Closed | `TODO-002` | Implemented with confirmation and short-ID handling. | No action. |
| #34 todo create idempotency | Closed | `TODO-002` | Implemented with idempotent create/batch behavior. | No action. |
| #35 single data-flow audit | Closed | `ARCH-032` | I108 audit complete; zero `broadcast::channel` usages; channel topology documented. | Closed with status comment. |
| #36 tool error propagation audit | Open | None selected in this audit | Not audited here. | Needs owner doc or selection. |
| #37 input history up/down | Open | None selected in this audit | Not implemented here. | Needs owner doc or selection. |
| #38 long-running task engine | Open | None selected in this audit | Not implemented here. | Needs owner doc or selection. |
| #39 dashboard transient notification | Open | `TUI-028` | Not implemented: dashboard availability still enters persistent scrollback as System stream line. | Keep open. |
| #40 multi-Talos discovery/communication | Open | None selected in this audit | Not implemented here. | Needs requirement intake; high architecture risk. |

## Verification Performed

- GitHub issue states checked with `gh issue list --state all --limit 80`.
- GitHub status sync performed:
  - Closed #18.
  - Closed #35.
  - Reopened #24.
  - Reopened #31.
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

Do not close #24, #25, #31, or #39 until the owner doc records direct runtime/visual evidence or a
reviewed requirement change. #26 may now be implemented only within ADR-034 v4 and TUI-029's
activation/test gates; the issue remains open until runtime evidence closes acceptance.
