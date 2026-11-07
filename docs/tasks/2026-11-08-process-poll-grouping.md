# TUI-POLL-001: Group Background Process Polling

> Document status: Review / Claimed
> Published plan date: 2026-11-08
> Deliverable: consecutive native process read/status calls for the same job occupy one live TUI summary, with errors and completion prominent.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / gpt-6 |
| Work Slice | TUI-only consecutive same-job process read/status grouping, expandable raw call/result presentation, and failure/terminal visibility |
| Claimed At | 2026-11-08 |
| Source Issue | None |
| Governance Claim PR | #873 |
| Authorization Mode | Independent review |
| Authorization Evidence | User screenshot and explicit confirmation to implement same-job read/status summary with state, poll count and output amount; independent Agent review and exact-head CI required |
| Implementation PR | Not started |
| Last Updated | 2026-11-08 |
| Handoff / Release Condition | Focused rendering/interaction regressions, standard preflight, MSVC xwin tests, independent review and exact-head CI before closure |

## Goal and acceptance

Given consecutive native process read/status calls for one job, show a single
collapsed summary updating latest state, polling count and observed output amount.
Given user activation of the summary, expand/collapse original call/result rows.
Given a different job, intervening chat/tool, malformed result, error or terminal
state, preserve ordering and display important failure/completion independently;
never merge unrelated jobs or conceal an error. Same behavior in live and restored
history, stable resizing/scrolling, and no changes to raw session/export/model data.
Output amount describes observed reads, not a claim of complete process output.
A successful process-tool call is not itself job completion.

## Constraints and scope

Hard: presentation-only; no execution, scheduling, cancellation, permission or
persistent format changes; no public API or dependency changes. Preserve ADR-019
scrollback boundary and current history projection/selection behavior.
Soft: only consecutive calls are grouped; summary is collapsed by default, click
uses existing history expansion interaction. Retain raw call/results for expansion.
Non-goals: grouping arbitrary tools, changing background auto-resume, CLI plain
output, Desktop, or the unrelated local ellipsis/theme experiment.
Required reads: `docs/sop/AGENT-COLLABORATION.md`, `docs/sop/TESTING.md`,
`docs/reference/ARCHITECTURE.md`, `crates/talos-tui/src/transcript.rs`,
`crates/talos-tui/src/history_projection.rs`, `crates/talos-tui/src/app/output.rs`.
Documentation: README.zh-CN.md; owner first then Board. Residual destination: here.

## Inventory and isolation

Base d4d11f5a includes I318 #872 implementation. I318 Review retains physical
keyboard acceptance; I310 Active Gemini retained; I302 Review handle evidence
retained; I277 Review deferred device acceptance retained; I249 Planned deferred;
I164 Paused superseded retained. No Blocked iteration header found. I317 Complete
retained. Open #838 Gemini, #849 search intake and #861 search helpers do not own
this TUI slice; recheck before merge. Existing uncommitted README, theme,
tool_display and ellipsis proposal remain untouched in the original worktree.
Separate worktree `/tmp/talos-process-grouping` isolates this task. Existing
high-risk governance profile retained; no workflow/profile redesign.

## Validation and closure

Test 20+ same-job polls collapse, expansion preserves details, different jobs and
intervening content split groups, malformed/provenance mismatch stays ordinary,
error and terminal state remain prominent, replay matches live, narrow/wide resize
and selection stay valid. Run pinned locked TUI tests, cargo-xwin MSVC with --tests,
and standard release_preflight.sh. Independent Agent review plus exact-head CI
and merge-time CAS before merge. Complete only with existing implementation SHA.

## Local implementation checkpoint

Claim #873 merged as `1a2c184d` after exact-head independent Agent APPROVE and
CI42391413797 passed. Implemented projection-only grouping with immutable raw
transcript entries; click expansion reuses existing history interaction. Terminal
observations remain separate and show actual state/exit code. Error/malformed
results and ambiguous parallel-call batches remain ordinary rows. Text-byte counts
include repeated reads; truncation/gaps/partial reads stay visible.
Initial TUI suite passed 626 tests; final added parallel-pairing/frame regressions,
preflight, xwin and independent implementation review remain pending. Original
worktree ellipsis/theme edits were not touched.

Independent preliminary review found malformed optional metadata was silently
defaulted during grouping. Corrected parsing to reject invalid types and retain
ordinary display; added read/status metadata regressions. First preflight stopped
at claim-reference formatting (PR field must be exactly #873); corrected the
field without changing claim provenance. Failed log: /tmp/talos-poll-preflight.log.

## Converged local candidate

Standard `./scripts/release_preflight.sh` passed in
`/tmp/talos-poll-preflight2.log`: governance, format, locked workspace check/Clippy,
workspace tests/doc-tests and both external SDK modes. TUI 628/628 passed including
real frame/click/drag, same-job collapse, malformed metadata, parallel-call
isolation, terminal/error boundaries, replay parity and resize/selection anchors.
`cargo xwin check --locked -p talos-tui --tests --target x86_64-pc-windows-msvc`
passed; this is compile evidence, not native execution. Supplemental independent
Agent review found no remaining blockers. Exact-head review/CI and merge remain.
