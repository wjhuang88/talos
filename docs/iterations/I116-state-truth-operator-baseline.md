# Iteration I116: State Truth And Operator Baseline

> Document status: Complete (2026-07-12)
> Published plan date: 2026-07-12
> Planned objective: Make governance state match shipped code and establish a repeatable operator
> smoke/status baseline before selecting more feature work.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: one real `talos` binary smoke packet and read-only status summary prove model,
> session, permission, release/toolchain, and ordered-turn health from a truth-synchronized Board.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| N100 | Four-month trust/productization plan | Planned | Current code and owner docs | Code/iteration/backlog/Board trace matrix |
| N101 | Governance/review closure | Planned | Non-terminal inventory | I085 and I106-I109 receive explicit dispositions |
| N102 | Runtime smoke | Planned | N100-N101 | Repeatable real-binary operator smoke |
| N103 | Read-only diagnostics | Refinement at activation | N100 | Bounded status summary with no secrets |
| N104 | Month-1 closeout | Planned | N100-N103 | Truth-synchronized closeout evidence |

### Scope

- Reconcile delivered I110-I115, SESSION-004, PERF-001, TOOL-020, and HOOK-001 facts.
- Resolve, preserve, or explicitly block I085 and I106-I109 without changing their evidence class.
- Provide a deterministic operator smoke and bounded status surface.

### Non-Goals

- Permission broadening, new session format behavior, new provider behavior, release tagging, or
  retroactive REL-002 qualification.

### Acceptance

- Given a clean checkout, when the operator runs the smoke packet, then version, connect/model,
  session export/resume, permission preflight, and ordered tool-turn checks produce bounded evidence.
- Given governance status, when it is compared with code/iteration evidence, then no delivered item
  remains falsely Planned and no incomplete item is marked Complete.
- Status output never exposes API keys, tokens, raw hidden reasoning, or unrestricted file content.

### Planned Validation

- `./scripts/release_preflight.sh`
- `scripts/validate_project_governance.sh .`
- real `talos` binary smoke packet
- redaction and owner-state trace tests

### Documentation To Update

- Owner docs found stale by N100
- `docs/iterations/README.md`, `docs/BOARD.md`, and relevant README diagnostics

### Risks And Rollback

- Risk: status reconciliation overstates code evidence.
- Rollback: retain Partial/Review/Blocked with the exact missing runtime proof.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-12 | Planning | Published as Month 1 shell; not activated. I085 and I106-I109 dispositions are activation prerequisites. |
| 2026-07-12 | Inventory disposition | I085 remains explicitly Paused because MC107 needs a real terminal `/connect` walkthrough; it is not absorbed or claimed complete. I106-I109 close as Complete with their recorded external-runtime/non-qualifying REL-002 classifications preserved. I018-I020 and I028 remain deferred/blocked by their published dependencies; I081-I083 and I086-I089 remain superseded historical shells; I117-I119 remain Planned and dependency-blocked. No other iteration is Active. |
| 2026-07-12 | Activation | I116 activated. The developer execution owner is `docs/tasks/2026-07-12-developer-trust-productization-long-task.md`. Begin with LT000-LT002; code work starts only after the baseline and isolated MC107 outcome are recorded. |
| 2026-07-12 | LT000 complete | Gate 0 passed: Rust 1.97.0 pinned, Cargo.lock committed, governance validation 0 warnings, branch `feature/i116-state-truth-operator-baseline` created. Planning changes committed on main. |
| 2026-07-12 | LT002 partial | Binary starts correctly in disposable HOME with mock provider (print mode exit 0). TUI cannot initialize in PTY (cursor-position query unsupported). MC107 interactive `/connect` + `/model` walkthrough remains a manual gate; I085 stays Paused per fallback. |
| 2026-07-12 | LT010 complete | State trace matrix created at `docs/reference/I116-STATE-TRACE-2026-07-12.md`. Three owner-state drifts reconciled with code+test evidence: SESSION-004 Ready→Complete, PERF-001 Partial→Complete (Phase 1 also delivered), TOOL-020 Planned→Complete. No status upgraded without evidence. |
| 2026-07-12 | LT011 complete | Operator smoke harness extended: `scripts/talos_smoke.sh` now covers version, validation plan/run, governance status/preview, mock provider, session list, config masking, **permission preflight**, **diagnostics status**, and **ordered tool turn**. 13/13 checks pass with real binary on clean HOME. |
| 2026-07-12 | LT012 complete | Read-only `talos diagnostics status` command implemented in `crates/talos-cli/src/diagnostics.rs`. Reports release/toolchain, session format (tlog/jsonl), workspace trust state, active iterations, and residual gates without credential values. 4 redaction/structure tests pass. JSON output supported. |
| 2026-07-12 | LT013 closeout | Release preflight passed, governance validation 0 warnings, `git diff --check` clean, `talos_smoke.sh` 13/13 pass. Month-1 owners synchronized: SESSION-004/PERF-001/TOOL-020 backlog updated, Board reconciled, state trace matrix published. I116 ready for Complete. |
