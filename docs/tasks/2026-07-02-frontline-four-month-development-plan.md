# 2026-07-02 Frontline Four-Month Development Plan

**Status**: Planned
**Owner area**: Post-v0.2.1 delegated development for frontline engineers.
**Created**: 2026-07-02
**Timebox**: 16 weeks / roughly 4 months
**Primary release marker**: Continue pre-1.0 hardening without claiming `REL-002` readiness.
**Supersedes**: No completed baseline. This plan starts after the completed
`docs/tasks/2026-07-01-four-month-self-bootstrap-replan.md`.

## Objective

Give frontline engineers a concrete four-month execution plan that turns the post-v0.2.1 backlog
into delegable, testable slices. The plan focuses on product usefulness, runtime extensibility,
governance visibility, configuration polish, document ingestion, and release/distribution hygiene
while keeping high-risk boundaries behind explicit design or review gates.

This is a development handoff, not a release authorization. It does not authorize crate publishing,
release tags, remote dashboard access, browser automation, write-capable plugin tools, marketplace
distribution, or permission-default changes.

## Audience And Execution Model

This plan is intended for front-line developers or small implementation pods. Each task is sized to
be independently reviewable and must end with runnable evidence. Engineers should start from the
lowest-numbered planned task in the active monthly iteration unless the maintainer explicitly
changes priority.

Owner docs remain the source of truth. This plan orders the work; it does not replace backlog item
acceptance criteria, ADRs, or iteration records.

## Operating Constraints

- Preserve Rust-first constraints from `AGENTS.md`; no Python/Node runtime dependency, no arbitrary
  native bindings, and no `unsafe` without ADR coverage.
- Treat plugin runtime, hooks, asset distribution, web/dashboard expansion, and permission behavior
  as security-sensitive surfaces.
- Keep dashboard work loopback-only unless a future ADR explicitly expands the boundary.
- Keep plugin work local, explicit, read-only, and provenance-carrying until a later gate expands
  capability.
- Do not silently load shared `~/.agents` skills, MCP servers, models, or assets. Shared imports are
  opt-in and Talos-owned config takes precedence.
- Do not claim `REL-002` or v1.0 readiness. Qualifying self-bootstrap sessions still require Talos
  as the primary executor.
- Update owner docs before `docs/BOARD.md`.
- If any GitHub issue-linked owner doc changes status, comment on the originating issue before
  closeout.

## Required Reads

- `AGENTS.md`
- `docs/BOARD.md`
- `docs/backlog/PRODUCT-BACKLOG.md`
- `docs/sop/LONG-RUNNING-TASK.md`
- `docs/sop/START-ITERATION.md`
- `docs/sop/ITERATION-WORKFLOW.md`
- `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md`
- `docs/backlog/active/GOV-003-builtin-project-governance.md`
- `docs/backlog/active/TUI-021-command-line-composer-navigation.md`
- `docs/backlog/active/PLUGIN-001-wasm-runtime-plugins.md`
- `docs/backlog/active/CMD-002-command-taxonomy-realignment.md`
- `docs/backlog/active/HOOK-001-config-introduced-hooks.md`
- `docs/backlog/active/CONF-001-config-editing.md`
- `docs/backlog/active/WEBFETCH-001-web-and-document-fetch-tools.md`
- `docs/backlog/active/TOOL-008-tree-sitter-on-demand.md`
- `docs/backlog/active/DIST-001-optional-runtime-asset-distribution.md`
- `docs/backlog/active/AGENT-002-dotagents-protocol-support.md`
- `docs/backlog/active/WEB-001-embedded-web-control-surface.md`
- `docs/backlog/active/ARCH-022-cli-mode-runner-residual-decomposition.md`
- `docs/backlog/active/ARCH-023-tui-app-residual-decomposition.md`
- `docs/reference/REL-002-READINESS-REPORT-2026-07-02.md`

## Starting Inventory

