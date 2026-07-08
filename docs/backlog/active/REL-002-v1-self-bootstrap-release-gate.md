# REL-002: v1.0 Self-Bootstrap Release Gate

**Status**: Planned — not ready for v1.0
**Priority**: P1
**Created**: 2026-06-28
**Source**: User-defined v1.0 target marker
**Depends on**: Embeddable runtime maturity; built-in governance reliability; tool/runtime
coverage sufficient for Talos development work

## Goal Marker

Talos reaches `1.0` only when the project can perform **100% self-bootstrap development**:
Talos itself, not Codex as the primary development executor, can plan, implement, verify, document,
and prepare its own changes through the project's normal governance and safety gates.

The current development process still relies on Codex. Therefore Talos remains pre-1.0 even when
individual product capabilities are useful and releaseable.

I093 activation note (2026-07-04): selected for readiness audit only. This does not authorize a
`v1.0.0` claim, tag, publish, or release action. Codex-primary work in the 2026-07-04 direct-owner
track remains non-qualifying for REL-002 unless a later Talos-primary rehearsal explicitly proves
otherwise.

I093 A13 result (2026-07-04): `docs/reference/REL-002-READINESS-REPORT-2026-07-04.md` updates the
runtime/governance/architecture readiness audit. Verdict remains not ready for `v1.0.0`; the
minimum next packet is a controlled Talos-primary rehearsal.

I093 A14 result (2026-07-04): `docs/tasks/2026-07-04-self-bootstrap-rehearsal-i093-a14-nonqualification.md`
records the attempted evidence packet as non-qualifying. Talos proved only the local `talos 0.2.2`
CLI version surface; Codex remained the primary executor for planning, editing, validation
orchestration, owner-doc sync, commit, and push.

I095 activation note (2026-07-04): selected as a prerequisite narrowing pass for runtime
validation evidence. This is still non-qualifying for REL-002 by itself; Codex remains the primary
executor for this unattended task unless a later I097 Talos-primary rehearsal proves otherwise.

I095 result (2026-07-04): `talos validate run` now executes built-in allowlisted profiles and
records command, exit status, stdout/stderr summaries, and the allowlisted-profile permission
decision. This closes the validation-evidence mechanism gap, but REL-002 remains No-go until
Talos-primary sessions use it while Codex is not the primary executor.

I096 result (2026-07-04): `talos governance iteration-record preview/write` now provides a narrow
owner-doc mutation gate with explicit preview, `--confirm-preview`, post-write governance
validation, and rollback on validation failure. This narrows owner-doc sync risk, but the I096
implementation remains Codex-primary and is not a qualifying self-bootstrap session.

I097 activation note (2026-07-04): selected for a documentation-only controlled self-bootstrap
rehearsal after I095/I096 narrowed validation and owner-doc mutation evidence gaps. If Codex
remains the primary executor, I097 must close as non-qualifying evidence.

I097 result (2026-07-04): `docs/tasks/2026-07-04-self-bootstrap-rehearsal-i097-b9-nonqualification.md`
records the controlled rehearsal as non-qualifying. Talos executed allowlisted governance
validation and wrote one bounded owner-doc execution row, but Codex remained primary for planning,
evidence interpretation, docs editing, broader validation orchestration, commit, and push.

I094-I097 closeout (2026-07-04): the direct high-risk execution set is complete and
`docs/reference/I094-I097-HIGH-RISK-GIX-RUNTIME-GOVERNANCE-CLOSEOUT-2026-07-04.md` preserves the
No-go release posture. No `v1.0.0` claim, tag, publish, release, or permission-default change
occurred.

I098-I101 closeout (2026-07-06): `docs/reference/I098-I101-AUTONOMY-PERMISSION-RUNTIME-CLOSEOUT-2026-07-06.md`
records the autonomy/permission/runtime hardening track as useful but non-qualifying REL-002
evidence. Talos gained permission preflight, structured exec, internal governance validation,
model browser closeout, and continued `gix` tracking, but Codex remained primary for planning,
editing, validation orchestration, evidence interpretation, commit, and push.

I102-I105 closeout (2026-07-08): `docs/iterations/I105-trial-readiness-closeout.md` records the
four-month developer operating plan as useful controlled-local-trial evidence but
non-qualifying for REL-002. The work closed provider runtime reliability, first-run setup,
long-session stability, trial docs, smoke evidence, and a GO/NO-GO report, but the execution was
external-runtime primary (OpenCode/minimax-m3 plus Codex architecture review), not Talos-primary.
Therefore `v1.0.0` remains No-go.

