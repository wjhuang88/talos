# VALIDATION-DISPLAY-001: Readable Validation Tool Results

> Document status: Active / Claimed (proposed; ineffective until claim #882 merges)
> Deliverable: readable validate_project plan and run results in live and replayed TUI history, with raw evidence preserved.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / gpt-6 |
| Work Slice | validate_project result presentation and regression coverage; new formatter plus minimal TUI wiring; excludes width clipping, theme, execution, permissions and polling |
| Claimed At | 2026-11-08 |
| Source Issue | None |
| Governance Claim PR | #882 |
| Authorization Mode | Independent review |
| Authorization Evidence | User requests fixing unreadable validation tool output; independent Agent technical review and exact-head CI required |
| Implementation PR | Not started |
| Last Updated | 2026-11-08 |
| Handoff / Release Condition | Focused display/replay/raw-preservation tests, standard locked preflight, MSVC xwin tests, exact-head review/CI and merged evidence |

## Acceptance and boundaries

The screenshot exposes compact JSON as a large wrapped block. Plan output must
explicitly say commands have not executed, show profile, required/optional checks
and findings as logical lines. Run output must show recorded statuses, failures,
permission decisions and useful diagnostics without claiming skipped/denied checks
passed. Preserve the original structured content for model, storage and export.
Malformed or unrelated results retain the ordinary fallback. Live results and
history replay use the same presentation path. No generic JSON reformatting of
unrelated tools. Existing clipping and head/tail limits remain authoritative.

Hard: no permission, validation execution, persistence schema, dependency or
public API break. Soft: use a display-only formatter at the existing rendering
boundary. Existing CLI text renderers are reference behavior, not a reason to
replace machine-readable evidence. Required reads: validation tool and renderers,
TUI result/replay/export paths, collaboration/testing/local-development SOPs.

## Inventory and isolation

Base 1f537340. I310 Active retained; I318/I302/I277 Review retained; I249 Planned
deferred; I164 Paused superseded; I162 Complete retained. No Blocked iteration.
AGENT-PAUSE-001 is independently in Review (#881). TUI-ELLIPSIS-001 owns width
clipping and theme; this task only adds content formatting before existing width
handling, with tests in its own module. Preserve original dirty files and do not
edit its width/style behavior. Other open PRs #838/#849/#861 do not overlap.
Use /tmp/talos-validation-display; existing governance profile is unchanged.

## Validation and closure ledger

Artifacts: formatter, minimal rendering integration, tests, README.zh-CN.md and
this owner. Owner first, Board mirror; no source Issue. Validate plan/run/error,
Unicode, malformed data, replay, export/raw preservation and width integration.
Run pinned locked release_preflight.sh and cargo xwin check with MSVC --tests for
affected crates; native CI remains required. Run both governance validators.
Review staged diff and exact head; recheck claim/CAS before merge. Completion
requires existing merged implementation SHA. Residual destination: this owner.
