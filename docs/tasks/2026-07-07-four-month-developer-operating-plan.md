# 2026-07-07 Four-Month Developer Operating Plan

**Status**: Planned
**Created**: 2026-07-07
**Timebox**: 16 weeks / roughly 4 months
**Owner boundary**: developer delegation package; maintainer or senior agent reviews every monthly closeout
**Primary objective**: make Talos ready for controlled daily developer use without overclaiming release maturity.

## Assumptions And Constraint Classification

Hard constraints:

- All changes must follow `AGENTS.md`, the Rust-first boundary, permission gating, no unreviewed
  `unsafe`, and no secrets in source or distribution.
- This plan does not authorize release tags, branch pushes, crates.io publishing, GitHub Releases,
  remote deployment, paid network activity, destructive cleanup, permission-default relaxation, or
  sandbox/process-hardening changes.
- Owner docs are authoritative. `docs/BOARD.md` is derived and must be updated after owner docs.
- Existing published iteration baselines are preserved. Changed objectives use new iteration IDs.

Soft constraints:

- Prefer small developer packets that can be completed in one branch or commit.
- Prefer existing stories and code paths over new abstractions.
- Prefer deterministic tests and recorded runtime evidence over manual claims.

Assumptions:

- The receiving developers are capable of Rust implementation and local validation, but should not
  make architecture, security, permission, release, or product-scope decisions without review.
- The current project state is post-`v0.3.0` closeout with I085 paused on a manual walkthrough
  residual and I086-I089 still planned.

## Outcome

Deliver a four-month execution queue that improves provider/runtime reliability, first-run setup,
long-session developer usability, diagnostics, documentation, and release-gate evidence. At the end
of the plan, the maintainer should have a concrete go/no-go report for a controlled market trial,
not a `v1.0` claim.

## In Scope

- Provider streaming fixture coverage and malformed tool-use invariants.
- Runtime/TUI terminal status visibility and redacted incident evidence.
- `/connect`, `/model`, and model-list usability for standard and custom providers.
- Read-only diagnostics for configuration, provider protocol, credentials source, data directories,
  and validation routing.
- Permission-noise evidence collection and low-risk UX fixes that preserve deny precedence.
- Tool-output readability and bounded display behavior.
- Documentation, smoke checklist, and handoff reports for controlled trial use.
- REL-002 evidence updates that honestly classify self-bootstrap attempts as qualifying or
  non-qualifying.

## Out Of Scope

- No permission-boundary redesign, sandbox relaxation, Guardian auto-approval, or exec policy DSL
  implementation without a separate ADR/review gate.
- No new provider credential schema, OAuth flow, or credential persistence behavior without a
  separate ADR.
- No remote plugin marketplace, remote install, executable hooks, browser automation, PDF/Office/OCR
  ingestion, or write-capable plugin tools.
- No runtime model catalog database resurrection.
- No session-storage default migration unless `SESSION-004` is activated by a separate senior-owned
  gate.
- No release tag, crate publish, installer signing, or external trial invitation.

## Existing Work Inventory And Disposition

This inventory satisfies `docs/sop/START-ITERATION.md` before introducing I102-I105. It does not
rewrite existing baselines.

| Area | Current State | Disposition For This Plan |
|---|---|---|
| R27 High-Risk Governance Gate | In Progress standing gate | Keep as oversight gate; it grants no implementation authority. |
| I018 Observability/Prompt Assets | Planned historical baseline | Deferred; not selected unless diagnostics expose a bounded log-retention gap. |
| I028 Scheduled Tasks | Planned | Deferred; scheduling/autonomy is outside this delegation package. |
| I046 Architecture/Governance Repair | File status stale Planned, board/history says Complete | Do not activate; status drift remains outside this plan unless governance audit selects it. |
| I081-I083 Frontline remainder | Planned historical shells | Superseded by later 2026-07-03 and 2026-07-06 plans; do not activate. |
| I085 Model Catalog Modernization | Paused | Resume only for MC107 real terminal `/connect` walkthrough, then close or retain residual. |
| I086-I089 Product hardening | Planned | Kept planned; I102-I105 supersede only the new four-month developer operating objective. |
| Provider runtime market package | Planned | Used as input; packetized into I102-I105 with clearer delegation and monthly gates. |
| PLUGIN-001 / CMD-002 / WEB-001 / WEB-005 | In Progress owner stories | Not selected except for read-only diagnostics/docs; no runtime expansion. |
| PERM-001 | Blocked/In Progress matrix complete | Policy changes remain senior-reviewed; developers may collect traces/tests only. |
| REL-002 | Planned, not ready | Evidence can be updated; no v1.0 claim or release action. |

## Four-Month Execution Matrix