I106-I109 planning note (2026-07-08): `docs/tasks/2026-07-08-four-month-talos-self-bootstrap-plan.md`
defines the next four-month attempt as Talos-primary by construction. The plan starts with an
evidence/control-plane month, then requires one user-facing feature or polish session, one
architecture-sensitive session, and a final REL-002 closeout. It does not authorize a `v1.0.0`
claim, tag, release, publish, permission-boundary change, sandbox change, dependency addition,
credential change, or session-storage default migration.

Maintainer correction (2026-07-04): self-bootstrap validation evidence must not assume Talos is a
Rust-only agent. `VALIDATION-001` records the requirement for a language-neutral internal
validation service. Cargo-based checks may remain a host-tool adapter for this repository, but they
cannot define the generic runtime validation model.

## Problem

Pre-1.0 releases can validate useful slices, but they do not prove that Talos has become the agent
runtime it is trying to build. A 1.0 label would be misleading unless Talos can sustain its own
development loop with the same rigor currently provided by Codex-assisted work:

- requirement intake and decomposition;
- architecture and permission-boundary review;
- implementation and refactoring;
- tests, lint, governance validation, and runtime smoke checks;
- documentation and backlog synchronization;
- release-readiness preparation.

## Scope

Define the release gate, evidence, and residual work needed before a future `v1.0.0` tag:

- Track self-bootstrap capability as a release milestone, not a single feature.
- Identify which Talos subsystems must be reliable enough for self-development.
- Require evidence from real Talos-on-Talos development sessions.
- Keep current pre-1.0 releases honest: useful product releases are allowed, but they do not imply
  1.0 readiness.

## Non-Goals

- No immediate `v1.0.0` planning iteration.
- No claim that Codex must be removed from all historical or fallback use.
- No lowering of validation, governance, permission, release, or architecture gates to make
  self-bootstrap easier.
- No automatic release tag or publishing action.

## Acceptance Criteria

- [ ] Talos can run a complete development iteration on this repository using Talos as the primary
      agent runtime.
- [ ] Talos can read current governance state, select work, and preserve iteration/backlog/board
      integrity without external prompt-only governance.
- [ ] Talos can implement code changes with permission-gated tools and produce passing validation
      evidence.
- [ ] Talos can perform architecture-risk classification and route high-risk work through the
      correct gates.
- [ ] Talos can update README/user docs, backlog, iterations, decisions, and board state after a
      real change.
- [ ] At least three non-trivial Talos-on-Talos development sessions are recorded, including one
      architecture-sensitive change and one user-facing feature or polish change.
- [ ] Codex is not the primary executor for the recorded qualifying sessions; any external agent
      assistance is explicitly labeled as fallback/review.
- [ ] A final `v1.0.0` release checklist names all remaining gaps or confirms none remain.

## Evidence To Record

Each qualifying self-bootstrap session must record:

- work item and owner document;
- runtime used;
- commands/tests run;
- files changed;
- governance synchronization evidence;
- residual work;
- whether external agent assistance was used and why.

## Rehearsal Evidence

