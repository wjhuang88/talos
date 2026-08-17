# 060: Supervised Background Command Job Lifecycle

## Status

Proposed (I188 / TOOL-024-A, 2026-08-14)

This decision is not an implementation authorization until the exact decision head receives
independent process/permission security review and merges to `main`. It introduces no production
background process, dependency, `unsafe` block, persistence migration, or user-visible behavior.

## Context

Issue #59 asks Talos to run an explicitly requested command without blocking the interactive
conversation. The current `bash`/PowerShell and `exec` tools are foreground operations:

- the tool call owns the child until exit or timeout;
- `exec` retains at most 32 KiB per stdout/stderr stream, while `bash` currently builds an
  unbounded combined `String`;
- timeout kills and waits for only the direct child;
- Unix children already enter a new session through the ADR-007 `setsid(2)` boundary, but Talos
  does not signal that whole process group;
- Windows PowerShell identity is defined by ADR-057, whose accepted residual is direct-child-only
  cleanup;
- a normal `Message::Tool` result must correspond to one provider tool-call ID inside a turn. A
  late second result for the same ID would duplicate the provider result and could trigger work the
  user did not request;
- session shutdown has no bounded finalizer registry, and permission evaluation/approval still has
  more than one composition owner.

The complete source characterization is
[`I188-BACKGROUND-JOB-CURRENT-PATH.md`](../reference/I188-BACKGROUND-JOB-CURRENT-PATH.md).

## Constraints

| Constraint | Type | Consequence |
|---|---|---|
| No unmanaged child process | Hard | Background spawn remains disabled wherever Talos cannot terminate and reap the owned process tree. |
| Foreground permission is not background permission | Hard | A foreground `always` grant cannot silently start a longer-lived job. |
| No global pub/sub bus | Hard / ADR-006 | The live session actor owns one supervisor and emits through its existing event channel. |
| RUNTIME-005 completes before production spawn | Hard / TOOL-024 | The supervisor must be a bounded runtime-finalizer consumer, not a parallel shutdown system. |
| PERM-006-C completes before production spawn | Hard / TOOL-024 | One agent-owned pipeline must gate approval, authorization, and spawn. |
| Public APIs are semver-bound | Hard | Additive tool fields default to foreground; new session events use the existing non-exhaustive event boundary. |
| Restart survival is excluded | Scope | Job IDs and retained output are valid only in the originating live runtime/session. |
| No automatic provider continuation | Product | Completion is visible to the host and retrievable by an explicit `process` call, but never submits a turn. |

## Decision

### 1. Ownership and lifecycle

The live `talos-agent` session actor owns one `BackgroundJobSupervisor`. `talos-tools` owns only
platform launch, bounded pipe capture, signal/termination primitives, and typed command validation.
The supervisor is injected into the agent tool-execution path; it is not a global singleton or an
event bus.

`BackgroundJobId` is an opaque, session-scoped newtype whose display form is `job_<uuid-v4>`.
Generation occurs in the supervisor. A job ID is stable for the life of that Talos process and is
never accepted by another session, even if its text is known.

The states are monotonic:

```text
starting -> running -> completed | failed | timed_out | cancelled | supervision_failed
starting -----------^ spawn_failed
```

Only the supervisor may perform the terminal compare-and-set. Natural exit, timeout, cancellation,
output-reader failure, and shutdown races therefore produce exactly one terminal state and at most
one terminal event. `spawn_failed` receives an allocated ID for attribution but owns no process.

Limits are part of the first implementation contract:

- at most 8 non-terminal jobs per live session;
- at most 32 retained terminal jobs per live session;
- running jobs are never evicted;
- terminal jobs evict oldest-first after the 32-job cap;
- eviction returns a typed `expired` result and never aliases a reused ID.

### 2. Typed start contract

Existing platform tool names stay stable: `bash` on Unix, `powershell` on Windows, and `exec` on
all platforms. `BashInput` and `ExecInput` gain one additive field:

```json
{ "background": true }
```

The field is `bool`, defaults to `false`, and is included in schema and approval projection.
Shell syntax such as `&`, `nohup`, `Start-Job`, or detached flags never implies background mode.

TOOL-024-B supports background mode only for Unix shell calls and a single top-level Unix `exec`
command. Background `exec.steps`, `exec.pipes`, and `ExecMode::Parallel` fail before spawn with
`unsupported_background_shape`; foreground behavior is unchanged. Windows background mode fails
before permission grant installation or spawn with `background_process_tree_unsupported` until the
TOOL-024-D Windows gate below is accepted.

