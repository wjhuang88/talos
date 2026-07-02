# T130 Tool Reliability Sweep

**Status**: Complete
**Date**: 2026-07-02
**Parent**: `docs/tasks/2026-07-01-four-month-self-bootstrap-replan.md`
**Iteration**: I079 Month 4 release readiness and handoff

## Scope

T130 reviewed the residual reliability items called out by the I077 closeout:

- flaky or ignored tests;
- runtime/example warning noise;
- shell naming and Windows/Unix assumptions.

This was a reliability sweep, not a cross-platform tool redesign. Shell naming and Windows command
support remain tracked by TOOL-006 because changing the `bash` tool contract affects user-facing
tool schemas, permission prompts, and compatibility aliases.

## Findings And Actions

| Area | Finding | Action | Status |
|---|---|---|---|
| Agent session tests | `test_interrupt_after_success_preserves_history` was ignored because it used sleeps around an interrupt-after-success race. | Removed the ignore and synchronized on `SessionEvent::TurnCompleted` from the event queue for both turns. | Fixed |
| Runtime examples | `cargo test --workspace` emitted dead-code warnings from shared example helpers that are intentionally reused unevenly across examples. | Added a module-level dead-code allowance to the example-only common helper module. | Fixed |
| Ignored test inventory | Source scan found no remaining `#[ignore]` in `crates/`. | No code change needed beyond the session test fix. | Clear |
| Shell naming / Windows assumptions | The legacy `bash` naming and Windows `cmd`/PowerShell support are still product-contract work. | Kept deferred to TOOL-006 instead of changing schemas in T130. | Deferred |

## Verification

- `cargo fmt --all -- --check`
- `cargo test -p talos-agent test_interrupt_after_success_preserves_history`
- `cargo test -p talos-runtime --examples`
- `cargo clippy -p talos-agent -p talos-runtime -- -D warnings`
- `rg -n "#\\[ignore\\]" crates docs`
- `cargo test -p talos-agent`
- `scripts/validate_project_governance.sh .`

Results:

- `cargo test -p talos-agent` passed with `190 passed; 0 failed; 0 ignored`.
- `cargo test -p talos-runtime --examples` passed without the previous runtime example dead-code warnings.
- `scripts/validate_project_governance.sh .` passed with 0 warnings.
- The only `#[ignore]` text remaining is governance/documentation guidance or historical I040 text,
  not active source code.

## Residuals

- TOOL-006 remains the owner for `bash` -> `sh` naming, Windows `cmd`/PowerShell support, and
  backward-compatible aliases.
- No publish, tag, release, permission default change, exec DSL expansion, or plugin host-call
  expansion was authorized or performed by this sweep.