| Bucket | Item | Owner Doc | Disposition |
|---|---|---|---|
| Complete | Four-Month Self-Bootstrap Replan | `docs/tasks/2026-07-01-four-month-self-bootstrap-replan.md` | Closed. T100-T137 and I076-I079 are historical baseline and must not be rewritten. |
| Planned | REL-002 | `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md` | Remains planned. This plan improves prerequisites but does not claim v1 readiness. |
| In Progress | PLUGIN-001 | `docs/backlog/active/PLUGIN-001-wasm-runtime-plugins.md` | Continue from read-only WASM tool review into CLI/config listing and package polish; no remote install or write tools. |
| Planned | GOV-003 | `docs/backlog/active/GOV-003-builtin-project-governance.md` | Promote read-only governance state into developer-facing command/dashboard views. Mutating governance remains future work. |
| Planned | WEBFETCH-001 | `docs/backlog/active/WEBFETCH-001-web-and-document-fetch-tools.md` | Select bounded document capture and HTML/link extraction slices. PDF/Office/OCR/browser remain gated. |
| Planned | TOOL-008 | `docs/backlog/active/TOOL-008-tree-sitter-on-demand.md` | Select Phase 2 feature-gated parser set. Runtime WASM parser loading remains future/DIST-gated. |
| Planned | DIST-001 | `docs/backlog/active/DIST-001-optional-runtime-asset-distribution.md` | Select proposal and manifest design only; no online install implementation in this plan unless separately approved. |
| Research | AGENT-002 | `docs/backlog/active/AGENT-002-dotagents-protocol-support.md` | Select opt-in shared skills discovery only after ADR/policy. Models/MCP shared import stay out of this plan. |
| Planned | CONF-001 | `docs/backlog/active/CONF-001-config-editing.md` | Select CLI subcommand hardening and TUI `/config` readiness decision. |
| Planned | ARCH-022/ARCH-023 | Owner docs | Use as bounded cleanup buffers when touched code grows; do not make architecture cleanup the only deliverable. |
| Paused | Architect-Owned High-Risk Work Group | `docs/tasks/2026-06-28-architect-owned-high-risk-work-group.md` | Remains paused. This plan delegates lower-risk slices with explicit gates. |

## Track Overview

| Track | Theme | Outcome |
|---|---|---|
| A | Governance and operator visibility | `/agile` and dashboard read-only governance views become useful for developers. |
| B | Runtime extension polish | Local explicit plugin packages are listable, diagnosable, and testable without expanding permissions. |
| C | Configuration and command UX | Config subcommands and command taxonomy are consistent enough for daily use. |
| D | Document and code ingestion | HTML/document capture and parser footprint improve without heavy runtime dependencies. |
| E | Distribution and shared ecosystem | Optional asset and dotagents compatibility policies are ready for later implementation. |
| F | Release/readiness evidence | Monthly closeouts keep pre-1.0 posture honest and maintainable. |

## Four-Month Execution Matrix