The existing `timeout_secs` becomes the absolute job deadline and remains clamped to 1–600 seconds.
The immediate normal tool result is a deterministic receipt containing job ID, `running` state,
tool identity, and deadline. That receipt completes the provider tool call and unblocks the turn.

### 3. Permission separation

A background start presents two conservative permission facets:

1. the existing foreground command facet, preserving current command/resource policy; and
2. an Execute/Command facet in the reserved resource namespace
   `background:<tool-name>:<normalized-command-resource>`.

The structured PERM-006 request marks the second facet as requiring an explicit resource-scoped
background grant. Deny on either facet denies the invocation. A generic or foreground Execute
Allow may satisfy the first facet but cannot satisfy the second; it degrades to Ask. `Always
approve` after that distinct prompt stores only the exact `background:` resource (or an explicitly
chosen background namespace pattern), never a generic Execute grant.

This reserved namespace is the compatibility bridge until PERM-006-D introduces typed effects. It
must not be inferred from the human-readable description field. PERM-006-B owns scoped grant
storage, and PERM-006-C owns the single evaluation/approval/authorization/spawn path. Approval
failure, cancellation, closed channels, invalid background shape, or missing supervisor fails
closed and executes zero processes.

The `process` tool uses `Read` facets for `list`, `status`, and `read`; session ownership is still
checked inside the supervisor. `cancel` uses the exact job's background Execute/Command resource,
so read authority cannot terminate a process and one job's grant cannot control another job.

### 4. Bounded ordered output

The supervisor records stdout and stderr as one globally ordered sequence of chunks. Each chunk has
`cursor: u64`, stream, bytes, and timestamp. Retained payload is capped at 64 KiB per job combined,
with at most 32 KiB returned by one read. When the ring evicts old bytes it advances
`earliest_cursor`; reads behind it return `truncated: true` plus the new earliest cursor. Invalid
UTF-8 is lossily projected for text while byte counts and truncation remain accurate.

The terminal summary includes state, exit code when available, total stdout/stderr byte counts,
retained range, truncation, start/finish times, and cleanup outcome. Reader failure becomes
`supervision_failed`; it is not reported as command success.

### 5. Result routing and `process` tool

The model-readable tool is named `process` and has one tagged operation:

- `list` — bounded summaries for the owning live session;
- `status { job_id }` — state and terminal metadata;
- `read { job_id, after_cursor?, max_bytes? }` — ordered chunks, capped at 32 KiB;
- `cancel { job_id }` — idempotently request cancellation and return the observed state.

Job completion emits one new UI-neutral, non-exhaustive `SessionEvent` carrying the terminal
summary. It is not a second `Message::Tool`, is not inserted into provider history, and does not
submit a provider request. A later explicit model/user `process` call produces an ordinary tool
result and is the only way output enters later model context.

The first implementation does not persist job state or the unsolicited terminal event. The
original start receipt and explicit `process` results follow normal turn persistence. Export while
a job runs may show the receipt but cannot claim a durable task. After restart or resume, the old
ID is expired. Durable background tasks remain TASK-001 and are not authorized here.

### 6. User cancellation and shutdown

Esc/Ctrl+C retain current active-turn semantics and do not silently cancel background jobs.
Cancellation is explicit through `process cancel` or host/UI control bound to that same operation.
Provider failure does not cancel an already accepted background job.

`/quit`, host shutdown, TUI drop, and unexpected session termination enter the RUNTIME-005 ordered
finalizer path. The supervisor stops admission, requests cancellation for every running job,
waits up to 2 seconds for graceful exit, force-terminates the owned tree, reaps leaders/readers, and
reports remaining job IDs and cleanup errors. The global RUNTIME-005 deadline remains authoritative;
the supervisor cannot extend it or hide an incomplete finalizer.

### 7. Platform process-tree gate

On Unix, ADR-007 already creates a new session/process group with the child as leader. TOOL-024-B
may add one narrowly reviewed `talos-tools::process_boundary` OS-ABI operation that sends SIGTERM
and, after the 2-second grace, SIGKILL to the negative process-group ID. When this ADR is Accepted,
it authorizes only the checked `libc::kill(-pgid, signal)` calls needed for that operation; every
site must reference this ADR, treat `ESRCH` as already exited, surface other errors, and never accept
an unvalidated/non-positive PGID. No other `unsafe` is authorized.

