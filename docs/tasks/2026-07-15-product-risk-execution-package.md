# Execution Package: Four-Month Product And Risk Plan

**Status**: P100-P120 Complete (I129/I130/I131, 2026-07-16). P130-P150 inactive.
**Program plan**: `docs/tasks/2026-07-15-four-month-product-risk-plan.md`.
**Unattended authority**: `docs/tasks/2026-07-15-product-risk-unattended-authorization.md`.

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
| 2026-07-15 | P100 Start Gate | `main` (uncommitted) | Gate passed; SESSION-005 fix v2 applied | Start Gate ran: clean tree, rustc 1.97.0, cargo metadata OK, governance 0 warnings, release_preflight initially failed on I128 durable concurrency test (SESSION-005), fix applied. Dashboard inspected: 6 GET-only routes returning JSON/plain-text, no rendered HTML pages. I018-I020 disposition recorded. I129 baseline created (Planned). | `crates/talos-session/src/durable.rs`, `docs/backlog/active/SESSION-005-*.md`, `docs/iterations/I129-*.md` | Pre-existing I128 SQLite WAL race found and fixed. Non-terminal inventory stale for I019/I020 (both Complete). | Await architecture review before marking I129 Active. |
| 2026-07-15 | P100 arch review v1 | `main` (uncommitted) | Fixes applied; awaiting re-review | Architecture review found: (1) SESSION-005 had expect() panic, wrong retry time semantics, missing tests; (2) I129 had /extensions scope creep, risky Accept default, stale checkpoint. All addressed: removed expect() and deferred busy_timeout after init (bounded ≤500ms), added deterministic tests for non-BUSY + retry exhaustion; removed /extensions, tightened to conservative JSON-default negotiation; fixed checkpoint text. SESSION-005 status → Review. | Same files | No format/API/dependency/permission change confirmed. | Re-run full validation ladder; submit for re-review. |
| 2026-07-15 | P100 complete | `main` (uncommitted) | I129 Complete | All validation green: fmt, check, clippy, test (40 dashboard tests), governance 0 warnings, git diff clean. Browser evidence: 4 HTML pages rendered with nav, empty states, redaction (api_key=***), content negotiation (JSON default preserved). /extensions stays JSON-only. | `crates/talos-dashboard/src/lib.rs`, `docs/iterations/I129-*.md`, `docs/backlog/active/WEB-001-*.md`, `docs/BOARD.md`, `docs/iterations/README.md` | No new deps, no API/format/permission change. | Commits 17dbe60 + e51b4b6 pushed to origin/main. All validation green. I129 Complete. |
| 2026-07-16 | P110 architecture acceptance | `6deae69`, `6e83efc`, `dd76d2a` | I130 Complete | Architecture re-review accepted nine state tests and five entry-point tests that inject `Event::Key(Up/Down)` through `handle_input_event`, proving navigation, multiline draft restoration, and slash-menu/approval/credential priority guards. Locked workspace validation, release preflight, governance, and diff checks passed. | `crates/talos-tui/src/state.rs`, `crates/talos-tui/src/app.rs`, `crates/talos-tui/src/inline_terminal.rs`, `crates/talos-tui/src/state_tests.rs`, `crates/talos-tui/src/app/app_tests.rs` | No deps/API/format/persistence or permission change. | P110 closed. Keep P120-P150 inactive until a new P120 iteration baseline and Start Gate are completed. |
| 2026-07-16 | P120 complete | `main` (uncommitted) | I131 Complete | Evidence-first audit: 9 fixture tests (3 OpenAI + 3 Anthropic + 3 compaction) prove every tool-error path preserved or explicitly rejected. No silent loss. 2 findings: orphan provider difference (observation), provider error loses unpersisted results (caller-dependent). No code fixes (guardrail). | `docs/reference/TOOL-021-*.md`, `crates/talos-provider/src/*.rs`, `crates/talos-agent/src/compaction/tests.rs` | No production code changed. | Commit and push; do not activate P130. |
| 2026-07-15 | P100 architecture acceptance | `main` (uncommitted) | Pass; I129 Active | Independent review accepted SESSION-005 and I129. Session crate tests (155), concurrent-open repetition (5), release preflight, governance, and diff checks passed. SESSION-005 is Complete. Maintainer granted unattended authority, including bounded Defer/Reject defaults for P120/P130/P140 and browser-test fallback. | SESSION-005/I129/Board/index/WEB-001/authority docs | No format/API/dependency/permission change. | Implement P100 only; retain all later packages inactive. |
