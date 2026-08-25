# 068: Windows Job Object Process Ownership For Supervised Background Jobs

## Status

Accepted on 2026-08-25 for I225 / TOOL-024-D1-A through decision PR #391 merge `0021690e`.
The accepted decision content is commit `fca45c46`, with exact-head CI `32797375011` and
independent Windows/process/unsafe/API approval `5404361120`.

This ADR is a decision contract only. Its acceptance did not enable Windows background execution,
add a dependency, add `unsafe`, change a public API, or change fail-closed behavior. D1-B remains a
separate implementation iteration and claim and must independently satisfy this contract.

## Context

ADR-060 requires every background process to be owned by a supervisor and requires Windows to
remain fail-closed until a separately accepted Job Object boundary can own descendants. ADR-057
defines the existing Windows PowerShell identity and intentionally retains direct-child-only
cleanup. I222/I224 now provide the live session supervisor, bounded output, model-readable process
control and ordered finalizers, but `talos-tools` still rejects Windows background admission with
`background_process_tree_unsupported`.

The current Windows path uses Tokio's process abstraction for foreground PowerShell/direct exec.
That abstraction is not, by itself, a proof that a suspended primary-thread handle, Job Object
assignment, child-handle allowlist and kill-on-close ownership are all available at the required
ordering points. A spawn-then-assign design is therefore not an acceptable migration.

## Decision

### Ownership ordering

The later D1-B launcher must perform this order for a background process:

