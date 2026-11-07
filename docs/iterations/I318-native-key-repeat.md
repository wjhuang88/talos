# Iteration I318: Native TUI Key Repeat

> Document status: Active — proposed atomic activation, ineffective until target merge
> Published plan date: 2026-11-07
> Planned objective: deliver Issue #269 native repeat-safe editing/navigation with one-shot and focus isolation.
> MVP deliverable: the real Talos TUI consumes backend Repeat for safe actions without a timer; submission/approval remain single-shot.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | I318 / TUI-054 native editing/navigation repeat routing and one-shot/focus isolation |
| Claimed At | 2026-11-07 |
| Source Issue | #269 |
| Governance Claim PR | #869 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer explicitly requested implementation while #38 CI runs; independent input/approval-boundary review and exact-head CI required |
| Implementation PR | Not started |
| Last Updated | 2026-11-07 |
| Handoff / Release Condition | Effective only after atomic claim merge; complete implementation and validation, retain Review for unmet real-terminal acceptance |

## Published Baseline

Selected Story: TUI-054 / #269 (previously Intake / Unclaimed). Dependencies: existing Crossterm
keyboard reporting and completed TUI-053 numeric approval behavior. No public API or policy change.

### Inventory and disposition

Current owner-frontmatter inventory on main@5b7d2c0a2: I164 Paused (superseded startup target), I249
Planned (dependency pilot, deferred), I277 Review (human/device acceptance retained), I302 Review
(Windows resource evidence retained), I310 Active (Gemini PR #838, separate owner), I317 Active
(search ownership PRs #860/#861, separate owner). All retain status/scope; no overlapping TUI input
claim/PR. #38 governance PR #868 waits for CI independently. User explicitly authorizes concurrent
non-overlapping #269 implementation; no unrelated task activation or global status repair.

### Scope and acceptance

- Composer Backspace/Delete, Left/Right and applicable navigation consume native repeat cadence.
- Shared action classification covers composer, menus/model/provider pickers and viewport controls;
  preserve each panel's wrap/clamp behavior, credentials and wizard editing boundaries.
- Enter/submit, approval, confirmation, command execution, mutation, Escape and modal transitions
  remain one-shot. Repeats/releases never approve the next request or submit twice.
- Focus/modal replacement, paste, resize and terminal focus changes invalidate stale repeat ownership;
  a held key cannot act on a newly focused panel. No application-owned repeat timer.
- Unicode/CJK/emoji/combining edits respect grapheme boundaries and empty/start/end no-op behavior.
- Preserve IME text/composition semantics and document backend limitations honestly: Press-only
  terminals cannot distinguish an OS-generated repeated Press from separate physical key presses.
- Exercise actual binary terminal event routing; record primary plus materially different terminal
  physical hold/release and non-English IME evidence before full completion.

### Validation and documentation

Focused TUI routing/state regression tests, pinned Rust 1.99.0 locked workspace standard
`./scripts/release_preflight.sh`, warnings-denied checks, exact-head CI and independent review of
approval/focus safety. Windows cross-checks use `cargo xwin ... --target x86_64-pc-windows-msvc
--locked` per LOCAL-DEV, not GNU. Native Windows execution is separate.
Update `crates/talos-tui/README.md`, owner, iteration index, Backlog, Board and Issue evidence.

### Risks and rollback

Missing terminal event metadata cannot be repaired by inventing timing heuristics. Focus identity
must not retain credential content; navigation repeats must not gain mutation authority. Preserve
existing conservative handling for unknown events. Roll back routing as a coherent change if it
fails one-shot safety; do not weaken approval tests to enable repeat.

## Execution / evidence

Claim-stage only: no implementation, tests or device acceptance claimed. Atomic claim and activation
become effective together on target merge. Real-terminal evidence unavailable here must remain a
tracked unmet row in this owner, not a complete claim or a silent scope reduction.

## Terminal and implementation preparation — 2026-11-07

Maintainer primary terminal: **Ghostty, no tmux** (explicit session answer). Physical hold/release
and non-English IME observations remain pending; a second materially different terminal/platform
is still required. Crossterm 0.29 source `event.rs` defines Press/Repeat/Release and documents
REPORT_EVENT_TYPES; Talos already requests that flag in `inline_terminal.rs`. Source inspection
is protocol evidence, not native keyboard acceptance.

Implementation choice: reuse the already-locked Rust `unicode-segmentation` 1.13.3 package (currently
used by Desktop) for complete grapheme boundaries instead of custom Unicode approximations; an
explicit direct TUI dependency and lockfile edge will be reviewed. No version upgrade or native code.
Repeat ownership will use content-free panel identity, never credential/provider input snapshots.
