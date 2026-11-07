# TUI-ELLIPSIS-001: Tool Result Clipping And Muted Background

> Document status: Planned / Unclaimed
> Deliverable: tool-result logical lines fit terminal width with ellipsis and muted default input/user-message background.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Talos / gpt-6-astra |
| Work Slice | Tool-result width clipping and Nord input/user-message background, focused tests and README; excludes poll grouping, spinner lifecycle and queued input |
| Claimed At | Not applicable |
| Source Issue | None |
| Governance Claim PR | Pending |
| Authorization Mode | Independent review |
| Authorization Evidence | User authorized governance PR publication; independent review and exact-head CI remain required |
| Implementation PR | Not started |
| Last Updated | 2026-11-08 |
| Handoff / Release Condition | Effective target claim, implementation tests, standard preflight, native visual acceptance, exact-head CI and review |

## Acceptance

Given a tool-result logical line exceeding available terminal cell width, render
one clipped row ending in `…`, without automatic continuation rows. Include the
prefix in the budget; handle zero/narrow widths, CJK and combining characters.
Given explicit newlines, preserve logical rows within existing summary and
head/tail limits. Preserve diff/todo styles and synchronize segments with text.
Raw tool results and exports, tool-call arguments and chat wrapping are unchanged.
Given default Nord theme, input/user-message background is `#353B49`; other themes
and explicit user configuration remain unchanged.

## Scope and required reads

This changes completed TUI-034 continuation-row acceptance for tool results;
retain its historical completion evidence rather than reopen it.
Hard: no runtime, permissions, sandbox, storage, dependency or public API change.
Exclude spinner, queued input, background wakeup and polling/grouping changes.
Soft: reuse existing display-width truncation. Required reads: TUI-034 owner,
tool_display.rs, theme.rs, scrollback width helpers, collaboration/testing SOPs.
Documentation: README.md. Residual destination: this owner.

## Selection and dependencies

Preparation target: ad71a208905b4354d1485e73edaa8b884ccf6fd7.
Target TOOL-ROUTE-001 inventory retains I310 Active, I318/I302/I277 Review,
I249 Planned/deferred and I164 Paused; no Blocked header reported there.
TUI-POLL-001 owns polling projection; TOOL-ROUTE-001 owns tool-routing guidance.
Neither scope transfers here. Recheck owner inventory before activation.
Open PRs checked: #861 search helpers, #849 hosted MCP research, #838 Gemini;
none claims this clipping/background behavior. This task is an independent slice,
not a replacement objective for an existing iteration.

## Validation and delivery boundary

Run focused locked TUI tests for ordinary, summary, head/tail, narrow widths,
Unicode, styles and raw-content preservation. Run pinned standard preflight.
Native walkthrough: long output, explicit multiline output, resize and colors.
Both governance validators must pass before claim review.

Original worktree has an uncommitted experiment: 605 TUI tests and foreground
standard preflight passed with C locale, two test threads and 32768 MiB address
limit. This is preliminary, not evidence for a future rebased implementation head.
An earlier TUI failure lacked retained detail and remains unexplained. Native
interactive acceptance is pending. No implementation belongs in this claim PR.
Backfill the actual PR number and propose Claimed / Active atomically before
review; ownership becomes effective only after target merge. Completion requires
existing implementation commit evidence, not this document or validation alone.