| ID | Week | Iteration | Track | Deliverable | Validation | Status |
|---|---:|---|---|---|---|---|
| D100 | 1 | I102 | Start gate | Reconfirm owner docs, stale statuses, and current provider/runtime regressions. | governance validation; `git status` recorded | Complete |
| D101 | 1 | I102 | Provider fixtures | OpenAI-compatible SSE fixture matrix covers split chunks, missing ids, `[DONE]`, usage chunks, malformed args. | provider fixture tests | Complete |
| D102 | 2 | I102 | Agent invariants | Agent rejects malformed `ToolUse` sequences without stuck processing. | agent invariant tests | Complete |
| D103 | 3 | I102 | Runtime status | TUI/conversation can distinguish model wait, tool wait, timeout, failure, and cancellation. | conversation/TUI tests | Complete |
| D104 | 4 | I102 | Closeout | Month-1 reliability evidence and residuals recorded. | workspace tests; governance | Complete |
| D110 | 5 | I103 | Connect UX | Standard providers do not ask for base URL; custom providers require it. | CLI/TUI connect tests | Planned |
| D111 | 6 | I103 | Model browsing | Large provider/model inventories remain searchable and bounded. | CLI/TUI model tests; manual evidence | Planned |
| D112 | 7 | I103 | Diagnostics | Redacted doctor output reports config, provider protocol, credential source, data dirs, validation adapters. | CLI tests; redaction check | Planned |
| D113 | 8 | I103 | Closeout | First-run docs and setup evidence are current. | docs review; governance | Planned |
| D120 | 9 | I104 | Permission evidence | Repeated-approval traces identify noise without changing deny precedence. | permission tests; trace artifact | Planned |
| D121 | 10 | I104 | Validation routing | Internal validation/project detection adapters are exercised for Rust and one non-Rust fixture. | validation tests | Planned |
| D122 | 11 | I104 | Tool display | Long output and argument display stay readable without changing model-visible payloads. | TUI/tool tests | Planned |
| D123 | 12 | I104 | Closeout | Long-session stability evidence and security residuals recorded. | workspace tests; governance | Planned |
| D130 | 13 | I105 | Trial docs | Install, first-run, provider, permission, local-data, and bug-report docs are trial-ready. | docs checklist | Planned |
| D131 | 14 | I105 | Smoke suite | Repeatable smoke checklist covers first run, `/connect`, `/model`, tool use, provider failure, resume, exit. | recorded smoke evidence | Planned |
| D132 | 15 | I105 | Self-bootstrap evidence | One Talos-primary attempt is classified honestly against REL-002. | REL-002 owner update | Planned |
| D133 | 16 | I105 | Final gate | Market-trial readiness report gives go/no-go, residual risks, rollback, and next owners. | workspace tests; clippy; governance | Planned |

## Monthly Exit Gates

| Month | Exit Criteria |
|---|---|
| Month 1 / I102 | No known provider/tool-use sequence can leave the UI processing forever without a terminal error or diagnostic trail. |
| Month 2 / I103 | A new developer can configure a standard provider, browse/select a model, and produce a redacted diagnostic bundle. |
| Month 3 / I104 | Long sessions have bounded prompt noise, readable tool output, and validation routing evidence without security-policy drift. |
| Month 4 / I105 | Trial docs, smoke evidence, REL-002 classification, and rollback instructions are complete enough for maintainer go/no-go. |

## Required Reads

Before starting any task, developers must read:

1. `AGENTS.md`
2. `docs/sop/LONG-RUNNING-TASK.md`
3. `docs/sop/ITERATION-WORKFLOW.md`
4. `docs/sop/GIT-WORKFLOW.md`
5. `docs/sop/DOC-CHECK.md`
6. `docs/BOARD.md`
7. `docs/backlog/PRODUCT-BACKLOG.md`
8. `docs/tasks/2026-07-07-provider-runtime-hardening-next-phase.md`
9. `docs/backlog/active/RUNTIME-002-turn-health-and-stuck-processing.md`
10. `docs/backlog/active/PROVIDER-002-response-reliability-timeout-retry.md`
11. `docs/backlog/active/PERM-003-permission-experience-reference-study.md`
12. `docs/backlog/active/VALIDATION-001-internal-validation-service.md`
13. `docs/backlog/active/SESSION-004-binary-session-log-format.md`
14. `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md`

## Operating Rules For Developers

- Work in ID order inside the active monthly iteration.
- Before editing, state the exact owner docs, source files, and tests expected to change.
- Use existing patterns in the affected crate. Do not introduce a new dependency without senior
  review and an owner-doc rationale.
- Behavior-facing work needs binary/runtime evidence, not only unit tests.
- Update owner docs before `docs/BOARD.md`.
- Record every residual in a backlog or iteration owner doc before closeout.
- Use one logical commit per accepted packet when commits are requested.
- Never claim a command passed unless it was run in this worktree.

## Validation Matrix

Targeted task gates are listed in I102-I105. Monthly closeout should run:

```sh
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
scripts/validate_project_governance.sh .
git diff --check
```

If a command cannot run, the closeout must record the command, failure summary, likely cause, and
the fallback validation actually run.

## Failure And Escalation Policy

Stop and request maintainer/senior review if work requires:

- permission allow/deny precedence changes;
- new runtime dependencies or dependency upgrades;
- provider credential schema or persistence changes;
- session-storage default migration;
- release, publish, tag, push, deployment, or external trial invitation;
- background watchdogs, global buses, or broad async ownership changes;
- destructive filesystem actions or secret-bearing diagnostics.

For ordinary test failures, fix within the task scope. After three materially different failed
approaches to the same problem, record the blocker in the iteration and ask for review.

## Recovery Instructions

1. Run `git status --short`.
2. Read this file, the active I102-I105 iteration, `docs/BOARD.md`, and
   `docs/backlog/PRODUCT-BACKLOG.md`.
3. Continue from the lowest-numbered Planned task in the active iteration.
4. If owner docs and board conflict, trust owner docs and update board only after the owner is fixed.
5. Re-run `scripts/validate_project_governance.sh .` after governance edits.

## Execution Log

| Date | Record |
|---|---|
| 2026-07-07 | Created planned developer operating package with I102-I105 shells, backlog/board links, and handoff prompt. |
