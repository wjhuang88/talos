# Iterations

## Purpose

Track current iteration plans, execution state, verification evidence, and retrospectives. Each
iteration's own document is authoritative for its scope and lifecycle.

The complete pre-closeout index is preserved unchanged at
[`archive/ITERATIONS-INDEX-pre-I170-closeout-2026-08-01.md`](archive/ITERATIONS-INDEX-pre-I170-closeout-2026-08-01.md).
That snapshot is historical evidence and not current activation authority.

## Lifecycle

1. **Planned** — objective, selected stories, acceptance and activation gate are published.
2. **Active** — explicitly activated work is in progress on a fresh current-main branch.
3. **Review** — implementation exists and required evidence/review is pending.
4. **Complete** — verification, acceptance, completion commit and retrospective are recorded.
5. **Paused / Blocked / Superseded** — non-active states with an explicit resume or replacement gate.

## Rules

- Every iteration has a unique ID.
- Published baselines are not silently repurposed; changed objectives use a new ID.
- Before activation, inventory current Issues, PRs, branches, owner docs and other non-terminal work.
- Ready/Planned does not mean Active.
- Recovery branches and PRs are provenance only unless a new current-main plan explicitly says otherwise.
- Complete requires runtime/acceptance evidence appropriate to the scope, not unit tests alone.

## Current Operating Set

| ID | Codename | State | Activation / Completion Gate |
|---|---|---|---|
| I169 | Transactional Batched Steering Turn | Planned — prerequisite satisfied; not active | TUI-044 is Ready. Re-read current facts, confirm no overlap, create a fresh branch from exact current `main`, record explicit activation, keep ADR-056 Proposed, and leave recovery PR #120 immutable. |
| I158 | Tool Registration Composition Consolidation | Review | Resolve scheduler/status contribution exception ownership and final architecture/tool-extension/finding documentation before Complete or Paused. |
| I159 | `talos-tools` Lightweight Feature Boundary | Blocked | Requires I158 Complete/Paused and a recorded TUI-037 disposition. |
| I160 | Shared CLI And Runtime Internal Composition | Blocked | Requires I159 Complete. |
| I161 | Sandbox Fallback And Coding Preset | Blocked | Requires I160 Complete and an independent security-review plan. |
| I162 | v0.6 SDK Fixture And Publication Readiness | Blocked | Requires I161 Complete and explicit readiness authorization; no real publish/tag/release. |

## Completed This Closeout

| ID | Codename | Final State | Completion Evidence |
|---|---|---|---|
| I170 | Windows Workspace Validation Unblocker | Complete (2026-08-01) | PR #126 squash-merged at `592254d73a98166df48da0139a02df67e9cd2cd6`; exact implementation Head `8cfe8edb2dbda581244f583fb809591391a54298`; CI run `30705366763`; walkthrough artifact `8820174164`; TOOL-023-A/C Complete; ADR-057 Accepted. |

I170's accepted residuals remain explicit:

- timeout cleanup is guaranteed for the direct shell child, not the complete descendant tree;
- TOOL-023-B still owns timeout default/configuration;
- a PowerShell lexer/parser, PowerShell 7 selection and Job Object lifecycle require separate decisions;
- I170 completion satisfies I169's prerequisite but does not activate or implement I169.

## Recent Non-Terminal / Completed Context

| ID | State | Notes |
|---|---|---|
| I168 | Complete (2026-07-30) | Provider terminal outcome integrity; completion commit `86262d02`. |
| I167 | Complete (2026-07-29) | Approval option contrast; implementation `3356aac`. |
| I166 | Complete (2026-07-28) | Interrupt shortcut reliability; automated and maintainer Alacritty acceptance passed. |
| I165 | Complete (2026-07-28) | Growing conversation composer continuity; all human acceptance cases passed. |
| I164 | Paused (2026-07-28) | Startup-inline target superseded; no Completion Commit. |
| I163 | Complete (2026-07-28) | Policy-controlled linked skill discovery. |
| I157 | Complete (2026-07-30 correction) | Provider removal/credential clear stale-snapshot concurrency correction. |
| I156 | Complete (2026-07-27) | Narrow-viewport and resize robustness; maintainer Alacritty walkthrough passed. |

## I169 Activation Checklist

I169 may be selected only after all of the following are true at activation time:

- [ ] current `main` SHA and repository state have been re-read;
- [ ] Issue #119, TUI-044, I169 and ADR-056 are re-read;
- [ ] no overlapping active steering/session implementation or newer owner exists;
- [ ] a fresh implementation branch is created from exact current `main`;
- [ ] I169/TUI-044/Board are changed from Planned/Ready to Active before code mutation;
- [ ] recovery PR #120 and branch `recovery/pr-68-i169-20260731` remain unchanged;
- [ ] Windows/macOS exact-head CI and a rebuilt real-TUI acceptance plan remain part of the implementation gate.

## History

The prior full iteration registry and non-terminal inventory remain available at:

- [`archive/ITERATIONS-INDEX-pre-I170-closeout-2026-08-01.md`](archive/ITERATIONS-INDEX-pre-I170-closeout-2026-08-01.md)

Individual plans and completion records remain under `docs/iterations/`; this compact index does not
replace or rewrite them.
