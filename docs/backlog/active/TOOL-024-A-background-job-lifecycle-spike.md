# TOOL-024-A: Background Job Lifecycle And Permission Contract Spike

**Status**: Review (2026-08-14; implementation PR #228)
**Priority**: P1
**Type**: Technical / Security Spike
**Parent Epic**: TOOL-024
**Depends on**: None technically; must respect the repository's one-active-iteration rule.
**Selected Iteration**: I188

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline implementation session 2026-08-14 |
| Work Slice | Implement only TOOL-024-A / I188: characterize current command execution ownership and produce the background-job lifecycle, permission, bounded-output, cancellation/shutdown, result-routing, process-control and cross-platform cleanup decision plus an implementation split. No production spawn, tool/API, permission-policy, TUI, persistence, runtime, dependency, unsafe, Job Object, PTY or TOOL-024-B/C/D change. |
| Claimed At | 2026-08-11 |
| Source Issue | #59 |
| Governance Claim PR | #196 |
| Authorization Mode | Independent review |
| Authorization Evidence | Claim PR #196 merged to `main` as `02a3558894a13204a28a48907fa39ca79a420d70`; its final claim head `a5e9ffce241adc2e3646b5925c51f22694bd4a09` passed CI run `31555885775`. The decision implementation still requires fresh independent security review on its own exact head. |
| Implementation PR | #228 |
| Last Updated | 2026-08-14 |
| Handoff / Release Condition | Obtain independent process/permission security review and green CI on PR #228's final exact head, repeat merge-time CAS, then merge before accepting ADR-060 or activating production work. |

## Goal / Value

Produce the explicit lifecycle and permission contract required before Talos can run a shell or
argv command in the background without blocking the interactive conversation and return its final
result to that conversation safely.

## Scope

- Trace the current `bash`, future Windows PowerShell, and `exec` execution paths from tool input
  through permission, spawn, result delivery, TUI projection, session persistence, cancellation,
  and shutdown.
- Write an ADR that selects or rejects a Talos-owned `BackgroundJobId`, typed background mode,
  supervisor ownership location, terminal-state vocabulary, bounded-output policy, exactly-once
  result-delivery strategy, and in-process-only lifecycle.
- Decide and document the approval contract: background intent must be distinguishable from a
  foreground call, and existing foreground `always` approvals must not silently cover it.
- Decide the result routing contract: append a user-visible identified tool result to the current
  conversation, persist only the completed result through existing session semantics if selected,
  and do not auto-submit it to the model in the first implementation.
- Define the model-readable `process` read/status/list/cancel tool contract, including stable job
  identity, ordered output cursors, bounded retention and terminal-result idempotency.
- Define cancellation and shutdown behavior, including known Unix/Windows process-tree residuals
  and whether a safe first slice must reject commands likely to spawn unmanaged descendants.
- Produce an implementation split with affected crates, test seams, migration impact, and manual
  Unix/Windows acceptance matrix.

## Exclusions

- No production background spawning, new tool input fields, public protocol changes, persistence
  schema changes, TUI controls, or permission-policy behavior changes.
- No `unsafe`, new dependencies, Job Objects, durable task scheduler, restart survival, remote
  worker, automatic model continuation, or release work.
- No resolution of TOOL-023-C itself; this Spike consumes its stated Windows boundary.

## Required Decisions

The ADR must answer all of the following before TOOL-024-B becomes Ready:

1. Which crate owns the supervisor and how it receives a cancellation token without creating a
   global message bus.
2. The exact typed input shape for foreground versus background execution and whether tool names
   remain stable on each platform.
3. The permission resource/facet distinction for background execution and the interaction with
   session-scoped `always` approvals.
4. The job terminal states, result event type/identity, output byte/line limits, truncation marker,
   and exactly-once guarantees across errors and late completions.
5. The meaning of Esc, Ctrl+C, `/quit`, TUI Drop, provider failure, session export, and resume while
   jobs are running.
6. The first-slice Unix/Windows child/process-tree cleanup guarantee and any explicit residual.
7. How terminal results enter the transcript without automatically triggering a provider request.
8. How the `process` tool exposes ordered incremental and terminal output without duplicating,
   losing or leaking output across sessions.

## Affected Areas

- `crates/talos-core` / `crates/talos-conversation`: only if a new event or public tool contract is
  selected; assess semver/migration impact first.
- `crates/talos-agent`: turn-loop ownership and tool-result sequencing.
- `crates/talos-tools`: `BashTool`, `ExecTool`, output capture, platform invocation.
- `crates/talos-permission`: approval facets and scoped allow semantics.
- `crates/talos-session`: completed-result persistence/resume boundary.
- `crates/talos-tui` and `crates/talos-cli`: live job/result projection and lifecycle controls.

## Acceptance

- Given the current foreground execution code, when the Spike traces it, then every ownership and
  result-routing handoff is documented with its crate boundary.
- Given a proposed background command, when the contract is reviewed, then it defines one owner,
  one cancellation/shutdown path, a bounded-output policy, and exactly one terminal-result delivery
  path.
- Given an existing foreground approval or `always` grant, when background execution is proposed,
  then the ADR explicitly prevents silent reuse unless a separately approved background scope exists.
- Given a background job completion, when it is returned to the session, then the ADR states that
  the first implementation surfaces it to the conversation without an automatic model request.
- A new ADR is Proposed and cross-linked by TOOL-024 and this Spike; no production code changes are
  made by the Spike.
- The parent/child backlog statuses and Board/program disposition remain synchronized; no new
  iteration is activated while another governed iteration is Active or Review.

## Validation

```bash
scripts/validate_project_governance.sh .
git diff --check
```

## State / Status Owners

- Parent scope/status: TOOL-024.
- Spike scope/status and ADR link: this document.
- Active implementation selection: iteration owner documents and `docs/BOARD.md`; this requirement
  does not change them.

## User-Facing Documentation

The Spike itself changes no runtime behavior. TOOL-024-B/C/D must update README EN/zh-CN, help/tool
schema documentation, and the user-visible cancellation/status guidance if implementation proceeds.

## Decision Output Under Review

- [ADR-060](../../decisions/060-supervised-background-command-jobs.md) proposes one session-owned,
  bounded supervisor contract and remains Proposed pending independent exact-head review.
- [Current-path characterization](../../reference/I188-BACKGROUND-JOB-CURRENT-PATH.md) records the
  foreground ownership, output, permission, cancellation, persistence, and shutdown evidence at
  baseline `556b5a43`.
- TOOL-024-B is narrowed to an implementation-ready Unix slice after A acceptance plus RUNTIME-005
  and PERM-006-C completion. Windows background spawn remains fail-closed until TOOL-024-D accepts
  and implements a Job Object/OS-ABI boundary.
