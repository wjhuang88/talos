# I300 Bounded Prepared Stage Extension

Status: Partial local implementation; not published or complete.
Owner: I300 / structured Git authorization repair.
Related decision: [ADR-089](../decisions/089-structured-git-effect-authorization.md).

## Problem

Agent and MCP previously capped prepared invocations at two stages. That is adequate
for sender configuration discovery followed by mutation, but not for local/file
push receivers whose `.git`, `commondir`, configuration includes, hooks and child
programs may introduce new external paths. Reading those paths during the first
stage would bypass admission; treating their eventual discovery as retrospective
authority is forbidden.

## Local Implementation — 2026-10-13

The shared stage history now permits eight distinct admitted stages, counts the
initial and terminal stages, and retains all identities. Eight is a finite
implementation bound, not permission to traverse eight arbitrary external
paths. Agent and MCP share an invocation-wide 600-second wall-clock bound;
their existing permission budgets (300 and 30 seconds by default) now expire
at an absolute deadline shared by all stage admissions. A final admitted Git
child retains its existing 300-second execution limit. The permission deadline
does not truncate an already-admitted child. These internal bounds do not
authorize new transports or a retry after uncertain mutation.

Independent review caught and closed an initial error that reused MCP's
30-second permission deadline for push execution. Core and Agent boundary
tests and the MCP JSON-RPC continuation/replay tests pass. Cumulative profile
budgets now reserve at most 65,536 facets and 16 MiB of resource/description
UTF-8 bytes before cloning. Git discovery and mutation share reservations for
128 Talos-spawned commands and 64 MiB of stdout/stderr capacity. Refusal does
not partially charge a budget; failure or cancellation does not refund it.
These are reserved capacities, not measured output; Git's internal descendants
are not counted. Large submodule layouts can exhaust the command allowance.
Evidence bytes, descendant/helper accounting, the complete concurrency matrix
below and publication review remain open.

## Candidate change

Use one shared, finite stage budget for all prepared runners, with the value
selected through security/API review. Preserve fresh plan identity and fresh
one-use proof at every transition. No Git-specific internal bypass, implicit
scope expansion, or mutable reuse of an already-approved plan is permitted.

The budget counts the initial stage and every continuation, including the final
mutation stage. A continuation returned at the limit is refused; the current
stage's cleanup receipt is still collected and awaited. The counter is owned by
the invocation and cannot reset on re-authorization, error handling, retry or
runner handoff. A stage proof is atomically consumed before executor entry and
is never recoverable after rejection or panic. It binds the invocation, exact
plan identity, facets, input and provenance; every identity seen in the
invocation is retained for replay detection, not only the immediately previous
stage. A continuation must receive ordinary admission for its own immutable
effects; an earlier proof never authorizes a newly discovered scope.

Each runner must reject repeated identity, missing identity, exhausted budget,
unknown proof, replayed proof and cancellation before executing the next stage.
Request replay reservation remains sticky across all continuations and errors.
Cleanup receipts remain owned and awaited across the complete invocation.
The stage budget is in addition to cumulative deadline, output/evidence byte,
helper-count and total authorization-request budgets; adding stages must not
multiply those per-stage limits. Cancellation cannot reset any cumulative
budget or deadline.
Cancellation and stage admission must be serialized: once cancellation wins,
no later proof may be admitted or executed, and an unconsumed proof is void.
The single cleanup owner must survive continuation handoff and enforce
seal → cancel/join → wait; timeout or an unconfirmed receipt is surfaced as a
cleanup failure even when the stage itself errors. Normal completion, protocol
disconnect and stage error must all release stage reservations. Replay
reservations need a bounded lifetime or owner finalizer so a dropped invocation
cannot leak them forever.

The stage ledger is owned by the Agent/MCP runner, not by a boxed plan. Stage
index, invocation token, replay token, cumulative deadline, byte budget and
child/helper count are part of an unforgeable admission context; `plan_identity`
alone is not an authorization or replay token. A `Continue` transition must
transfer the same cleanup owner on every Continue, Complete, error and drop
path. Authorized read-only discovery may execute bounded child commands before
returning `Continue`, as required by ADR-089. Those commands must have finished
with confirmed cleanup; no child may continue running across the transition.
Receiver mutation belongs to the final admitted stage. A partial or uncertain
mutation must terminate with an error describing possibly incomplete effects,
never a retryable continuation or an automatically repeated mutation.

Local/file push should proceed through sender discovery, admitted receiver
layout discovery, receiver configuration discovery, auxiliary Read evidence,
and finally mutation admission. External metadata pointer chains may require
additional stages but must terminate within the same finite budget. All receiver
plans must pass before any destination is mutated. Bare-only execution is not
full local/file support; non-bare and linked-worktree effects remain in scope.

## Required evidence before acceptance

- Independent security/API review of the budget, proof consumption, identity
  checks, request replay, cancellation, cumulative work and receipt ownership.
- Matching Agent and MCP tests for successful multi-stage progression and
  refusal at the budget boundary, including repeated identities and proofs.
- Denial/cancellation at every receiver discovery boundary produces no receiver
  write and no unapproved external read or child execution.
- Fresh final plan binds all newly discovered effects and refuses stale evidence.
- Existing two-stage add/commit/checkout/direct-git push behavior stays covered.
- The concurrency matrix covers cancellation or transport disconnect after
  stage N-1 returns Continue, dropped continuations, old-proof reuse with the
  same or different identity, stage-index rollback/jump, parallel submissions
  where only one proof may be consumed, budget expiry with a child running,
  post-Complete replay, and cleanup Timeout/Unconfirmed precedence.

This proposal does not resolve the separate trace2 contract decision, enable any
new transport, or establish I300 completion.
