# 2026-07-01 Four-Month Self-Bootstrap Replan

**Status**: Planned
**Owner area**: High-risk product hardening, self-bootstrap validation, plugin/runtime execution,
tool reliability, web/dashboard security, session orchestration, memory/context policy, and release
readiness.
**Created**: 2026-07-01
**Timebox**: 16 weeks / roughly 4 months
**Primary release marker**: `REL-002` v1.0 self-bootstrap gate
**Supersedes**: Month 4 (T55-T65) of
`docs/tasks/2026-06-30-four-month-self-bootstrap-product-hardening-plan.md`; Months 1-3 remain
historical completed baseline.

## Objective

Replan the remaining self-bootstrap work after Month 3 by treating the old Month 4 tasks as an
unfinished task set, adding GitHub issue demand (#7-#16), and deliberately scheduling the high-risk
work that should be handled by a senior coding agent in this session family: provider usage
accounting, direct process execution policy, plugin tool integration, autonomous validation, and
session-level orchestration.

This is not a release authorization. It is an execution plan that produces implementation,
evidence, and go/no-go decisions required before `REL-002` can be evaluated seriously.

## Operating Constraints

- No real `cargo publish`, tag, release, or `publish = false` removal without explicit maintainer
  approval for that exact action.
- No permission-default change without an ADR or owner-doc gate that names the exact behavioral
  change and regression tests.
- `exec` and plugin tool execution are process/runtime boundary work. They must pass security
  review before becoming default-presented tools.
- Browser/dashboard work remains loopback-only. No remote access, browser automation, cookies,
  storage, or profile access without a new ADR.
- Associative memory prompt injection remains default-off unless T60/T131 produces metrics and an
  accepted decision to enable it under config.
- Owner docs define truth; update owner docs before `docs/BOARD.md`.
- Published baselines are append-only. This replan supersedes future scheduling, not historical
  evidence.

## Success Criteria

- OpenAI-compatible provider usage is accurate enough for status bar percentages, exit summaries,
  and cost accounting.
- The tool surface is safer and more transparent: write/edit outputs are verifiable, tool output has
  visual hierarchy, and direct `exec` has an explicit permission policy.
- Plugin MVP moves from raw WASM adapter to a permission-gated, provenance-carrying read-only
  `AgentTool` path or is explicitly deferred with security findings.
- WEB-001/WEB-005 security review produces concrete fixes, not only a report.
- Session orchestration improves: slash panel auto-execute, model-switch markers, and session todo
  foundations land with persistence tests.
- Thinking content is separated from durable history.
- Autonomous validation becomes a first-class self-bootstrap loop; Talos can run or request its own
  validation, not only edit files.
- Month 4 release/readiness artifacts are replaced by a better closeout: validation matrix, release
  posture, SDK/package posture, and residual owner list.

## Required Reads

- `docs/BOARD.md`
- `docs/backlog/PRODUCT-BACKLOG.md`
- `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md`
- `docs/backlog/active/PROVIDER-001-openai-streaming-usage.md`
- `docs/backlog/active/TUI-017-context-usage-percentage.md`
- `docs/backlog/active/SESSION-003-model-switch-context-marker.md`
- `docs/backlog/active/TUI-018-context-limit-million-format.md`
- `docs/backlog/active/TOOL-015-write-edit-result-visibility.md`
- `docs/backlog/active/TUI-019-tool-output-visual-hierarchy.md`
- `docs/backlog/active/TUI-020-thinking-preview-not-history.md`
- `docs/backlog/active/TOOL-016-direct-exec-tool.md`
- `docs/backlog/active/TODO-001-session-todo-list.md`
- `docs/backlog/active/TUI-016-slash-panel-smart-auto-execute.md`
- `docs/backlog/active/PLUGIN-001-wasm-runtime-plugins.md`
- `docs/backlog/active/WEB-001-embedded-web-control-surface.md`
- `docs/backlog/active/WEB-005-browser-session-continuity-research.md`
- `docs/backlog/active/MEM-008-weighted-associative-memory-graph.md`
- `docs/reference/CRATE-PUBLICATION-MATRIX.md`
- `docs/reference/SELF-BOOTSTRAP-EVIDENCE-TEMPLATE.md`

## Starting Inventory

This inventory was taken before selecting I076-I079. It records disposition for Active, Review,
Planned, and Blocked/Paused work that affects this replan.

| Bucket | Item | Owner Doc | Disposition |
|---|---|---|---|
| Active | R27 High-Risk Governance Gate | `docs/tasks/2026-06-27-personal-oversight-high-risk-roadmap.md` | Remains active; this replan does not grant personal approval authority for tags, publish, destructive cleanup, network spend, new runtime deps, or permission-boundary changes. |
| Active | PLUGIN-001 | `docs/backlog/active/PLUGIN-001-wasm-runtime-plugins.md` | Include high-risk T110/T111. Raw WASM adapter exists; AgentTool/permission integration is not complete. |
| Active | CMD-002 | `docs/backlog/active/CMD-002-command-taxonomy-realignment.md` | Keep as dependency for plugin/hook command UX; no direct Month 1 implementation unless needed. |
| Active | ARCH-031 | `docs/backlog/active/ARCH-031-crate-publication-boundary.md` | Keep publication gate as documentation/decision work only unless maintainer approves publish. |
| Active | DATA-001 | `docs/backlog/active/DATA-001-local-data-lifecycle-storage-hygiene.md` | Already substantially delivered; not selected except where session todo storage touches lifecycle constraints. |
| Review | SKILL-002 | `docs/backlog/active/SKILL-002-explicit-runtime-activation.md` | Leave in Review; do not mix with this replan unless validation evidence is needed. |
| Planned | I018/I028 and legacy planned iterations | `docs/iterations/README.md` | Deferred. This replan creates I076-I079 rather than activating old broad baselines. |
| Planned | Old Month 4 T55-T65 | `docs/tasks/2026-06-30-four-month-self-bootstrap-product-hardening-plan.md` | Superseded into the Unfinished Set below. |
| Planned | GitHub issues #7-#16 | Backlog owner docs listed in Required Reads | Included. #12 is scheduled first because #9 depends on accurate token usage. |
| Planned | TODO-001 | `docs/backlog/active/TODO-001-session-todo-list.md` | Included in Month 3 of this plan due to direct self-bootstrap orchestration value. |
| Planned | WEB-001/WEB-005 | Owner docs and ADR-031 | Security review and HTML/dashboard fixes included before further web expansion. |
| Blocked/Paused | PERM-001 | `docs/backlog/active/PERM-001-guardian-exec-policy.md` | Remains deferred, but TOOL-016 must define a narrower exec permission policy before code. |
| Paused | Architect-Owned High-Risk Work Group | `docs/tasks/2026-06-28-architect-owned-high-risk-work-group.md` | Stays paused; this replan handles high-risk work under explicit story gates. |

## Unfinished Task Set From Prior Month 4

| Old ID | Prior Task | New Disposition |
|---|---|---|
| T55 | `talos-cli` real publish if approved | T133 gate packet only; real publish remains maintainer-only. |
| T56 | `talos-runtime` publish/reserve if approved | T133 gate packet only; real publish remains maintainer-only. |
| T57 | Tool reliability sweep | Split across T101, T104, T105, T122, and final sweep T130. |
| T58 | WEB-001/WEB-005 security review | T112 plus T113 fixes. |
| T59 | Plugin MVP security review | T110/T111. |
| T60 | Automatic associative memory injection decision | T131. |
| T61 | Third Talos-on-Talos rehearsal | T123 and T132; target includes autonomous validation. |
| T62 | Release/user docs consolidation | T134. |
| T63 | v1.0 readiness report | T135. |
| T64 | Final closeout | T136. |
| T65 | Final handoff artifacts | T137. |

## Track Overview

| Track | Theme | Outcome |
|---|---|---|
| A | Governance and self-bootstrap loop | Autonomous validation and evidence become executable, not prompt-only. |
| B | Provider/session correctness | Usage accounting, status display, model switch markers, thinking separation. |
| C | Tooling and execution safety | Write/edit visibility, output hierarchy, direct exec policy and implementation. |
| D | Web/browser/dashboard security | Loopback dashboard and browser-page mock are reviewed and hardened. |
| E | Plugin/runtime extension | WASM adapter becomes a gated plugin tool or is explicitly deferred. |
| F | Memory/context orchestration | Associative memory injection decision and session todo foundations. |
| G | Release/package readiness | Publish gates, docs consolidation, v1 readiness, final handoff. |

## Four-Month Execution Matrix

| ID | Week | Track | Deliverable | Dependencies | Validation | Status |
|---|---:|---|---|---|---|---|
| T100 | 1 | A | Publish this replan, I076-I079 iteration shells, and starting disposition. | Board/backlog inventory | Governance validator | Complete |
| T101 | 1 | B | Fix PROVIDER-001 streaming usage: request `include_usage`, parse usage-only chunks. | Issue #12 | `cargo test -p talos-provider`; request-body tests | Review |
| T102 | 1 | B | Implement TUI-018 million-unit context limit formatting. | TUI status docs | `cargo test -p talos-tui` targeted | Review |
| T103 | 1 | B | Implement TUI-017 context usage percentage using accurate usage when available. | T101 | TUI compact/status tests | Review |
| T104 | 2 | C | Implement TOOL-015 write/edit result visibility with bounded preview/diff. | TOOL-003 | `cargo test -p talos-tools -p talos-tui` | Planned |
| T105 | 2 | C | Implement TUI-019 visual hierarchy for primary vs secondary tool output. | T104/TUI-007 | TUI style tests | Planned |
| T106 | 2 | B | Implement SESSION-003 model-switch context marker with persistence. | SESSION-001/CMD-001 | session JSONL/request-preview tests | Planned |
| T107 | 3 | A | Design autonomous validation loop: command/tool shape, security boundary, and no-hidden-pass rules. | REL-002/T52 evidence | ADR/proposal or owner-doc decision | Planned |
| T108 | 3 | A | Implement first safe validation surface if design clears: bounded read-only validation command or explicit tool. | T107 | targeted tests; no permission bypass | Planned |
| T109 | 4 | A | Month-1 closeout: provider/status/tool/session fixes and validation-loop decision. | T100-T108 | `cargo test --workspace`; governance | Planned |
| T110 | 5 | E | Plugin MVP security review: WASM adapter, timeout, host calls, permission/provenance gap. | T46/ADR-032 | Review document; threat model | Planned |
| T111 | 5-6 | E | Implement read-only WASM plugin `AgentTool` registration path if T110 clears. | T110 | permission/provenance/trap tests | Planned |
| T112 | 6 | D | WEB-001/WEB-005 security review: loopback auth, token display, logs, browser-page fields. | T42/T47 | Review document; no secret leakage tests | Planned |
| T113 | 6 | D | Apply dashboard/browser-page hardening from T112. | T112 | dashboard/tools tests; localhost smoke | Planned |
| T114 | 7 | C | TOOL-016 exec permission policy: allowlist/default/env/cwd/audit decision. | PERM-001/Issue #16 | ADR or owner-doc gate; permission tests planned | Planned |
| T115 | 7-8 | C | Implement direct `exec` tool only if T114 clears. | T114 | success/non-zero/timeout/denial tests | Planned |
| T116 | 8 | A | Second closeout: plugin/web/exec security posture and residual gates. | T110-T115 | `cargo test --workspace`; governance | Planned |
| T120 | 9 | F | Implement TUI-016 slash-panel smart auto-execute. | TUI-010/CMD-001 | command classification and Enter-branch tests | Planned |
| T121 | 9-10 | F | TODO-001 Phase 1: session todo data model and agent tool API behind permission pipeline. | TODO-001 | storage/tool tests; cycle detection | Planned |
| T122 | 10 | F | TODO-001 Phase 2: read-only slash/TUI views. | T121/CMD-001 | TUI/command tests | Planned |
| T123 | 10 | A | Self-bootstrap rehearsal using validation loop on a real doc/code slice. | T108/T122 | evidence record; validation run by Talos where feasible | Planned |
| T124 | 11 | B | TUI-020 thinking preview without durable history pollution. | TUI-004/session docs | stream/finalize/persistence/resume tests | Planned |
| T125 | 11 | F | TODO-001 Phase 3: bounded prompt integration for active todos. | T121/T122 | cache-stability and budget tests | Planned |
| T126 | 12 | A | Month-3 closeout: self-bootstrap coverage delta and TODO/thinking residuals. | T120-T125 | workspace tests; governance | Planned |
| T130 | 13 | C | Tool reliability sweep: flaky tests, shell naming residuals, Windows/Unix assumptions. | T104/T115 | issue list + targeted fixes | Planned |
| T131 | 13 | F | Decide automatic associative memory injection: reject, default-off, or config-gated. | MEM-008/T51 metrics | ADR/proposal update | Planned |
| T132 | 14 | A | Third rehearsal: architecture-sensitive slice with autonomous validation target >60%. | T123/T131 | evidence record; gap list | Planned |
| T133 | 14 | G | Publish gate packet for `talos-cli` and `talos-runtime`; no real publish unless approved. | ARCH-031/T55/T56 | publish guard; dry-run/blocker matrix | Planned |
| T134 | 15 | G | Consolidate release/user docs: README, site, crate docs, SDK examples, changelog draft. | all tracks | link/site validators | Planned |
| T135 | 15 | A | Produce REL-002 readiness report and next-quarter residual owner list. | T132/T134 | governance validation | Planned |
| T136 | 16 | A | Final closeout: validation matrix, commits, unreleased changes, issue sync status. | T100-T135 | `cargo test --workspace`; governance; publish guard | Planned |
| T137 | 16 | G | Final handoff artifacts: release posture, install posture, SDK posture, self-bootstrap posture. | T136 | handoff doc | Planned |

## Milestones

| Milestone | Target Week | Exit Criteria |
|---|---:|---|
| M1 Correctness and validation loop | 4 | Provider usage, status display, write/edit visibility, model-switch marker, and validation-loop decision are complete or blocked precisely. |
| M2 Runtime and web security | 8 | Plugin/web/exec decisions are documented; cleared slices have permission/security tests. |
| M3 Orchestration and history integrity | 12 | Slash auto-execute, TODO foundations, thinking separation, and a validation-backed rehearsal are complete or blocked precisely. |
| M4 Release posture known | 16 | Publish gates, docs, REL-002 readiness, residuals, and handoff are recorded. |

## Validation Policy

Minimum validation for implementation slices:

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- Targeted `cargo test -p <crate>` for touched crates
- `cargo clippy -p <crate> -- -D warnings` for touched crates
- `scripts/validate_project_governance.sh .`
- `scripts/check_publish_guard.sh .` when Cargo manifests, publish posture, or release docs change
- Site validator when `site/` changes
- Permission/security review note when touching tool execution, plugin runtime, dashboard/browser,
  sandbox, or process execution

Run `cargo test --workspace` at each monthly closeout and before final closeout.

## Issue Sync Policy

Issues #7-#16 already have backlog owner docs and Planned status comments. Any transition to
In Progress, Review, Complete, Blocked, or Cancelled must be commented back to the originating
issue with story ID, status, commit reference, and one-line summary. Close only when the owner doc
is Complete or Cancelled.

## Explicit Non-Authorizations

This replan does not authorize:

- real crate publication;
- release tags or GitHub Releases;
- remote dashboard/LAN access;
- browser automation or browser profile access;
- write-capable plugin tools;
- default-on associative memory injection;
- default-allow process execution;
- permission-default changes without accepted gate.

## Team Handoff Prompt

```text
You are taking over Talos's 2026-07-01 four-month self-bootstrap replan.

Start by reading:
- docs/tasks/2026-07-01-four-month-self-bootstrap-replan.md
- docs/BOARD.md
- docs/backlog/PRODUCT-BACKLOG.md
- docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md

Execute the plan in order. Owner docs are source of truth; update them before docs/BOARD.md.
Do not publish crates, tag releases, enable browser automation, add write-capable plugin tools,
or change permission defaults without the exact approval/gate named in the plan.

Begin with T100-T103. At each month closeout, run cargo test --workspace and
scripts/validate_project_governance.sh ., then append a checkpoint to the plan.
```

## Recovery Instructions

1. Run `git status --short`.
2. Read the latest checkpoint in this file.
3. Read the owner docs for the next planned task.
4. Run `scripts/validate_project_governance.sh .` before editing governance files.
5. Continue from the lowest-numbered planned item unless the maintainer explicitly changes priority.

## Execution Log

### Planning Checkpoint (2026-07-01)

- Created this replan after Month 3 closeout and GitHub issue sync.
- Old Month 4 T55-T65 retained as unfinished task set and remapped into T130-T137 plus earlier
  security/rehearsal work.
- New issue demand #7-#16 included.
- I076-I079 created as the next four monthly execution shells.

### I076 Activation Checkpoint (2026-07-01)

- Activated I076 for unattended execution.
- First implementation packet selected: T100-T103.
- Owner status changed to In Progress for PROVIDER-001, TUI-017, and TUI-018; verification and issue comments remain pending until tests pass.

### I076 T100-T103 Review Checkpoint (2026-07-01)

- T101 implemented OpenAI-compatible `stream_options.include_usage` and usage-only chunk parsing through `TurnEnd` usage.
- T102 implemented `M ctx` formatting for million-token context limits while preserving sub-million behavior.
- T103 implemented context usage percentages in the status bar using input plus output tokens.
- Verification passed: `cargo fmt --all -- --check`; `cargo test -p talos-provider`; `cargo test -p talos-tui status_bar`; `cargo test -p talos-tui`; `cargo check --workspace`; `cargo clippy -p talos-provider -p talos-tui -- -D warnings`; `scripts/validate_project_governance.sh .`.