1. Validate the already-authorized command and create/configure one private Job Object with
   `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, plus only the required stdout/stderr pipe endpoints. No
   user code may run before ownership is established. The Job handle remains owned by the
   launcher/supervisor until the terminal cleanup path transfers or closes it.
2. Build the `STARTUPINFOEXW` attribute list and its allowlisted child-handle list before process
   creation. Only the required child stdout/stderr handles may be inherited. The design must
   prevent unrelated inheritable handles from entering a concurrently created child. If the
   binding cannot prove the allowlist, fail closed before attempting to launch.
3. Create the Windows process with `CreateProcessW` (or a binding that proves identical semantics)
   using `CREATE_SUSPENDED` and the existing ADR-057 PowerShell command identity. Capture the process
   and primary-thread handles; do not use shell jobs or `taskkill` as the ownership primitive. The
   attribute list is supplied at this call, so no user code runs before the suspended state is
   reached.
4. Assign the suspended process to the private Job Object. Assignment failure is terminal: terminate
   the suspended process, close the process/thread/job/attribute-list/pipe duplicates, and return a
   typed launch failure. Never resume an unassigned process.
5. Resume the primary thread only after steps 1-4 succeed. A resume failure is terminal and must
   invoke the same bounded cleanup path.
6. Transfer the process/job control and pipe readers to the existing Agent/session supervisor.
   The supervisor owns one terminal race among natural exit, deadline, cancel and shutdown. Closing
   the final Job handle after the required grace/force sequence must terminate remaining descendants
   through kill-on-close.

### Handle and inheritance boundary

- D1-B must use a Windows-native binding with explicit feature closure; the dependency choice and
  version are recorded in the implementation owner and cannot be smuggled into this ADR's claim.
- All process, primary-thread, Job Object, attribute-list and pipe handles have one owner and an
  explicit RAII/cleanup path. Parent and child duplicates are closed at the first safe point after
  successful creation and on every partial failure.
- `bInheritHandles` and attribute-list behavior must be treated as one contract. The implementation
  must inherit only the allowlisted stdio handles and must not rely on broad process-wide inheritable
  state. Concurrent spawn tests must prove an unrelated inheritable handle is not observable in the
  child.
- Required access rights, null/invalid handles, UTF-16 conversion, size arithmetic and Win32 error
  values are checked. Any inability to create/configure the attribute list, Job Object or pipe set
  fails closed.
- No parent environment mutation, shell syntax inference, detached child, `taskkill`, PowerShell
  `Start-Job`, or direct-child-only cancellation is an acceptable substitute.

### State and failure contract

The implementation must make these states explicit in tests and returned errors:

`validated -> resources_created -> job_configured -> suspended_created -> assigned -> resumed ->
supervised -> terminal`.

For a failure in any state before `resumed`, no user code is allowed to continue and all owned
resources are closed. For a failure after `resumed`, the supervisor must attempt graceful Job
termination, apply the existing bounded force deadline, close the Job handle, reap the leader and
report cleanup failure rather than silently claiming success. Nested Job Object rejection,
unsupported host policy, channel closure, deadline expiry, cancellation and shutdown all fail
closed. A child that cannot be proven owned is never admitted.

### Compatibility and migration

- Foreground PowerShell/direct-exec behavior, permission resources, public SDK types, Unix process
  groups and I222/I224 model-readable contracts remain unchanged.
- Windows background admission remains rejected until D1-B lands with the accepted implementation
  evidence. D1-B may add a platform-private launcher seam; any public API or serialized schema
  change requires a separate migration decision.
- D2 owns CLI/TUI projection, help text and integrated Unix/Windows walkthroughs. I223 owns the
  deferred human/device rows. Neither may be folded into D1-B's process-ownership authority.

## D1-B Implementation Boundary

Expected production files are limited to the Windows process-boundary launcher and its focused
tests, plus the minimum composition seam required to select it. The implementation owner must
publish a changed-file inventory before push and prove no Dashboard/I213, permission, `/auto`,
release, Desktop, D2 or I223 overlap. The exact inventory is not authorized by this ADR alone.

## Validation Matrix

D1-B must provide real Windows evidence for:

| Case | Required proof |
|---|---|
| Assigned-before-exec | A child writes a marker only after resume; forced assignment failure leaves no marker/process. |
| Descendant cleanup | Child creates a grandchild; cancel, timeout and shutdown terminate/reap both. |
| Handle allowlist | Child can use required stdio; unrelated inheritable handles are absent under concurrent spawn. |
| Partial setup failure | Attribute-list, pipe, Job, assignment and resume failures close all parent/child duplicates and leave no process. |
| Nested/unsupported Job | Rejection is typed, fail-closed and preserves foreground behavior. |
| Output/deadline | Existing bounded stdout/stderr and absolute deadline semantics remain truthful. |
| Session/control | `process` status/read/list/cancel remains session-scoped and terminal delivery is exactly once. |
| Shutdown | Runtime finalizer ordering reaches a terminal cleanup report before session shutdown closes. |

## Security Review Requirements

Independent review must inspect every Windows OS-ABI/`unsafe` site, handle right, inheritance flag,
attribute-list lifecycle, process creation flag, Job limit, error path and concurrent-spawn test.
The review must reject any implementation that resumes before assignment, uses broad inherited
handles, relies on direct-child kill, or cannot prove complete partial-failure cleanup.

## Rejected Alternatives

- Spawn then assign: permits user code to run outside Talos ownership.
- `taskkill`, shell jobs or direct-child `kill`: do not establish an owned descendant boundary.
- Broad `bInheritHandles`: leaks unrelated inheritable handles in a concurrent host.
- Resume before handle-list/Job configuration: creates an unowned execution window.
- Persisting live jobs: expands this slice into TASK-001 without restart semantics.

## Rollback and Reversal Triggers

Rollback is to reject D1-B and retain `background_process_tree_unsupported` on Windows. Revisit this
ADR if Rust gains a proven safe cross-platform process-tree API, Windows Job Object policy changes,
the permission namespace is replaced, or a durable task runtime supersedes the live-session model.

## Related

- [ADR-060: Supervised Background Command Job Lifecycle](060-supervised-background-command-jobs.md)
- [ADR-057: Windows PowerShell Process Boundary](057-windows-powershell-process-boundary.md)
- [ADR-007: Process Hardening And Unsafe Boundary](007-process-hardening-unsafe.md)
- [TOOL-024-D1-A owner](../backlog/active/TOOL-024-D1-A-windows-job-object-decision.md)
