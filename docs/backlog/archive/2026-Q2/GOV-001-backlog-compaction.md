# GOV-001: Backlog Compaction

## Problem

`docs/backlog/PRODUCT-BACKLOG.md` has grown past 1,600 lines. The updated governance skill now
treats large or growing product backlogs as a drift risk: the main backlog should remain a compact
prioritization surface, while executable detail moves into active item files with explicit Required
Reads.

## Goal

Migrate the backlog toward the compact shape defined by the governance skill without losing active
decision context.

## Non-Goals

- Do not rewrite story history for completed iterations.
- Do not archive active, blocked, or still-constraining work.
- Do not change story priority while compacting unless a separate prioritization decision is made.

## Status

Complete (2026-06-05).

## Priority

P2. This is not blocking current implementation, but it should happen before adding another large
planning wave.

## Governing References

- `docs/backlog/PRODUCT-BACKLOG.md`
- `docs/iterations/README.md`
- `docs/roadmap/REQUIREMENT-CONVERGENCE.md`
- `agent-project-governance/references/backlog-compaction.md`

## Acceptance Criteria

- [x] `PRODUCT-BACKLOG.md` becomes a compact routing/prioritization surface.
- [x] Active and blocked work has item files under `docs/backlog/active/`.
- [x] Each active row lists Required Reads including governing ADRs and item files.
- [x] Archived or completed work has an archive index and keeps enough decision context.
- [x] The governance validator passes after compaction.

## Validation Evidence Required

- `sh <agent-project-governance>/scripts/validate_project_governance.sh .`
- Link/required-read spot check for at least I014-I020 and active/paused work.

## Validation Evidence

- `sh /Users/GHuang/WorkSpace/AiProjects/skill-sources/agent-project-governance/skills/agent-project-governance/scripts/validate_project_governance.sh /Users/GHuang/WorkSpace/RustProjects/talos` passed with 0 warnings on 2026-06-05.
- Required Reads spot check covers TUI-001, PROV-001, TOOL-001, GIT-001, OBS-001, MEM-001, RES-001, EXT-001, and PERM-001.
