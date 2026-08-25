# I225 Windows Job Object Current-Path And Migration Matrix

**Status**: Proposed decision evidence for I225 / TOOL-024-D1-A; no implementation authority.
**Baseline**: `main@e698072252cdc0966e4046a83bc3a6503c008efb`.

## Current Authority Map

| Concern | Current owner | Observed behavior | D1-B boundary |
|---|---|---|---|
| Typed background intent | `talos-tools/src/bash_tool.rs`, `exec_tool.rs` | `background` is explicit; Windows admission returns `background_process_tree_unsupported`. | Preserve schema and foreground path; select only a Windows launcher after ownership is proven. |
| Foreground Windows identity | `talos-tools/src/bash_tool.rs` / ADR-057 | Native `powershell.exe -NoLogo -NoProfile -NonInteractive -Command`; direct-child timeout residual. | Reuse identity; do not translate shell text. |
| Unix background launcher | `talos-tools/src/process_boundary.rs` | `setsid`, bounded pipes, process-group control and leader reap. | Unchanged; D1-B is Windows-only. |
| Agent supervisor | `talos-agent/src/background_jobs.rs` | Admission, launch fence, session identity, bounded cursor/output, cancel and finalizer. | Consume a checked Windows `BackgroundProcessControl`; do not create a second supervisor. |
| Model process control | `talos-agent/src/process_tool.rs` | Read/status/list use Read; cancel uses exact job resource. | No API/schema or permission change. |
| Tool execution admission | `talos-agent/src/tool_execution.rs` and `talos-core/src/background_job.rs` | One normalized admission/launch path; unsupported platform fails before spawn. | Preserve final gate and exact-once launch. |
| Runtime shutdown/finalizer | `talos-agent/src/session.rs`, `talos-runtime/src/lib.rs` | Session finalizer invokes supervisor closure before shutdown completion. | Reuse existing ordering/report; no new finalizer authority. |
| Permission resource | `bash_tool.rs`, `exec_tool.rs`, `process_tool.rs` | Background facet is explicit; cancel resource contains job identity. | Do not broaden foreground grants or alter namespace. |
| Persistence | `talos-session` / turn persistence | Live jobs are not persisted across restart. | Preserve live-session-only boundary. |

## Proposed Migration Sequence

1. Accept ADR-068 and create a separate D1-B owner/iteration/claim.
2. Select a Windows-native API binding and add only the minimum Windows-target dependency under D1-B,
   with an exact lockfile and unsafe review.
3. Implement a private launcher that creates/configures Job + allowlisted stdio handles while
   suspended, assigns before resume, and returns the existing platform-neutral launcher contract.
4. Add focused Windows tests for assignment ordering, descendants, inherited handles, partial
   cleanup, cancellation, deadline, shutdown and output.
5. Obtain exact-head Windows CI and independent process/unsafe/API review; only then enable Windows
   admission. D2 and I223 follow their own owners.

## Compatibility And Rollback

- Before D1-B merge, Windows behavior remains exactly `background_process_tree_unsupported`.
- If any setup or ownership proof fails, return a typed error and terminate/close without resuming
  an unowned child.
- Revert D1-B's private launcher/feature gate to restore fail-closed admission; no session schema
  migration or data rollback is required.
- A public API/schema or permission change is out of scope and requires a new decision/claim.

## Evidence Commands

```bash
rg -n "background_process_tree_unsupported|ToolExecutionAdmission::Background" \
  crates/talos-tools/src/bash_tool.rs crates/talos-tools/src/exec_tool.rs
rg -n "BackgroundJobSupervisor|process_action|finalizer" \
  crates/talos-agent/src/background_jobs.rs crates/talos-agent/src/process_tool.rs \
  crates/talos-agent/src/session.rs crates/talos-runtime/src/lib.rs
rg -n "windows-sys|winapi" Cargo.toml crates/*/Cargo.toml Cargo.lock
```
