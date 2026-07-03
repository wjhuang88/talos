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

## Readiness Reports

| Date | Record | Verdict |
|---|---|---|
| 2026-07-02 | `docs/reference/REL-002-READINESS-REPORT-2026-07-02.md` | Not ready for `v1.0.0`; pre-1.0 releases may continue with explicit posture. |

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
