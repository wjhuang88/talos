# TUI-059: Permission Prompt Composer-Relative Docking

| Field | Value |
|---|---|
| Story ID | TUI-059 |
| Type | Bug / TUI / Permission Layout Story |
| Priority | P0 corrective residual from I211 |
| Status | In Progress / Claimed |
| Source | [GitHub Issue #330](https://github.com/wjhuang88/talos/issues/330) |
| Selected Iteration | I230 (claim PR #416; activation effective after merge) |
| Depends On | TUI-045/I197 merged anchor state; inline composer ownership; ADR-054 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 Sol mainline TUI-059 session |
| Work Slice | Implement only TUI-059 composer-relative permission prompt docking and deterministic layout restoration. Exclude permission semantics, request identity, persistence, provider, release, Dashboard, Desktop and `/auto`. |
| Claimed At | 2026-08-28 |
| Source Issue | #330 |
| Governance Claim PR | #416 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer-directed long-task objective; finalized claim requires exact-head CI, governance validators and independent review before merge. |
| Implementation PR | Not started |
| Last Updated | 2026-08-28 |
| Handoff / Release Condition | Claim PR #416 merged as `0f17e79d`; implementation starts from that merge or later. Protected permission-surface changes require independent security review. |

## Identity / Goal / Value

Keep a transient permission request spatially attached to the current logical composer, including
in a new session where the composer is not at the physical terminal bottom.

## Scope

- Derive permission-panel placement from the current composer/layout plan rather than terminal
  bottom coordinates.
- Preserve the non-bottom composer and triggering context when space is sufficient.
- Apply only the minimum deterministic local reflow needed to show all required choices.
- Restore prior follow-tail or anchored-history state after approve, deny, cancel, timeout or error.
- Cover queued prompts, multiline/wrapped content, narrow/short terminals and resize without blank
  growth, duplicate rows, cursor artifacts or progressive bottom drift.
- Keep running tool activity, composer and permission choices in one stable visual hierarchy.

## Exclusions

- No permission-policy, default-decision, request-identity or persistence change.
- No global composer-bottom rule, broad renderer rewrite, dependency, release or publication work.

## Evidence And Dependency Facts

Issue #302 natural-person checkpoint `5341637918` found the permission selector below the composer
while running activity remained above it and left the required #125 terminal matrix incomplete.
The maintainer separately observed a new-session, non-bottom composer with the permission request at
the physical terminal bottom. I197 final head `9fce4f13` merged as `d98f37e7`; its owner remains
Review.

## Acceptance For Behavior

- Given a new session whose composer is above the terminal bottom, when a permission request opens,
  then the panel is adjacent to that composer and does not jump to the physical bottom.
- Given insufficient height, when the panel opens, then only the minimum required viewport reflow
  occurs and all choices remain visible.
- Given approve, deny, cancel, timeout or error, when the panel closes, then the prior logical
  composer/viewport relationship and follow-tail state are restored.
- Repeated prompts, wrapped descriptions, multiline drafts, narrow/short terminals and resize
  remain usable without overlap or drift.
- Permission semantics and request identity remain unchanged.

## Required Reads

- `docs/backlog/active/TUI-045-permission-prompt-layout-anchor.md`
- `docs/iterations/I197-tui045-permission-prompt-anchor-stability.md`
- `docs/decisions/054-alternate-screen-app-owned-transcript-rendering.md`
- Issue #125
- Issue #302 comment `5341637918`

## State / Status Owners

- Story scope and acceptance: this file.
- Remote corrective report: Issue #330.
- Failed source evidence: VALIDATION-002/I211 and Issue #302.
- Derived views: Product Backlog, Board and Issue status matrix.

## User-Facing Documentation

Update TUI behavior documentation in the future implementation iteration. This intake changes no
runtime behavior.

## Residual Destination

General panel docking or permission-policy changes require separate owners; TUI-058 owns only
tool-activity correlation and unnamed outcome rows.