| ID | Week | Iteration | Track | Deliverable | Dependencies | Validation | Status |
|---|---:|---|---|---|---|---|---|
| F100 | 1 | I080 | A | Publish this plan plus I080-I083 iteration shells and board sync. | Current board/backlog inventory | Governance validator | Planned |
| F101 | 1 | I080 | C | `talos config list/get/set` subcommands using existing config API, preserving old flags. | CONF-001 | CLI tests; config masking tests | Planned |
| F102 | 2 | I080 | C | Config validation evidence: env substitution, schema rejection, api_key masking, save/load round-trip. | F101 | `cargo test -p talos-config -p talos-cli` | Planned |
| F103 | 2 | I080 | C | TUI `/config` readiness decision: implement model/provider read-only or defer with UX spec. | F101/CMD-001 | TUI command tests or decision doc | Planned |
| F104 | 3 | I080 | C | TUI composer command-line navigation shortcuts: `Ctrl+A` line start and `Ctrl+E` line end. | TUI-021/TUI-009 | TUI keyboard tests | Planned |
| F105 | 3 | I080 | A | GOV-003 `/agile status` read-only command backed by board/backlog/iteration parsing. | GOV-003/CMD-001 | CLI/TUI command tests | Planned |
| F106 | 4 | I080 | A | Dashboard governance read-only page or JSON route for board/backlog/validation status. | WEB-001/GOV-003/F105 | dashboard route tests; redaction tests | Planned |
| F107 | 4 | I080 | F | Month-1 closeout: config/governance/input evidence and residuals. | F100-F106 | `cargo test --workspace`; governance | Planned |
| F110 | 5 | I081 | B | Plugin package diagnostics: list configured local packages, manifests, capabilities, and validation errors. | PLUGIN-001/CMD-002 | plugin manifest tests; `/plugins` tests | Planned |
| F111 | 5 | I081 | C | `/hooks` builtin-hook listing and config-introduced hook diagnostics without executable carriers. | HOOK-001/CMD-002 | command registry tests; hook diagnostics tests | Planned |
| F112 | 6 | I081 | B | Read-only plugin fixture polish: examples, failure fixtures, bounded output/provenance tests, docs. | PLUGIN-001 | `cargo test -p talos-plugin -p talos-tools` | Planned |
| F113 | 6 | I081 | B | Plugin package enablement UX: explicit config/CLI opt-in for local packages, disabled by default. | F110/F112 | config/load tests; permission denial tests | Planned |
| F114 | 7 | I081 | E | DIST-001 asset distribution proposal: manifest, cache layout, verification, offline/mirror policy. | DIST-001/PLUGIN-001 | proposal + ADR draft | Planned |
| F115 | 8 | I081 | B/E | Security closeout for plugin/assets: no remote install, no write tools, no marketplace claims. | F110-F114 | security review note; governance | Planned |
| F116 | 8 | I081 | F | Month-2 closeout: extension/distribution evidence and residuals. | F110-F115 | `cargo test --workspace`; governance | Planned |
| F120 | 9 | I082 | D | WEBFETCH design update for bounded document capture and HTML/link extraction. | WEBFETCH-001/TOOL-014 | proposal/owner doc update | Planned |
| F121 | 9 | I082 | D | `document_extract` for local text/Markdown/HTML/JSON/CSV with bounded output and metadata. | F120 | tools tests; permission tests | Planned |
| F122 | 10 | I082 | D | `fetch_url` HTML extraction with title/main content/top links/full link count. | F120 | fetch_url tests; mock server tests | Planned |
| F123 | 10 | I082 | D | Link store/reference follow-up metadata without implicit persistence of fetched content. | F122 | storage/reference tests | Planned |
| F124 | 11 | I082 | D | TOOL-008 Phase 2 parser feature gates and graceful fallback for unavailable parsers. | TOOL-008/CODE-002 | default/all-features tests; release size check | Planned |
| F125 | 12 | I082 | D | Document/code ingestion closeout: unsupported binary behavior, docs, and dependency rationale. | F121-F124 | workspace tests; site/docs validation if touched | Planned |
| F126 | 12 | I082 | F | Month-3 closeout: ingestion/parser evidence and residuals. | F120-F125 | `cargo test --workspace`; governance | Planned |
| F130 | 13 | I083 | E | AGENT-002-B ADR/policy for opt-in `~/.agents/skills` discovery. | AGENT-002/SKILL-002 | ADR + owner doc update | Planned |
| F131 | 13 | I083 | E | Implement opt-in shared skills discovery if F130 clears; Talos-owned skills take precedence. | F130 | skill loader tests; prompt budget tests | Planned |
| F132 | 14 | I083 | A/F | REL-002 rehearsal packet using Talos where feasible; record exact primary-executor boundary. | REL-002/GOV-003 | rehearsal evidence doc | Planned |
| F133 | 14 | I083 | C/F | Command/help/docs consistency sweep: `/agile`, `/plugins`, `/hooks`, `/config`, dashboard URL docs. | F101-F131 | README/site validators | Planned |
| F134 | 15 | I083 | F | Pre-release posture report: publish guard, release notes draft, install/docs matrix, no v1 claim. | all tracks | publish guard; governance | Planned |
| F135 | 16 | I083 | F | Final closeout matrix for F100-F134 with residual owner list. | F134 | `cargo test --workspace`; clippy; governance | Planned |
| F136 | 16 | I083 | F | Final handoff for the next maintainer/developer cycle. | F135 | handoff doc | Planned |

