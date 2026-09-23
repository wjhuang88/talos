# ADR-082: Owned Unix Sandbox Command Cancellation

> Status: Accepted
> Date: 2026-09-23
> Owner: I283 / DESKTOP-001-D5

## Context

I283 requires cancellation of a running tool, not only dismissal of its approval.
The macOS/Linux sandbox currently awaits `Command::output()` with a five-second
timeout. Dropping that future does not establish termination of its descendants.
The Desktop integration tests also exposed an unrelated cwd defect; selecting the
workspace as the command cwd is already corrected locally.

ADR-007 and ADR-060 authorize specific Unix process operations in the tools boundary.
They do not implicitly authorize new sandbox `unsafe` sites or exit observation
without reaping. Killing only the direct child cannot satisfy I283 acceptance.

## Accepted Decision

Implement an owned Unix command supervisor in `talos-sandbox`, used by its existing
macOS and Linux execution paths. Keep the SandboxProvider API and permission policy
unchanged. No new dependency, host `kill` utility, release or unrelated background-job
migration is included.

- Create an isolated session/process group before running the sandbox wrapper;
  stdin is null and stdout/stderr are independently drained.
- A supervisor owns the child, process-group identity and cleanup state. Caller
  cancellation signals that supervisor; it does not discard the sole child owner.
- Observe leader exit without reaping (`waitid` with `WEXITED | WNOHANG | WNOWAIT`).
  Retaining the unreaped leader prevents its numeric identity being recycled before
  the supervisor has completed its last group-signal operation.
- Serialize timeout, caller cancellation, normal output completion and cleanup.
  Terminate the owned group before reaping the leader; never signal a stored group
  ID after reaping. Readers must reach EOF or produce an explicit cleanup failure;
  do not truncate output or convert supervision errors into success.
- No competing `Child::wait`, `try_wait`, child drop/reaper or external wait path
  may reap the leader before the final group signal. `ECHILD` means the ownership
  guarantee was lost: report failure and do not signal a possibly recycled PGID.
  Reaping and permanently disabling subsequent group signals form one serialized
  lifecycle transition.
- On ordinary leader exit, preserve descendant output until EOF or the original
  deadline. Cancel/timeout still applies while descendants hold pipes.
- Report execution/cleanup failure honestly. Application shutdown must keep the
  cleanup owner alive until completion or an explicitly reported global deadline;
  a detached task alone is not proof of cleanup.
- Return an explicit cleanup receipt to the Runtime/host shutdown owner. Desktop
  must await it before destroying its Tokio runtime, and must report unconfirmed
  cleanup instead of projecting a successful stopped state on deadline expiry.

Acceptance of this ADR authorizes the narrow sandbox boundary using
`setsid`, checked negative-PGID SIGTERM/SIGKILL, non-reaping `waitid`, and the
Darwin-only `proc_listpids(PROC_PGRP_ONLY, owned_pgid)` proof described below.
Validate positive owned IDs, handle EINTR, treat ESRCH as absent, propagate other
errors, and document memory/FFI safety at every site. No arbitrary signal numbers,
global process scans, Talos-parent environment mutations or new general unsafe
permission is granted. Existing tools background behavior is not silently changed.

On Darwin only, when group termination returns `EPERM` after the leader has been
observed exited, query the owned PGID with `proc_listpids(PROC_PGRP_ONLY)` into a
two-entry `pid_t` buffer. Accept the zombie-only interpretation only when the
query returns exactly one PID matching the still-unreaped leader, then repeat the
non-reaping ownership observation before reaping. Zero, multiple, malformed,
truncated, or failed results remain cleanup failures. This is a bounded read-only
query for the already-owned group, not a global process scan; deliberate group
escape remains outside this guarantee.

Process-group containment does not by itself contain a malicious descendant that
creates a different session. Platform policy and escape review must explicitly
establish the supported containment guarantee before claiming whole-tree cleanup.
The accepted bounded guarantee excludes deliberate group escape; do not describe
group termination as universal tree containment.

## Validation Before Merge

1. Ready-handshaked shell and grandchild, cancellation before a permitted write:
   both processes terminate, no subsequent write, readers finish and leader reaps.
2. Parent exits before its descendant: delayed output survives normal completion;
   timeout and cancellation still clean up while pipe handles remain inherited.
3. Concurrent/repeated cancel, deadline races, read failure, caller drop and host
   shutdown produce one honest terminal outcome and no late numeric-ID signal.
4. Error-path tests for spawn/session establishment, wait observation and signals;
   unavailable containment fails closed, with no unsandboxed fallback.
5. Platform escape review, pinned locked tests/preflight, and fresh independent
   security/API review of the final implementation. macOS and Linux behavior must
   be verified separately; Windows Job Object tests remain separate evidence.

## Alternatives And Reversal

Direct-child kill leaves descendants. Aborting readers does not cancel blocking
reads and can lose output. Checking whether a PID exists before signaling is still
racy. Moving existing bare-PGID controls without ownership changes does not solve
identity reuse. These alternatives do not meet the acceptance target.

Revise before implementation if non-reaping observation is not portable or ownership
cannot survive Runtime cancellation/shutdown. Deliberate group escape remains outside
the accepted lifecycle guarantee and is an unimplemented security residual.
Do not weaken I283 acceptance to make the existing tests pass.

## Decision Gate

Independent design review found that the existing Linux invocation does not use a
PID namespace and the macOS profile allows process operations. Neither currently
proves that a descendant cannot change its session/process group. Keeping the
leader unreaped solves identity reuse, not intentional group escape.

The maintainer must choose the product guarantee before accepting implementation:

- **Bounded lifecycle guarantee:** cancel and reap the owned group, including
  ordinary shell descendants; explicitly exclude descendants deliberately leaving
  that group. This is a lifecycle correction, not a new malicious-process security
  boundary. Selecting it explicitly amends I283's cancellation acceptance rather
  than silently weakening it. Retain the escape-containment requirement as an
  unimplemented residual under I283/#29, not as completed security work.
- **Enforced descendant containment:** require proof that arbitrary descendants
  cannot outlive cancellation. Investigate Linux PID namespaces/cgroups and an
  enforceable macOS mechanism before implementation approval. Existing process-group
  APIs alone do not meet this guarantee; I283 cannot close that acceptance yet.

Recommendation for this development candidate is the first option, only with
explicit maintainer acceptance of its limitation. Do not interpret silence or
prior general development authorization as acceptance of this scope decision.

## Acceptance Record — 2026-09-23

The maintainer explicitly replied “确认接受” to the bounded lifecycle option and
ADR-082. This accepts the narrow ABI/ownership extension above and amends I283's
cancellation guarantee to ordinary descendants remaining in the owned group.
Independent implementation security/API review and validation remain mandatory.
Deliberate group-escape containment remains unimplemented under I283/#29; acceptance
does not claim that security boundary is delivered. I283 remains incomplete;
I284/I285 are not activated by this decision.

### Darwin supplement acceptance

After the exact `proc_listpids` supplement and its failure-closed conditions were
presented, the maintainer instructed “请继续处理吧”. This authorizes the bounded
Darwin query above; it does not authorize arbitrary process enumeration or expand
the accepted containment guarantee. Implementation review remains mandatory.
