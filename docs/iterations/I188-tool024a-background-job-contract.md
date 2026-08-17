# Iteration I188: TOOL-024-A Background Job Lifecycle Contract

> Document status: Complete
> Published plan date: 2026-08-11
> Planned objective: decide the ownership, permission, cancellation, bounded-output, terminal-result and cross-platform cleanup contract required before supervised background command jobs can be implemented.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a reviewed Proposed ADR plus a runnable current-path characterization that makes TOOL-024-B implementation-ready without spawning any background process.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline implementation session 2026-08-14 |
| Work Slice | Implement only TOOL-024-A / I188: characterize current command execution ownership and produce the background-job lifecycle, permission, bounded-output, cancellation/shutdown, result-routing, process-control and cross-platform cleanup decision plus an implementation split. No production spawn, tool/API, permission-policy, TUI, persistence, runtime, dependency, unsafe, Job Object, PTY or TOOL-024-B/C/D change. |
| Claimed At | 2026-08-11 |
| Source Issue | #59 |
| Governance Claim PR | #196 |
| Authorization Mode | Independent review |
| Authorization Evidence | Claim PR #196 merged as `02a3558894a13204a28a48907fa39ca79a420d70`. Decision head `d7d4fe7ae4cc67e452be2ee8ab1c9aab6ef0f803` passed CI `31995198205`, independent security review `5312482823`, and merge-time CAS before PR #228 merged as `1db1211e2fedeab277db366c3c76db0239691732`. |
| Implementation PR | #228 |
| Last Updated | 2026-08-17 |
| Handoff / Release Condition | None. Production children remain separately blocked and require their own effective claims. |

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
- I189/PERM-006-A remains Planned/Claimed and is not activated by I188.
- The I185-I187 bullets above preserve the 2026-08-11 selection baseline. At 2026-08-14
  activation, I185-I187 and I193 were Complete and no governed iteration was Active or Review.
- Security-sensitive process and permission design requires independent exact-head review before
  ADR acceptance and implementation merge.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-11 | Selection | TOOL-024-A selected as a Planned decision Spike only. PR #196 proposes the claim but remains ineffective before merge; no implementation branch is authorized, and activation waits for the recorded non-terminal Review disposition. |
| 2026-08-14 | Claim reconciliation | PR #196 claim merge `02a35588` is an ancestor of current `main`; final claim head `a5e9ffce` passed CI `31555885775`. No overlapping Active/Review iteration or implementation PR was found. |
| 2026-08-14 | Activation | User directed the mainline session to clear Issue #59 implementation blockers. I188 activated from exact `origin/main` `556b5a43` in isolated branch `feat/runtime-I188-background-job-contract`; no Rust, Cargo, persistence, dependency, unsafe, Desktop, or Dashboard scope is authorized. |
| 2026-08-14 | Decision | Proposed ADR-060 selects a session-owned bounded supervisor, explicit background permission resource, live-only terminal event and Unix-first process-group slice. Windows remains fail-closed pending D's separate Job Object gate. |
| 2026-08-14 | Review handoff | Decision implementation commit `245eddeb` pushed and PR #228 opened against `main`; iteration moved to Review. No self-review or ADR acceptance is recorded. |
| 2026-08-17 | Independent review and merge | Exact decision head `d7d4fe7a` passed CI `31995198205` and independent process/permission security review `5312482823`; merge-time CAS preserved that head and PR #228 merged as `1db1211e`. ADR-060 is accepted without production implementation. |

## Verification Evidence

- Effective claim merge: `02a3558894a13204a28a48907fa39ca79a420d70`.
- Claim final head `a5e9ffce241adc2e3646b5925c51f22694bd4a09`; CI run `31555885775` passed.
- Decision artifacts: `docs/decisions/060-supervised-background-command-jobs.md` and
  `docs/reference/I188-BACKGROUND-JOB-CURRENT-PATH.md`.
- Implementation commit `245eddeb`; PR #228.
- Exact decision head `d7d4fe7ae4cc67e452be2ee8ab1c9aab6ef0f803`; CI `31995198205`.
- Independent process/permission security review `5312482823`; PR #228 merge
  `1db1211e2fedeab277db366c3c76db0239691732` preserved the reviewed head through merge-time CAS.

## Completion Evidence

- Completion Commit: `245eddebae762d1d0c7ee796baea50d0bb080bd5`.
- The cited implementation evidence predates this closeout and contains ADR-060 plus the current-path
  characterization. This status-only closeout does not certify itself.

## Variance And Residuals

- TOOL-024-B remains blocked by accepted TOOL-024-A, completed RUNTIME-005, completed PERM-006-C and completed TOOL-023-C. Its first production scope is Unix-only.
- Windows spawn remains blocked until TOOL-024-D owns and accepts a separate Job Object/OS-ABI
  decision; direct-child cleanup cannot satisfy the Epic invariant.
- TOOL-024-C/D and Issue #59 remain open after this Spike.

## Retrospective

- The initial issue framing conflated PowerShell availability with descendant ownership. ADR-057
  proves only command identity/direct-child timeout; background Windows execution must remain
  fail-closed until Job Object ownership is independently reviewed.
- Keeping the decision-only slice separate prevented acceptance of a lifecycle contract from being
  mistaken for production spawn authority.
