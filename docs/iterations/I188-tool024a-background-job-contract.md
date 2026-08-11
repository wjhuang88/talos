# Iteration I188: TOOL-024-A Background Job Lifecycle Contract

> Document status: Planned
> Published plan date: 2026-08-11
> Planned objective: decide the ownership, permission, cancellation, bounded-output, terminal-result and cross-platform cleanup contract required before supervised background command jobs can be implemented.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a reviewed Proposed ADR plus a runnable current-path characterization that makes TOOL-024-B implementation-ready without spawning any background process.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #59 |
| Governance Claim PR | Pending |
| Authorization Mode | Independent review |
| Authorization Evidence | Pending exact-head independent security review; this draft has no ownership effect. |
| Implementation PR | Not started |
| Last Updated | 2026-08-11 |
| Handoff / Release Condition | Finalize and merge the independently reviewed claim before implementation; activation also waits until no other governed iteration remains Active or Review. |

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| TOOL-024-A | TOOL-024 / Issue #59 | Ready | No technical prerequisite; process/permission security review and iteration coordination apply | Proposed ADR and current-path matrix for one supervised, bounded, session-owned background-job contract |

### Scope

- Trace foreground `exec`, Unix shell and Windows PowerShell from typed input through permission, spawn, output, cancellation, shutdown, result delivery and session projection.
- Decide one runtime/session-owned supervisor, stable job identity, typed background intent, monotonic terminal states, bounded ordered output and exact-once result delivery.
- Decide the separate permission facet and grant semantics for background start and job control without changing current policy behavior.
- Define model-readable `process` read/status/list/cancel semantics, session ownership, cursor/eviction behavior and no-auto-model-continuation routing.
- Record Unix process-group and Windows descendant-cleanup guarantees, explicit first-slice residuals and the RUNTIME-005/PERM-006 dependencies for production work.

### Non-Goals

- No production spawn, tool input, process manager, permission-policy, TUI, protocol, persistence-schema or runtime shutdown change.
- No `unsafe`, new dependency, Windows Job Object implementation, PTY, detached daemon, restart survival, remote worker or automatic provider continuation.
- No TOOL-024-B/C/D, RUNTIME-005, PERM-006 or TOOL-023 behavior implementation.

### Acceptance

- Given the current foreground command paths, when the characterization is reviewed, then every permission, spawn, output, cancellation, shutdown and result-routing owner is mapped to an exact crate/source boundary.
- Given a proposed background start, when the ADR is reviewed, then foreground approval cannot silently authorize background intent and Deny/fail-closed behavior remains authoritative.
- Given output, cancellation and natural-exit races, when the contract is applied, then one session-owned supervisor produces bounded ordered reads and exactly one terminal state/result.
- Given Unix and Windows descendant cleanup differ, when the first implementation slice is defined, then guarantees and residuals are explicit rather than inferred from direct-child timeout behavior.
- A Proposed ADR and implementation split are published with no production code changes.

### Planned Validation

- Source-path characterization against `talos-tools`, `talos-agent`, `talos-permission`, `talos-session` and runtime composition.
- Deterministic contract fixtures or read-only probes for current foreground timeout/cancellation/output boundaries.
- `scripts/validate_project_governance.sh .`
- `bash scripts/validate_collaboration_claims.sh .`
- `git diff --check`
- Exact-head CI and independent security review before claim merge and again before ADR acceptance.

### Documentation To Update

- `docs/backlog/active/TOOL-024-A-background-job-lifecycle-spike.md`
- `docs/backlog/active/TOOL-024-background-command-jobs.md`
- a new ADR and current-path reference matrix during implementation
- `docs/iterations/README.md`, `docs/backlog/PRODUCT-BACKLOG.md` and `docs/BOARD.md`
- README EN/zh-CN remain unchanged because this Spike adds no user-visible behavior.

### Risks And Rollback

- Risk: an underspecified descendant-cleanup or grant contract could authorize unmanaged processes or permission broadening.
- Risk: treating late job output as an automatic model input could create unsolicited work and token spend.
- Rollback: keep TOOL-024-B blocked and reject the ADR; no production behavior or durable data changes in this Spike.

## Non-Terminal Coordination Record

- I185 remains Planned under its separate SQLite validator claim and PR #191.
- I186/TUI-046-B remains separately owned by its claim/implementation chain and PR #193.
- I187/SESSION-008-A remains Review in PR #195; I188 must not activate while that Review remains open.
- I159-I162 remain Blocked under their existing gates; I164 remains Paused; none overlaps this decision-only Work Slice.
- This claim may be prepared for batched review, but security-sensitive process and permission design requires independent review before claim merge and before ADR acceptance.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-11 | Selection | TOOL-024-A selected as a Planned decision Spike only. The draft claim is ineffective, no implementation branch is authorized, and activation waits for the recorded non-terminal Review disposition. |

## Verification Evidence

- Pending finalized claim head.

## Completion Evidence

- No completion evidence. A status-only commit cannot certify this iteration.

## Variance And Residuals

- TOOL-024-B remains blocked by accepted TOOL-024-A, completed RUNTIME-005, completed PERM-006-C and completed TOOL-023-C.
- TOOL-024-C/D and Issue #59 remain open after this Spike.

## Retrospective

- Pending activation and execution.