On Windows, ADR-057 does not own descendant cleanup. PowerShell background mode remains fail-closed
until TOOL-024-D has a separately claimed and Accepted Job Object/OS-ABI decision, assigns the child
before it can escape, enables kill-on-close, proves nested-child cleanup, and records dependency and
`unsafe` review. Direct-child `kill`, `taskkill`, shell jobs, and documentation disclaimers are not
acceptable substitutes.

Consequently TOOL-024-B is an implementation-ready Unix slice, TOOL-024-C adds the shared process
tool over that supervisor, and TOOL-024-D owns Windows enablement plus cross-platform acceptance.
Issue #59 cannot close before D.

## Implementation Split

| Slice | Deliverable | Principal areas | Gate |
|---|---|---|---|
| TOOL-024-B | Unix supervisor, explicit start field, bounded capture, group cancel/reap, terminal event | `talos-agent`, `talos-tools`, `talos-core` | A Accepted; TOOL-023-C, RUNTIME-005 and PERM-006-C Complete |
| TOOL-024-C | Session-scoped `process` list/status/read/cancel | `talos-agent`, `talos-tools`, composition roots | B Complete |
| TOOL-024-D1 | Windows Job Object security/ABI decision and implementation | new bounded owner; `talos-tools` process boundary | C Complete; independent Windows/security review |
| TOOL-024-D2 | CLI/TUI projection, docs, real Unix/Windows acceptance | `talos-cli`, `talos-tui`, README/help | D1 Complete |

Each row requires its own owner, iteration, effective claim, implementation PR, exact-head CI,
independent review, merge-time CAS, and completion commit. They may not be collapsed into I188.

## Compatibility And Migration

- Omitted `background` remains foreground; existing configs and provider calls preserve behavior.
- Existing foreground grants remain valid only for foreground execution.
- The `background:` resource namespace is new and has no automatic migration from foreground
  grants. Operators must approve it explicitly.
- `SessionEvent` is already non-exhaustive; consumers must retain their fallback arm. The terminal
  event is additive and must be documented in the SDK change log when implemented.
- No TLOG schema migration occurs in B/C. If durable terminal summaries are later required, create
  a separate session-schema ADR and migration iteration rather than widening this decision.
- Windows users receive an explicit unsupported error until D, never a best-effort leaked process.

## Validation

Before A acceptance, independently verify this ADR and the current-path matrix at the same exact
head. B-D must additionally prove:

- no spawn after Deny/Ask failure, admission close, cap exhaustion, or unsupported shape/platform;
- foreground `always` does not cover the `background:` facet;
- natural-exit/timeout/cancel/shutdown races emit one terminal state/event;
- output remains within caps and cursor expiry is explicit;
- Unix child and grandchild are both terminated and reaped;
- Windows child and grandchild are both terminated by the accepted Job Object boundary;
- completion does not submit a provider request;
- runtime finalizer timeout/error reports are complete and ordered.

## Rejected Alternatives

- Shell `&`, `nohup`, or PowerShell jobs: bypass typed intent, permission, ownership, and cleanup.
- A global job manager/event bus: violates session isolation and ADR-006.
- Reusing a foreground `always` rule: silently broadens lifetime and shutdown risk.
- Emitting a second provider tool result: duplicates one tool-call ID and can trigger unsolicited
  model work.
- Treating direct-child kill as tree cleanup: contradicted by current tests and ADR-057 residuals.
- Using `taskkill` as the Windows primary implementation: host-dependent, race-prone, and not an
  assigned-before-exec ownership primitive.
- Persisting live jobs in B/C: turns an in-process feature into TASK-001 without scheduler/recovery
  semantics.

## Reversal Triggers

Revisit this decision if Talos adopts a durable task runtime, Rust gains safe cross-platform
process-tree ownership, PERM-006-D replaces the reserved resource namespace, job output must survive
restart, or provider protocols gain a standard non-turn background-result channel. Any wider Unix
signal surface or Windows OS-ABI surface requires amendment or a new ADR before code.

## Related

- [TOOL-024](../backlog/active/TOOL-024-background-command-jobs.md)
- [TOOL-024-A / I188](../backlog/active/TOOL-024-A-background-job-lifecycle-spike.md)
- [ADR-007](007-process-hardening-unsafe.md)
- [ADR-039](039-runtime-event-semantic-single-flow.md)
- [ADR-057](057-windows-powershell-process-boundary.md)
- [RUNTIME-005](../backlog/active/RUNTIME-005-bounded-graceful-shutdown.md)
- [PERM-006](../backlog/active/PERM-006-permission-pipeline-convergence.md)
