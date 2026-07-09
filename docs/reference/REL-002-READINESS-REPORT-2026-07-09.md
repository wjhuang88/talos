# REL-002 Readiness Report — 2026-07-09

**Date**: 2026-07-09
**Plan**: `docs/tasks/2026-07-08-four-month-talos-self-bootstrap-plan.md` (I106-I109)
**Verdict**: **NO-GO for v1.0.0**
**Runtime**: glm-5.2 via zai-coding-plan (external)

## Executive Summary

The four-month Talos-primary self-bootstrap plan (I106-I109) has completed all four monthly
iterations. Every session was executed by an external runtime (glm-5.2 via zai-coding-plan), not
the `talos` binary. Per REL-002 acceptance criterion 7, none of these sessions qualify as
self-bootstrap evidence. The plan produced useful artifacts — a request-dispatch timeout fix, a
channel topology audit, an evidence schema, a smoke harness, and a governance mutation rehearsal —
but the core self-bootstrap capability was not demonstrated.

**The verdict is NO-GO for v1.0.0.** Talos remains pre-1.0. No `v1.0.0` tag, release, publish, or
external trial is authorized.

## Acceptance Criteria Audit

For each of the 8 REL-002 acceptance criteria:

### Criterion 1: Talos can run a complete development iteration as primary runtime
**Status**: UNMET
**Evidence**: No session in I106-I109 used the `talos` binary as the primary executor. I107 SBT111 implemented the #18 dispatch timeout fix, but the executor was glm-5.2 (external), not Talos. The 2 partial sessions from 2026-07-06 (SB100-SB130, SSP100-SSP150) used deepseek-v4-pro through Talos, but both had Codex remediation or incomplete push/Board sync, keeping them partial rather than fully qualifying.

### Criterion 2: Talos can read governance state, select work, preserve integrity
**Status**: PARTIAL
**Evidence**: I106 SBT103 proved the governance mutation path (preview → write → validate → rollback) works through the `talos` binary. I108 SBT121 proved Talos can read and audit architecture state. However, no session demonstrated the full governance selection → work → sync loop with Talos as the autonomous primary executor.

### Criterion 3: Talos can implement code changes with permission-gated tools and passing validation
**Status**: PARTIAL
**Evidence**: I107 SBT111 delivered a real code change (`dispatch_timeout_secs`) with provider tests, and the follow-up audit added agent/CLI bridge tests for terminal processing cleanup. The SB100-SB130 and SSP100-SSP150 pilots also delivered code changes through Talos. However, all I106-I109 code changes were executed by glm-5.2 external, not Talos. The partial sessions had Codex remediation.

### Criterion 4: Talos can perform architecture-risk classification
**Status**: PARTIAL
**Evidence**: I108 SBT120-SBT121 performed the ARCH-032 Single Data Flow Audit with correct risk classification (audit-only, no code changes, no permission/sandbox/dependency gates). The audit was factually correct and traceable to source. However, the executor was external (glm-5.2), not Talos.

### Criterion 5: Talos can update README/user docs, backlog, iterations, decisions, board
**Status**: PARTIAL
**Evidence**: I106 SBT103, I107 SBT112, and I108 SBT122 all performed owner-doc updates in the correct owner-first order. Governance validation passed with 0 warnings in all sessions. However, the doc edits were performed by glm-5.2 external, not Talos.

### Criterion 6: At least 3 non-trivial Talos-on-Talos sessions (1 arch-sensitive + 1 feature)
**Status**: UNMET
**Evidence**: Zero fully qualifying sessions exist. Two partial sessions (SB100-SB130, SSP100-SSP150) used Talos (deepseek-v4-pro) as primary but had Codex remediation or incomplete push. All I106-I109 sessions were external-runtime primary. The plan needed at least 3 fully qualifying sessions; it has 0.

### Criterion 7: Codex/external agent is NOT the primary executor
**Status**: UNMET
**Evidence**: Every I106-I109 session was executed by glm-5.2 via zai-coding-plan. This is explicitly an external runtime, not the `talos` binary. The plan's own rules (§Operating Rules) require: "If external Codex or another runtime edits files, mark the affected session partial or non-qualifying in REL-002." All I106-I109 sessions are non-qualifying.

### Criterion 8: Final v1.0.0 release checklist names all remaining gaps
**Status**: MET (this report)
**Evidence**: This report serves as the final release checklist. All remaining gaps are named below.

## Session Evidence Summary

