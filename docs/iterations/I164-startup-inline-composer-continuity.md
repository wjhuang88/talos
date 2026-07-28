# Iteration I164: Startup Inline Composer Continuity

> Document status: Active
> Published plan date: 2026-07-28
> Planned objective: Restore a compact new-session composer position approximately two rows below the Alternate-Screen Logo, then transition cleanly to the existing full-frame conversation layout after the first submitted user message.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: A rebuilt `talos` TUI shows an editable startup composer near the Logo, preserves it through first-draft redraws, and transitions to normal history/composer layout after first submission without transcript or resize regressions.
> Activation rule: satisfied 2026-07-28; I163 is Complete and the activation record below makes I164 the sole implementation authority.

## Published Baseline

### Iteration Inventory And Disposition At Selection

| Iteration | State on 2026-07-28 | Disposition |
| --- | --- | --- |
| I163 / SKILL-003 | Active; completion review ready | Remains the sole implementation authority until Complete or Paused. |
| I157 / MODEL-010 | Planned | Preserved unchanged; explicitly deferred by maintainer priority shift to I164 after I163 disposition. |
| I158-I162 | Blocked | Remain blocked; no dependency or status change. |
| I164 / TUI-038 | Planned | Selected now; activate only after I163 disposition and a fresh dependency inventory. |

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
| --- | --- | --- | --- | --- |
| TUI-038 | None | Ready | I163 disposition; TUI-035 Complete; ADR-054 Accepted | Fresh-session composer sits two rows below Logo and transitions once after first submit. |

### Authorized Scope

- implement only TUI-038's startup-layout branch in the existing Alternate-Screen full-frame renderer;
- place the composer two display rows below the Logo at normal usable heights before the first submitted user message;
- use `AppLayout`-owned bounded rectangles and final cursor targets;
- transition to the existing normal layout on first submit;
- add focused projection/layout/input-entry/full-frame tests and real rebuilt-binary evidence;
- update the TUI startup documentation, TUI-038, this iteration, Board, and program execution record.

### Non-Goals

- no Primary Screen/native scrollback/DECSTBM/reverse-index work;
- no second renderer, transcript/session/export/provider/tool behavior change;
- no post-first-submit composer redesign, dashboard work, or logo-artwork change;
- no I157 implementation, I158-I162 activation, publish, tag, release, Desktop work, or workspace-version change.

### Acceptance

TUI-038 owns the behavioral acceptance. The iteration may not be completed
without a rebuilt-binary Alternate-Screen walkthrough that proves initial
composer placement, first submit transition, resize, narrow/short fallback,
CJK/multiline editing, wheel navigation, and terminal restore.

### Planned Validation

```bash
cargo test --locked -p talos-tui
cargo fmt --all -- --check
cargo check --workspace --locked
cargo clippy --workspace --locked -- -D warnings
cargo test --workspace --locked
scripts/validate_project_governance.sh .
git diff --check
cargo build --locked -p talos-cli
```

### Runtime Evidence

Use the actual rebuilt binary in Alternate Screen mode. At normal height,
confirm two spacer rows between Logo and composer; type/edit a first CJK and
multiline draft; submit once and confirm normal layout transition; resize before
and after submission; test narrow/short fallback; wheel through Logo/history;
then exit and confirm terminal restore.

### Risks And Rollback

- Risk: startup visual rows leak into TranscriptStore or use a second geometry
  source, breaking anchors/resize.
- Mitigation: one frame-size snapshot, `AppLayout` rectangles, and transcript
  invariants before/after transition.
- Rollback: remove only the startup-layout branch and retain current ADR-054
  full-frame behavior.

## Actual Activation And Execution

| Date | Type | Record |
| --- | --- | --- |
| 2026-07-28 | Planning / priority shift | Maintainer requested immediate implementation scheduling. I164/TUI-038 is selected as the next candidate after I163 disposition; I157 remains Planned but deferred. No production code changed and I164 is not Active. |
| 2026-07-28 | Activation | I163/SKILL-003 is Complete. TUI-035 is Complete and ADR-054 is Accepted. Fresh inventory confirms I157 remains Planned, I158-I162 remain Blocked, ADR-053 remains Proposed, and no conflicting Active iteration exists. Baseline `c600110`. I164/TUI-038 is the sole implementation authority. |
| 2026-07-28 | Maintainer execution-mode decision | No implementation work is running in parallel. The maintainer explicitly authorized I164 execution directly on `main`; the unused `feature/i164-startup-inline-composer` worktree and branch were removed before any implementation change. If parallel work begins, re-run the inventory and restore the required isolation before continuing. |

## Verification Evidence

- Planning/governance validation: activation inventory complete; repository
  governance validation is replayed in the activation commit.
- Focused tests: pending implementation.
- Full locked validation: pending implementation completion.
- Rebuilt-binary runtime evidence: pending activation.

## Completion Evidence

- Prerequisite I163 Completion Commit: `12ef1e3`.
- Completion Commit: pending implementation.
- A status-only documentation commit cannot satisfy completion evidence.

## Variance And Residuals

- The maintainer priority shift changes execution order only. I157's published
  baseline remains intact and its owner document records the deferral.

## Retrospective

- Outcome: pending activation.
- Documentation: pending implementation.
- Lessons: pending implementation.