## Milestones

| Milestone | Target Week | Exit Criteria |
|---|---:|---|
| M1 Config, input, and governance visibility | 4 | Config subcommands, composer navigation shortcuts, and read-only governance status are usable and documented, or blockers are explicit. |
| M2 Extension surface disciplined | 8 | Plugin/package/hook surfaces are diagnosable without new permission risk; asset distribution policy is drafted. |
| M3 Ingestion footprint improved | 12 | Document/HTML ingestion and parser feature gates land with bounded output and fallback behavior. |
| M4 Delegated cycle closed | 16 | Shared skills decision, REL-002 rehearsal evidence, docs, release posture, and residual owners are recorded. |

## Non-Authorizations

This plan does not authorize:

- real crate publication;
- release tags or GitHub Releases;
- remote dashboard or LAN binding;
- browser automation, cookies, storage, or profile access;
- write-capable plugin tools;
- remote plugin installation or marketplace behavior;
- automatic downloads of executable or model assets;
- default-on shared `~/.agents` discovery;
- permission-default changes;
- v1.0 readiness claims.

## Validation Policy

Minimum validation for implementation slices:

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- Targeted `cargo test -p <crate>` for touched crates
- Targeted `cargo clippy -p <crate> -- -D warnings` for touched crates
- `scripts/validate_project_governance.sh .`

Monthly closeout validation:

- `cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- `scripts/validate_project_governance.sh .`
- `scripts/check_publish_guard.sh .` when manifests, release posture, or publish docs changed
- `scripts/validate_public_site.sh` when `site/` changed

Security-sensitive slices also require a written review note when touching plugin execution,
dashboard/web routes, optional asset distribution, shared config discovery, permission evaluation,
or document/network ingestion.

## Issue Sync Policy

If an owner doc references a GitHub issue and this plan changes its status, comment on that issue
with the story ID, new status, commit reference, and one-line summary. Close only when the owner doc
is Complete or Cancelled.

## Frontline Handoff Prompt

```text
You are taking over Talos's 2026-07-02 frontline four-month development plan.

Start by reading:
- docs/tasks/2026-07-02-frontline-four-month-development-plan.md
- docs/BOARD.md
- docs/backlog/PRODUCT-BACKLOG.md
- docs/iterations/I080-frontline-config-governance-visibility.md

Execute the plan in order. Owner docs are source of truth; update them before docs/BOARD.md.
Do not publish crates, tag releases, enable remote web access, add browser automation, add
write-capable plugin tools, implement remote plugin install, or change permission defaults without
the exact approval/gate named in the plan.

Begin with F100-F104. At each month closeout, run cargo test --workspace, cargo clippy --workspace
-- -D warnings, and scripts/validate_project_governance.sh ., then append a checkpoint to the plan.
```

## Recovery Instructions

1. Run `git status --short`.
2. Read the latest checkpoint in this file.
3. Read the current monthly iteration shell (`I080` through `I083`).
4. Read owner docs for the next planned task.
5. Run `scripts/validate_project_governance.sh .` before changing governance files.
6. Continue from the lowest-numbered planned task unless the maintainer explicitly changes priority.

## Execution Log

### Planning Checkpoint (2026-07-02)

- Created a new delegated four-month plan after the v0.2.1 release tag was pushed.
- Prior 2026-07-01 self-bootstrap replan remains Complete and historical.
- New iteration shells I080-I083 define the monthly execution containers.
- No backlog owner status is changed by this planning checkpoint.
