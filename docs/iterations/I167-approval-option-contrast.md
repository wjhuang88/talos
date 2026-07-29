# Iteration I167: Approval Option Contrast

> Document status: Review
> Published plan date: 2026-07-29
> Planned objective: Make unselected interactive approval choices clearly readable without changing approval behavior, layout, or terminal lifecycle.
> Baseline rule: preserve this target; changed targets use a new iteration ID.
> MVP deliverable: A rebuilt Talos TUI renders every approval choice with readable contrast and retains the selected-row distinction.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `TUI-040` | None | Ready | TUI-008 and I156 Complete; no competing Active/Review iteration | Unselected approval choices use the readable primary semantic foreground. |

### Scope

- Correct only unselected approval-option foreground styling.
- Add buffer-level style regression coverage for selected and unselected rows.
- Verify the existing narrow/wide approval option rendering remains bounded.

### Non-Goals

- No change to approval semantics, permission policy, keyboard handling, layout, renderer ownership,
  theme architecture, dependencies, or release state.

### Acceptance

- Given an approval panel with multiple choices, when rendered, then every unselected actionable
  option uses `semantic::TEXT_PRIMARY`, not `semantic::DIM_TEXT`.
- Given an approval panel, when the selection changes, then the selected row remains visually
  distinct through its existing accent/background treatment.
- Given 40- and 80-column approval panels, when rendered to a buffer, then all visible option rows
  remain present, bounded, and readable.

### Planned Validation

```bash
cargo test --locked -p talos-tui
cargo fmt --all -- --check
cargo check --workspace --locked
cargo clippy --workspace --locked -- -D warnings
cargo test --workspace --locked
scripts/validate_project_governance.sh .
git diff --check
```

### Runtime Evidence

Build the current CLI binary and open an approval prompt in an interactive terminal. Verify that
all choices are visible before navigation and that selected/unselected states remain distinguishable
after Up/Down movement.

### Documentation To Update

- TUI-040;
- this iteration;
- `docs/iterations/README.md`;
- v0.6 program and four-month package checkpoint;
- `docs/backlog/PRODUCT-BACKLOG.md` and `docs/BOARD.md`.

### Risks And Rollback

- Risk: a broad theme change would alter unrelated UI surfaces.
- Rollback: restore the single approval unselected-style line; no state or persistence changes exist.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-29 | Activation | Inventory at `1cf9f6ffea17b4afc808a0c23135efcef365016c`: I157/MODEL-010 Complete; I164 Paused; I158-I162 Blocked; ADR-053 Proposed; no Active or Review implementation iteration. TUI-040 is Ready, has no unresolved dependency, and I167 is the sole Active implementation authority on direct `main`. |
| 2026-07-29 | Implementation | `3356aac52e755a29c4bfbdd43854c47e851569d9` changes only the unselected approval-option foreground from muted metadata to `semantic::TEXT_PRIMARY`. It preserves the selected accent/background treatment and approval behavior. |

## Verification Evidence

- Focused tests: `cargo test --locked -p talos-tui test_approval_render_unselected_options_use_readable_primary_foreground` passed (1 test); `cargo test --locked -p talos-tui --lib` passed (484 tests).
- Full locked validation: `cargo fmt --all -- --check`, `cargo check --workspace --locked`, `cargo clippy --workspace --locked -- -D warnings`, and `cargo test --workspace --locked` passed. The latter required the normal local test-server permission boundary; in the restricted sandbox the unrelated CLI server tests fail with `Operation not permitted`, then passed outside that sandbox.
- Build: `cargo build --locked -p talos-cli` passed; rebuilt binary is `target/debug/talos` from `3356aac`.
- Governance validation: `scripts/validate_project_governance.sh .` passed with 0 warnings; `scripts/assess_project_scale.sh .` reports the existing high-risk/release-managed/on-demand profile.
- Runtime evidence: pending maintainer visual check in a real terminal. Trigger an approval prompt, confirm all choices are readable before navigation, then use Up/Down to confirm selected and unselected rows remain distinguishable.

## Completion Evidence

- Implementation Commit: `3356aac52e755a29c4bfbdd43854c47e851569d9`.
- Completion Commit: pending real-terminal visual acceptance.
- Do not cite a status-only documentation commit as implementation completion.
- Keep Review until the runtime visual check passes; do not use this documentation commit as implementation completion.

## Variance And Residuals

- 2026-07-29 priority correction: I158 cannot start because ADR-053 remains Proposed. The maintainer
  reported an independently actionable approval discoverability defect, so I167/TUI-040 was selected
  as a bounded corrective iteration. The I158-I162 dependency chain remains unchanged.

## Retrospective

- Outcome: automated acceptance achieved; manual visual acceptance remains pending.
- Documentation: owner, Board, program, package, and iteration index synchronized to Review.
- Lessons: actionable menu choices must not reuse muted metadata styling merely because they are unselected.