| Session | Runtime | Qualification | Key Artifact |
|---|---|---|---|
| I106 SBT100-SBT104 | glm-5.2 external | Non-qualifying | Evidence schema, smoke harness, governance rehearsal |
| I107 SBT110-SBT113 | glm-5.2 external | Non-qualifying | #18 request-dispatch timeout fix (dispatch_timeout_secs) |
| I108 SBT120-SBT123 | glm-5.2 external | Non-qualifying | ARCH-032 Single Data Flow Audit (ADR-006 compliant) |
| I109 SBT130-SBT133 | glm-5.2 external | Non-qualifying | This readiness report |
| SB100-SB130 (2026-07-06) | Talos (deepseek-v4-pro) | **Partial** | 4 code changes, Codex remediation |
| SSP100-SSP150 (2026-07-06) | Talos (deepseek-v4-pro) | **Partial** | 5 commits, push/Board not performed |

**Fully qualifying sessions: 0. Partial sessions: 2. Non-qualifying sessions: all others.**

## What Was Achieved

Despite the NO-GO verdict, the four-month plan produced concrete value:

1. **Request-dispatch timeout (#18)**: The #18 issue-audit residual is fixed. `dispatch_timeout_secs` (default 60s) bounds the `send().await` phase independently from stream timeouts. 4 provider tests plus 2 agent/CLI bridge tests cover dispatch timeout propagation and terminal `is_processing=false`.
2. **Channel topology audit (ARCH-032)**: All 12 src/ directories audited. Zero broadcast channels. All paths ADR-006 compliant. ARCHITECTURE.md updated with factual current-state diagrams.
3. **Evidence/control plane (I106)**: Repeatable smoke harness (`scripts/talos_smoke.sh`), evidence schema with Qualifying/Partial/Non-qualifying rubric, governance mutation path with rollback.
4. **Partial self-bootstrap evidence**: SB100-SB130 and SSP100-SSP150 proved Talos (deepseek-v4-pro) CAN act as primary executor for bounded code changes — the gap is consistency and completeness, not capability absence.

## Remaining Gaps For v1.0.0

| Gap | Priority | Next Owner |
|---|---|---|
| No fully qualifying Talos-primary session exists | P0 | Next self-bootstrap attempt must use the `talos` binary as primary executor for planning, implementation, validation, docs, commit, and push — with no external runtime code edits |
| Only 2 partial sessions (need 3+ fully qualifying) | P0 | Need at least 1 more session where Talos completes the full loop without Codex remediation |
| No architecture-sensitive session by Talos as primary | P0 | ARCH-032 was audit-only and external; need a Talos-primary architecture-sensitive change |
| Push and Board sync not performed in partial sessions | P1 | Talos-primary sessions must include push authorization and full Board sync |
| #39 dashboard transient notification still open | P2 | TUI-028 corrective queue residual |
| #24/#31 visual evidence gaps | P2 | TUI-028 runtime/visual evidence needed |
| TUI-029 thinking history decision | P2 | ADR-034/TUI-020 policy decision required before implementation |

## Recommendation

The next self-bootstrap attempt should:

1. Use the `talos` binary with a capable configured provider (e.g., deepseek-v4-pro or claude-sonnet-4) as the **sole primary executor** — no external runtime code edits.
2. Start with a bounded, low-risk packet (similar to I107's #18 fix) to prove the full loop.
3. Include push authorization and full Board sync in the execution contract.
4. Record evidence using the I106 evidence schema and smoke harness.
5. Aim for at least 3 fully qualifying sessions before claiming v1.0.0.

## No Release Action

This report does not authorize:
- `v1.0.0` tag, release, publish, or deployment
- External trial invitations
- Permission-default, sandbox, credential, or storage-default changes
- Any claim that partial or external-runtime sessions satisfy REL-002

## Related Documents

- `docs/tasks/2026-07-08-four-month-talos-self-bootstrap-plan.md` — the four-month plan
- `docs/iterations/I106-self-bootstrap-control-plane.md` — Month 1
- `docs/iterations/I107-talos-primary-feature-polish.md` — Month 2
- `docs/iterations/I108-architecture-sensitive-self-bootstrap.md` — Month 3
- `docs/iterations/I109-rel002-self-bootstrap-closeout.md` — Month 4 (this report's iteration)
- `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md` — the release gate
- `docs/reference/REL-002-READINESS-REPORT-2026-07-04.md` — previous readiness report
