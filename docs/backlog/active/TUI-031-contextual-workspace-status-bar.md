# TUI-031: Contextual Workspace Status Bar

**Status**: Complete (I141, 2026-07-18)
**Priority**: P2
**Source**: Maintainer request 2026-07-17
**Relates To**: TUI-011, TUI-017, TUI-018, TUI-028, MODEL-007

## Identity / Goal / Value

Make the single-line TUI status bar answer the operator's immediate questions
without opening a command: which model/variant is active, what workspace and
Git state are in use, what platform is running, and how much context is left.

The target information hierarchy follows the supplied reference image, while
remaining Talos-native and width-aware:

```text
<model> · <variant>  >  <workspace>  >  <git branch + dirty count>  >
<platform>  <context-used % / context-limit>  <existing queue/phase>
```

## Current Capability And Gap

`scrollback_status` already shows model/provider, workspace path, output usage,
context limit/percentage, queue count, and terminal phase. It does not display
selected variant, Git branch/working-tree summary, or a platform label, and its
layout does not reserve priority tiers for those fields.

## Scope

- Render the active model and selected variant as separate, concise fields.
  Until MODEL-007 supplies variants, omit the variant field rather than invent
  a value.
- Render a display-safe workspace path.
- Read a bounded Git summary for the configured workspace: branch name or a
  detached indicator, plus a bounded dirty count/dirty marker. A non-Git,
  unreadable, or unavailable repository must degrade to omission, never block
  the UI, spawn an unbounded command, or print raw Git errors.
- Render a compile-time platform label (for example `macOS`, `Linux`, or
  `Windows`), not host identity, username, device name, or OS version.
- Render context consumption as `used% / limit` using existing token accounting;
  preserve the current unknown-usage fallback.
- Define a deterministic width budget. On narrow terminals, remove fields in
  this order: platform, Git detail, workspace middle segments, variant, then
  noncritical metrics. Model identity and a readable context indicator remain.
- Refresh Git state at a bounded cadence or explicit workspace/model change;
  never run a repository scan on every 50 ms draw.

## Non-Goals

- No Git mutation, repository discovery outside the selected workspace,
  network access, shell invocation, terminal title integration, user telemetry,
  configurable status-bar widgets, or new theme system.
- No exposure of absolute home directory prefixes, credentials, remote URLs,
  commit messages, untracked filenames, diff content, raw provider responses,
  tool arguments, or reasoning content.
- No change to model selection behavior; MODEL-007 owns the variant source and
  selection flow.

## Architecture Gate

Use the already approved Rust-native `gix` boundary if it can provide the
required read-only summary through an existing crate direction. Do not add a
`talos-tui -> talos-tools` dependency merely to read Git state. If a shared
cross-crate status payload needs additive fields, document the compatibility
impact and keep unknown/missing values optional.

## Acceptance For Behavior

- Given an active configured model, when the TUI is wider than the expanded
  threshold, then the status bar shows model, provider or variant where
  applicable, workspace display path, Git branch/dirty summary when available,
  platform label, and context `used% / limit`.
- Given a clean Git workspace, when rendered, then the branch is shown without
  file paths or remote URL; given a dirty workspace, then only a bounded count
  or marker is shown.
- Given a detached HEAD, non-Git directory, unavailable Git metadata, or Git
  read error, when rendered, then the remaining status fields still render and
  no raw error or panic is exposed.
- Given a width below each documented threshold, when rendered, then fields are
  removed in the declared order and no line wraps, overlaps, or loses the model
  identity/context indicator.
- Given a platform build target, when rendered, then it shows only the stable
  platform family label and no device-identifying information.
- Given status refreshes over time, when the workspace does not change, then
  Git inspection respects the documented bounded cadence and does not run once
  per draw.
- Given a model variant is unavailable, when rendered, then the bar omits it;
  given MODEL-007 data, it displays the selected variant without leaking
  provider configuration.

## Validation

- Unit tests for clean/dirty/detached/non-Git/error Git projections, platform
  labels, CJK/Unicode width, every compaction tier, context formatting, and
  secret/path suppression.
- Integration test using a temporary Git repository with a bounded dirty state;
  no test may inspect the developer's real repository or home directory.
- Native terminal screenshot/walkthrough at wide and narrow widths, on the
  current platform; CI fixtures cover macOS/Linux/Windows path separators and
  platform labels without requiring three physical hosts.
- Locked workspace fmt/check/clippy/test, release preflight, governance
  validation, and `git diff --check`.

## State / Documentation Owners

- README interactive TUI section and status-bar reference image/text
- `docs/backlog/active/TUI-011-status-bar-exit-polish.md`
- `docs/backlog/active/TUI-017-context-usage-percentage.md`
- `docs/backlog/active/TUI-018-context-limit-million-format.md`
- iteration owner, Board, iteration index, and execution package when selected

## Required Reads

- `AGENTS.md`
- `docs/sop/REQUIREMENT-INTAKE.md`
- `docs/sop/NEW-FEATURE.md`
- `docs/backlog/active/TUI-011-status-bar-exit-polish.md`
- `docs/backlog/active/TUI-017-context-usage-percentage.md`
- `docs/backlog/active/TUI-018-context-limit-million-format.md`
- `docs/backlog/active/MODEL-007-hierarchical-model-variant-selection.md`
- `crates/talos-tui/src/scrollback_status.rs`
- `crates/talos-conversation/src/types.rs`
- `crates/talos-cli/src/session_setup.rs`
- `crates/talos-tools/Cargo.toml`

## Residual Destination

- Full repository health, ahead/behind, diff statistics, or remote metadata:
  separate read-only Git UX story.
- Variant definition and selection: MODEL-007.
