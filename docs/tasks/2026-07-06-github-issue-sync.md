# GitHub Issue Sync 2026-07-06

> Created: 2026-07-06
> Trigger: Maintainer requested pulling GitHub issues and synchronizing requirements/status after
> the self-bootstrap pilot closeout.
> Status: Complete

## Scope

Pull open GitHub issues and map each one to a local owner document, proposal, or existing backlog
story. This task does not implement issue content.

## Source Snapshot

Command:

```sh
gh issue list --limit 100 --state open --json number,title,state,labels,updatedAt,url
```

Result: 18 open issues, #18 through #35.

## Mapping

| Issue | Local owner | Status | Notes |
|---|---|---|---|
| [#18](https://github.com/wjhuang88/talos/issues/18) tool error can leave turn stuck in processing | `RUNTIME-002` | Planned | P0 runtime reliability. |
| [#19](https://github.com/wjhuang88/talos/issues/19) todo batch create/update and hide UUID | `TODO-002` | Planned | Split into mutation API and TUI rendering acceptance. |
| [#20](https://github.com/wjhuang88/talos/issues/20) edit diff color rendering | `TOOL-018` | Planned | Coordinates with `TUI-023` diff colors and `TOOL-015`. |
| [#21](https://github.com/wjhuang88/talos/issues/21) git diff unified output | `TOOL-018` | Planned | Must preserve read-only permission boundary. |
| [#22](https://github.com/wjhuang88/talos/issues/22) workspace permission sandbox design | `PERM-004` | Planned | ADR-required architecture work. |
| [#23](https://github.com/wjhuang88/talos/issues/23) bash exit-code classification | `TOOL-019` | Planned | P0/P1 tool reliability; avoid false `is_error`. |
| [#24](https://github.com/wjhuang88/talos/issues/24) preview animation cadence instability | `TUI-028` | Planned | TUI feedback reliability. |
| [#25](https://github.com/wjhuang88/talos/issues/25) thinking animation redesign | `TUI-028` | Planned | Visual slice, depends on stable animation cadence. |
| [#26](https://github.com/wjhuang88/talos/issues/26) thinking content in history | `MODEL-003` / `TUI-028` | Needs decision | Conflicts with ADR-034/TUI-020 transient-thinking boundary unless a new decision approves persistence/export. |
| [#27](https://github.com/wjhuang88/talos/issues/27) preview residue after Ctrl+C/new input | `TUI-027` / `TUI-028` | Planned | Existing `TUI-027` owns ordering/residue; `TUI-028` owns user-facing feedback polish. |
| [#28](https://github.com/wjhuang88/talos/issues/28) dashboard available message looks like error | `TUI-028` | Planned | Startup/info message presentation. |
| [#29](https://github.com/wjhuang88/talos/issues/29) talos-desktop proposal | `docs/proposals/talos-desktop.md` | Proposal | No implementation authority. |
| [#30](https://github.com/wjhuang88/talos/issues/30) multi-agent architecture proposal | `docs/proposals/multi-agent-architecture.md` | Proposal | No implementation authority; ADR-006 guardrails apply. |
| [#31](https://github.com/wjhuang88/talos/issues/31) status bar model-name format jump | `TUI-028` | Planned | Status bar visual stability. |
| [#32](https://github.com/wjhuang88/talos/issues/32) health check task | `RUNTIME-002` | Planned | Systemic follow-up to #18; auto-recovery requires explicit safety boundary. |
| [#33](https://github.com/wjhuang88/talos/issues/33) `/todo delete` slash command | `TODO-002` | Planned | Write-capable user command requires confirmation/permission design. |
| [#34](https://github.com/wjhuang88/talos/issues/34) todo_create idempotency | `TODO-002` | Planned | P0 duplicate prevention for resume/retry. |
| [#35](https://github.com/wjhuang88/talos/issues/35) single-data-flow audit | `ARCH-032` | Planned | Documentation/audit first, no code changes in the audit story. |

## Local Documents Updated

- `docs/backlog/active/RUNTIME-002-turn-health-and-stuck-processing.md`
- `docs/backlog/active/TODO-002-todo-mutation-reliability.md`
- `docs/backlog/active/TOOL-018-diff-output-and-rendering.md`
- `docs/backlog/active/TOOL-019-bash-exit-code-classification.md`
- `docs/backlog/active/TUI-028-preview-status-feedback-reliability.md`
- `docs/backlog/active/PERM-004-workspace-trust-sandbox.md`
- `docs/backlog/active/ARCH-032-single-data-flow-audit.md`
- `docs/proposals/talos-desktop.md`
- `docs/proposals/multi-agent-architecture.md`
- `docs/backlog/PRODUCT-BACKLOG.md`
- `docs/BOARD.md`
- `docs/proposals/README.md`

## Validation

- `scripts/validate_project_governance.sh .`: passed, 0 warnings
- `git diff --check`: passed