| Date | Plan Item | Record | REL-002 Qualification |
|---|---|---|---|
| 2026-07-02 | T123 | `docs/tasks/2026-07-02-self-bootstrap-rehearsal-t123-todo-views.md` | Does not qualify. Talos generated a read-only validation plan, but Codex remained the primary executor for implementation, validation execution, docs, git, push, and issue sync. |
| 2026-07-02 | T132 | `docs/tasks/2026-07-02-self-bootstrap-rehearsal-t132-architecture-decision.md` | Does not qualify. Talos generated workspace/governance validation plans for an architecture-sensitive ADR slice, but Codex remained the primary executor and the >60% autonomous coverage target was missed. |
| 2026-07-04 | I093-A14 | `docs/tasks/2026-07-04-self-bootstrap-rehearsal-i093-a14-nonqualification.md` | Does not qualify. Talos only proved `talos 0.2.2` CLI availability; Codex remained primary for planning, edits, validation orchestration, docs sync, commit, and push. |
| 2026-07-04 | I095-B5 | `docs/iterations/I095-runtime-validation-evidence.md` | Does not qualify by itself. The allowlisted validation evidence mechanism exists, but this implementation and validation were Codex-primary. |
| 2026-07-04 | I096-B7 | `docs/iterations/I096-governance-mutation-gates.md` | Does not qualify by itself. The owner-doc mutation gate exists, but this implementation and validation were Codex-primary. |
| 2026-07-04 | I097-B9 | `docs/tasks/2026-07-04-self-bootstrap-rehearsal-i097-b9-nonqualification.md` | Does not qualify. Talos executed bounded validation and owner-doc mutation commands, but Codex remained primary for planning, evidence interpretation, docs editing, broader validation orchestration, commit, and push. |
| 2026-07-06 | I098-I101 | `docs/reference/I098-I101-AUTONOMY-PERMISSION-RUNTIME-CLOSEOUT-2026-07-06.md` | Does not qualify. Talos capabilities improved, but this long task was Codex-primary; Talos was the implementation target and validation subject, not the primary autonomous executor. |
| 2026-07-06 | SB100-SB130 | `docs/tasks/2026-07-06-talos-low-ambiguity-self-bootstrap-pilot.md` | **Partial qualification.** Talos (deepseek-v4-pro) was the primary runtime for SB100-SB130: startup inventory, evidence frame, RunningTool turn-phase completion, TUI-022 checkbox unification, TUI-023 diff backgrounds, PERF-001 bash policy build-time materialization, validation, commit, and owner-doc sync. Four non-trivial code changes across 4 crates (talos-conversation, talos-tui, talos-cli, talos-tools). All validation gates passed. No Codex/senior-agent code edits are recorded in the Talos-authored execution evidence; the later Codex review/remediation corrected closeout defects and keeps the result partial rather than full REL-002 qualification. PERF-001 Phase 1 (models.toml) remains future work per pilot scope boundary. This is the first qualifying partial-evidence session where Talos acted as primary across the full development loop. |
| 2026-07-06 | SSP100-SSP150 | `docs/tasks/2026-07-06-self-bootstrap-stability-pilot.md` | **Partial qualification.** Talos (deepseek-v4-pro) was the primary runtime for the full stability pilot: SSP100 startup inventory and evidence frame, SSP110 TOOL-019 bash exit-code classification, SSP120 TODO-002 idempotent todo_create, SSP130 TUI-028 stale preview clear, SSP140 RUNTIME-002 engine-level is_processing verification, and SSP150 closeout. Five conventional commits across 4 crates (talos-tools, talos-session, talos-tui, talos-conversation). All validation gates passed: cargo fmt, cargo check, governance validation, workspace tests (all 60 suites, 0 failed). Talos planned, implemented, tested, committed, and synced owner docs autonomously. No Codex code edits in qualifying evidence; push/final Board sync not performed (not authorized). This is the second qualifying partial-evidence Talos-primary development session. |
| 2026-07-08 | I102-I105 | `docs/iterations/I105-trial-readiness-closeout.md` | Does not qualify. The four-month developer operating plan produced controlled-local-trial evidence and a NO-GO v1.0 classification, but execution was external-runtime primary rather than Talos-primary. |
| 2026-07-08 | I106-I109 planned | `docs/tasks/2026-07-08-four-month-talos-self-bootstrap-plan.md` | Planned Talos-primary attempt. No qualification claim yet; future evidence must be recorded by I106-I109 execution. |

## Readiness Reports

| Date | Record | Verdict |
|---|---|---|
| 2026-07-02 | `docs/reference/REL-002-READINESS-REPORT-2026-07-02.md` | Not ready for `v1.0.0`; pre-1.0 releases may continue with explicit posture. |
| 2026-07-04 | `docs/reference/REL-002-READINESS-REPORT-2026-07-04.md` | Not ready for `v1.0.0`; runtime/governance/architecture prerequisites improved, but no qualifying Talos-primary sessions exist. |
| 2026-07-06 | `docs/reference/I098-I101-AUTONOMY-PERMISSION-RUNTIME-CLOSEOUT-2026-07-06.md` | Not ready for `v1.0.0`; autonomy/runtime prerequisites improved, but no qualifying Talos-primary development session exists. |
| 2026-07-08 | `docs/iterations/I105-trial-readiness-closeout.md` | Not ready for `v1.0.0`; controlled local trial can proceed, but REL-002 remains unmet because I102-I105 were external-runtime primary. |

## Relationship To Other Work

| Item | Relationship |
|---|---|
| `RUNTIME-001` | Provides the embeddable runtime boundary needed for Talos to be reused and tested as a real runtime, not only a CLI app. |
| `GOV-003` | Built-in governance logic is a major self-bootstrap enabler. |
| `WEB-001` | Optional control surface for status/governance/review workflows, not itself required for 1.0. |
| `TOOL-004` / `TOOL-007` / `WEBFETCH-001` | Tool quality and context ingestion affect whether Talos can perform real development work without external support. |
| `MEM-005` / `MEM-007` / `MEM-003` | Context and memory reliability affect long-running self-development. |
| `ARCH-030` | Remaining architecture residual roots must not block self-development or be hidden debt at 1.0. |

## Required Reads

- `docs/backlog/active/RUNTIME-001-embeddable-agent-runtime-api.md`
- `docs/backlog/active/GOV-003-builtin-project-governance.md`
- `docs/tasks/2026-06-28-architect-owned-high-risk-work-group.md`
- `docs/reference/ARCHITECTURE.md`
- `docs/BOARD.md`
