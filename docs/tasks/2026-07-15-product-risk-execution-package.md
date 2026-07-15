# Execution Package: Four-Month Product And Risk Plan

**Status**: Ready for assignment; no iteration activated.
**Program plan**: `docs/tasks/2026-07-15-four-month-product-risk-plan.md`.

## Start Gate

1. Read `AGENTS.md`, this package, the program plan, `docs/BOARD.md`, selected owner docs, ADRs, and `docs/iterations/README.md`.
2. Confirm no Active/Review/Planned iteration is bypassed. Record I018-I020's deferral in the new activation record.
3. Confirm a clean `main`, then run `git status -sb`, `rustc --version`, `cargo metadata --locked --no-deps --format-version 1`, `scripts/validate_project_governance.sh .`, and `./scripts/release_preflight.sh`.
4. Inspect affected crates/composition roots, state assumptions, and create one iteration plan from `docs/iterations/TEMPLATE.md`.
5. Activate that iteration only. Later packages remain inactive.

## Work Order

| ID | Output | Completion gate | Fallback |
|---|---|---|---|
| P100 | WEB-001 rendered read-only pages | Browser evidence and redaction tests | Retain API-only behavior; create a refinement finding. |
| P110 | TUI-030 in-memory composer history | TUI runtime evidence and input-priority regressions | Write a bounded design note; do not change persistence. |
| P120 | TOOL-021 error-flow matrix | Reviewed fixtures and explicit follow-up owner | Stop after report; do not implement unapproved repair. |
| P130 | TASK-001 ADR/defer/reject | Security/architecture review | Leave issue open with decision link. |
| P140 | A2A-001 ADR/defer/reject | Threat-model review | Leave issue open with decision link. |
| P150 | Synced closeout packet | Governance, Board/index, issue and residual check | Mark Partial with exact recovery step. |

## Non-Negotiable Guardrails

- P100: loopback only, redaction first, no web write/action/approval route.
- P110: no durable composer history or transcript-format change.
- P120: observe provider differences; do not normalize behavior speculatively.
- P130: resumed write actions receive fresh permission decisions; no scheduler/daemon/direct tool path.
- P140: no implicit authority, credential transfer, auto-discovery, or multi-agent runtime.

## Checkpoint Template

| Time | Package | Branch/commit | State | Evidence | Changed files | Risk/deviation | Next exact action |
|---|---|---|---|---|---|---|---|

Retry an unchanged failed command at most twice. If code and owner docs disagree, preserve code and test evidence, then synchronize the owner record before progressing.

## Initial Checkpoint

| Time | Package | Branch/commit | State | Evidence | Changed files | Risk/deviation | Next exact action |
|---|---|---|---|---|---|---|---|
| 2026-07-15 | Planning handoff | `main` | Ready | Scope, inventory, ordered packages, gates, and escalation boundaries recorded. | Plan and execution package. | No iteration is active. | Assign P100; run Start Gate; inspect `talos-dashboard` and WEB-001; create only P100's iteration baseline. |
