# TOOL-024-B: Managed Background Execution Core

> Document status: Active / Claimed through PR #379 merge `48e8ae9b`

| Field | Value |
|---|---|
| Story ID | TOOL-024-B |
| Type | Product / Runtime / Process-Security Story |
| Priority | P0 |
| Status | Active / Claimed through PR #379 merge `48e8ae9b` |
| Parent Epic | [TOOL-024](TOOL-024-background-command-jobs.md) |
| Source | [GitHub Issue #59](https://github.com/wjhuang88/talos/issues/59) |
| Selected Iteration | I222 Active / Claimed through PR #379 merge `48e8ae9b` |
| Depends On | TOOL-024-A/I188, TOOL-023-C, RUNTIME-005 and PERM-006-C/I221 Complete; ADR-060 Accepted |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 Sol mainline Issue #59 session 2026-08-23 |
| Work Slice | Implement only TOOL-024-B/I222 under Accepted ADR-060: Agent/session-owned Unix supervisor; default-false background input for bash and one top-level exec; semantic pre-admission; exact background permission facet; bounded receipt, state, output and terminal event; checked process-group termination/reap; ordinary AppServerSession and Runtime finalizer cleanup. Permission authority is limited to making a resource-less generic Execute Allow degrade to Ask for the reserved `background:` Command namespace, with Deny precedence and focused tests; no public permission schema or PERM-006-D/E behavior. Windows and unsupported/detached shapes fail closed. No process tool, CLI/TUI/Dashboard projection, Windows spawn, persistence, `/auto`, release or later TOOL-024 child. |
| Claimed At | 2026-08-23 |
| Source Issue | #59 |
| Governance Claim PR | #379 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer long-task objective selects Issue #59 and explicitly authorizes the bounded I213/I222-B parallel pair in Issue #366 comment `5386904546`. Claim PR #379 exact head `5f0816aa` passed CI `32650593056`, independent Agent-role claim review `5386970071` and merge-time CAS `5386973729`, then merged as `48e8ae9b`. Implementation still requires fresh exact-head process/permission/unsafe/API review. |
| Implementation PR | Not started |
| Last Updated | 2026-08-24 |
| Handoff / Release Condition | Claim/activation is effective through #379 merge `48e8ae9b`; implementation starts from that merge or later main and remains inside the recorded file/authority boundary. |

## Identity / Goal / Value

Give a live Talos Agent session one bounded Unix background-command supervisor so an explicitly
backgrounded shell or single direct exec returns promptly without orphaning processes, bypassing
permission authority, duplicating provider results or weakening foreground behavior.

## Scope

- One Agent/session-owned `BackgroundJobSupervisor`; no global registry or event bus.
- Default-false `background` input for the Unix shell and one top-level Unix `exec` command.
- Immediate bounded start receipt, stable session-scoped job identity and monotonic lifecycle.
- Existing command permission facets plus one exact `background:` Execute/Command resource facet.
- At most 8 non-terminal and 32 retained terminal jobs per session.
- Terminal-job retention is the ADR-060 capacity policy: keep at most 32 terminal jobs, evict
  oldest-first, and discard the live-only registry at session/process end. B adds no wall-clock TTL;
  the Accepted ADR's deterministic cap supersedes Issue #59's tentative TTL wording.
- One 64-KiB combined stdout/stderr ordered ring per job and explicit cursor truncation metadata.
- Deadline, natural exit, cancellation, shutdown and reader-failure races converge to one terminal
  state and at most one UI-neutral terminal `SessionEvent`.
- Narrow Unix process-group SIGTERM/SIGKILL boundary authorized by ADR-060, with validated positive
  PGID, `ESRCH` handling, error propagation and child/grandchild reap evidence.
- RUNTIME-005 finalizer integration: close job admission, cancel, wait at most two seconds, force
  terminate/reap and report incomplete cleanup without extending the global deadline.
- Agent/session and embedded Runtime integration tests proving the slice is runnable without CLI,
  Dashboard or later `process`-tool projection work.

## Exclusions

- No `process` list/status/read/cancel tool; TOOL-024-C owns that model-facing contract.
- No Windows background spawn or Job Object implementation; unsupported Windows input fails before
  permission grant installation or spawn until separately governed TOOL-024-D1.
- No CLI/TUI lifecycle projection, host controls or manual cross-platform acceptance; TOOL-024-D2
  owns those surfaces.
- No changes to `crates/talos-cli/**`, `crates/talos-dashboard/**`, I213/WEB owners, Dashboard
  implementation, `/auto`, PERM-006-D/E behavior, release, version or publication.
- No persistent jobs, restart survival, automatic provider continuation, PTY/stdin, `exec.steps`,
  pipes or parallel groups.
- No OS-detached or self-daemonizing command. B supports only a leader and descendants that remain
  in the Talos-created Unix process group; known detach syntax/shapes fail semantic admission, and
  B does not claim containment for a child that deliberately creates a new session/process group.
- No new third-party dependency or additional `unsafe` authority beyond ADR-060's checked Unix
  process-group signal operation.

## Dependencies

All technical prerequisites are Complete/Accepted on `main@e1c375e6`. Maintainer authorization
`5386904546` permits the exact I213/I222-B non-overlapping pair. I222 owns no file in I213 PR #372's
17-file production inventory and may not edit `crates/talos-cli/**`, `crates/talos-dashboard/**`,
README files or I213/WEB product authority. Shared derived governance views use union semantics.
Same production-file or authority overlap pauses only the overlapping work and requires new
coordination.

## Decision Links And Constraints

- [ADR-060](../../decisions/060-supervised-background-command-jobs.md) is normative.
- [ADR-007](../../decisions/007-process-hardening-unsafe.md) remains the existing `setsid` boundary;
  ADR-060 alone authorizes the checked negative-PGID signal calls required by this Story.
- [ADR-006](../../decisions/006-event-architecture-boundary.md) forbids a global job bus.
- [ADR-063](../../decisions/063-bounded-runtime-shutdown-finalization.md) and completed RUNTIME-005
  define the authoritative shutdown deadline/finalizer order.
- [ADR-067](../../decisions/067-agent-owned-permission-pipeline.md) keeps evaluation, resolution,
  authorization and final execution gating inside the Agent-owned permission pipeline.
- A foreground Allow/Session grant never authorizes the separate `background:` resource.
- Completion emits no second provider tool result and starts no provider request.

## Uncertainty And Validation Path

Synchronization primitives and internal module placement may vary, but ADR-060 ownership, limits,
states and platform split may not. Before the first implementation push, prove no I213 or CLI
production file is needed. If RUNTIME-005 needs an additive public finalizer seam, document its
compatibility impact and cover it with an external runtime fixture; a breaking change requires a
new ADR and migration plan.

## State / Status Owners

- Story status and acceptance: this file.
- Iteration execution: `docs/iterations/I222-tool024b-managed-background-execution-core.md`.
- Program execution: `docs/tasks/2026-08-23-issue59-supervised-background-jobs.md`.
- Deferred manual/device evidence: [Issue #378](https://github.com/wjhuang88/talos/issues/378)
  and planned cleanup I223.
- Source requirement/discussion: [Issue #59](https://github.com/wjhuang88/talos/issues/59).
- Derived views: `docs/backlog/PRODUCT-BACKLOG.md`, `docs/iterations/README.md`, `docs/BOARD.md`.

## User-Facing Documentation

- Add API/runtime documentation for the additive background field, receipt, live-process-only
  lifetime and Unix-only B boundary.
- Do not advertise complete background-job control until TOOL-024-C/D and Issue #378 acceptance
  are closed.

## Required Reads

- `docs/backlog/active/TOOL-024-background-command-jobs.md`
- `docs/backlog/active/TOOL-024-A-background-job-lifecycle-spike.md`
- `docs/iterations/I188-tool024a-background-job-contract.md`
- `docs/decisions/060-supervised-background-command-jobs.md`
- `docs/reference/I188-BACKGROUND-JOB-CURRENT-PATH.md`
- `docs/backlog/active/RUNTIME-005-C-ordered-finalizer-durable-closure.md`
- `docs/backlog/active/PERM-006-C-agent-owned-permission-pipeline.md`
- `crates/talos-tools/src/bash_tool.rs`
- `crates/talos-tools/src/exec_tool.rs`
- `crates/talos-tools/src/process_boundary.rs`
- `crates/talos-agent/src/tool_execution.rs`
- `crates/talos-agent/src/session.rs`
- `crates/talos-runtime/src/shutdown.rs`
- `crates/talos-core/src/session.rs`
- `crates/talos-permission/src/rule.rs`
- `crates/talos-permission/src/permission_tests.rs`

## Acceptance For Behavior / Technical Work

- Omitted or false `background` preserves existing shell/exec schemas, permissions, output,
  timeout and foreground execution behavior.
- Admitted Unix shell or one top-level exec background input starts one process group and promptly
  returns one bounded receipt with opaque job ID, running state, tool identity and deadline.
- A foreground grant without the exact background grant remains Ask/Deny and executes zero process.
- Windows, missing supervisor, unsupported shape, admission closure, cap exhaustion, Deny, resolver
  failure or stale authorization fails before grant installation or spawn.
- Exit, timeout, cancel, reader failure and shutdown races yield one terminal state/event and reap
  every supported process and reader that remains in the Talos-created process group.
- Output stays within 64 KiB and stale cursors expose truncation and the earliest retained cursor.
- Unix cancel/timeout/shutdown kills and reaps a spawned leader and same-group grandchild; a known
  detach/daemonize shape is rejected before permission/grant installation and spawn.
- Completion without an explicit later submission starts no provider request and persists no second
  `Message::Tool`.
- Focused tests, external runtime fixture, locked workspace checks, Clippy, doctests, release
  preflight, exact-head CI and independent process/permission/unsafe/API review pass.
- Issue #378 row V59-B1 binds the exact implementation head; I222 stays Review until I223 resolves it.

## Residual Destination

TOOL-024-C owns model-readable controls; TOOL-024-D1 owns the Windows Job Object decision and
implementation; TOOL-024-D2 owns CLI/TUI projection and integrated platform acceptance. Stronger
Unix containment for deliberately escaping/self-daemonizing children is outside Issue #59's
supervised-child contract and requires a separate decision/owner. Any public finalizer compatibility
residual receives its own owner before B closeout.

## 2026-08-23 Proposed Activation Checkpoint

Governance PR #379 proposes the TOOL-024-B/I222 atomic claim and activation from
`main@e1c375e6`. Maintainer authorization `5386904546` permits I213 and I222-B to run in parallel
under exact authority/file/CAS boundaries; it supersedes the earlier serial note `5386854800` and
cannot be reused for C/D. I223 remains Planned/Unclaimed. This proposed state has no target-branch
effect before #379 merge and grants no C/D, CLI, Dashboard, Windows, release or `/auto` authority.

## 2026-08-24 Effective Activation Checkpoint

PR #379 exact head `5f0816aa` passed docs-route CI `32650593056`, independent Agent-role claim
review `5386970071` and merge-time CAS `5386973729`, then merged as `48e8ae9b`. TOOL-024-B/I222
is therefore Active/Claimed on `main`. Implementation must start from that merge or later `main`,
preserve the exact I213/I222-B authority and file boundary, and obtain fresh exact-head process,
permission, `unsafe` and public-API review. This activation grants no C/D, CLI, Dashboard, Windows,
release or `/auto` authority.

## 2026-08-24 Permission Namespace Change Control

Post-activation code inspection found that the current permission matcher lets a resource-less
generic Execute Allow match the reserved `background:` Command facet. That would violate ADR-060's
hard foreground-vs-background separation even though the facet text is distinct.

This Story therefore adds one narrow `talos-permission` authority: generic resource-less Execute
Allow must degrade to Ask for `background:` Command resources, while explicit Deny retains
precedence and explicit matching background rules/exact grants remain usable. Focused permission
tests are mandatory. Public permission schema/config, PERM-006-D/E, `/auto`, Dashboard, CLI and
unrelated matching behavior remain excluded. This amendment is ineffective until its governance PR
reaches `main` after exact-head CI, independent permission/security/API review and CAS.
