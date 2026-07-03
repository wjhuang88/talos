# Iteration I080: Frontline Month 1 — Config And Governance Visibility

> Document status: Complete
> Published plan date: 2026-07-02
> Planned objective: Execute weeks 1-4 of the 2026-07-02 frontline plan: config subcommands,
> config validation evidence, TUI `/config` readiness, TUI composer command-line navigation,
> read-only `/agile status`, and dashboard governance visibility.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a developer can inspect/edit core config through stable CLI commands, navigate
> the composer with common command-line shortcuts, and inspect project governance state through
> read-only command/dashboard surfaces.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| F100 | Frontline plan | Planned | Board/backlog inventory | Published plan and iteration shells |
| F101 | CONF-001 | Partial | Existing config API | `talos config list/get/set` subcommands |
| F102 | CONF-001 | Partial | F101 | Config validation and masking evidence |
| F103 | CONF-001 | Partial | F101/CMD-001 | TUI `/config` readiness decision or read-only slice |
| F104 | TUI-021 | Planned | TUI-009 | Composer `Ctrl+A` / `Ctrl+E` navigation |
| F105 | GOV-003 | Planned | CMD-001 | Read-only `/agile status` |
| F106 | GOV-003/WEB-001 | Planned/In Progress | F105/ADR-031 | Read-only dashboard governance route/page |
| F107 | Frontline plan | Planned | F100-F106 | Month-1 closeout |
| F108 | WEB-001 | Planned | ADR-031 | `[dashboard] loopback_only` config (default true; false restores token auth) |

### Scope

- Add or harden user-facing config commands without changing config persistence semantics.
- Keep secret masking and `${ENV_VAR}` substitution intact.
- Add read-only governance status surfaces.
- Keep dashboard changes loopback-only; token auth is opt-in via `[dashboard] loopback_only = false`.
- Add command-line style composer cursor shortcuts without changing popup or approval priority.

### Non-Goals

- No TUI config write UI unless F103 explicitly proves a narrow safe slice.
- No full readline/keybinding framework.
- No governance mutation, auto-repair, or enforcement.
- No dashboard write routes, approvals, or remote access.

### Acceptance

- Given a valid config key, when a user runs `talos config get <key>`, then the value is shown with
  secrets masked.
- Given an invalid config value, when a user sets it, then schema validation rejects it before save.
- Given a governed workspace, when a user runs `/agile status` or the equivalent command path, then
  Talos reports board/backlog/iteration/validation state without mutating files.
- Given normal composer mode, when a user presses `Ctrl+A` or `Ctrl+E`, then the cursor moves to
  the current line start or end while slash/approval panels keep priority.
- Given dashboard is enabled, when the governance endpoint is requested with a valid token, then it
  returns read-only status with secrets redacted.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test -p talos-config -p talos-cli`
- `cargo test -p talos-conversation -p talos-tui` when slash commands change
- `cargo test -p talos-tui` when composer shortcuts change
- `cargo test -p talos-dashboard` when dashboard routes change
- `cargo test --workspace` at closeout
- `cargo clippy --workspace -- -D warnings` at closeout
- `scripts/validate_project_governance.sh .`

### Documentation To Update

- `docs/backlog/active/CONF-001-config-editing.md`
- `docs/backlog/active/TUI-021-command-line-composer-navigation.md`
- `docs/backlog/active/GOV-003-builtin-project-governance.md`
- `docs/backlog/active/WEB-001-embedded-web-control-surface.md` if dashboard changes
- README/site command documentation if user-facing command names change
- `docs/BOARD.md` after owner docs

### Risks And Rollback

- Risk: config commands drift from the existing `--config-*` flags. Rollback: keep flags as the
  implementation path and expose subcommands as thin wrappers only.
- Risk: dashboard governance data leaks local secrets or absolute paths. Rollback: keep route out
  of the dashboard and close F105 as a documented deferral.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-02 | Planning | Created as Month 1 shell for the frontline development plan. |
| 2026-07-02 | Execution | F101 README doc inconsistency fixed. F102 added `Config::validate()` before save + 9 evidence tests. F104 added Ctrl+A/Ctrl+E line navigation + evolution re-keyed to Ctrl+G + 11 tests. F108 added `[dashboard] loopback_only` config (default `true`). |
| 2026-07-02 | Execution | F105 implemented `/agile status` slash command: new `governance_summary` module in `talos-conversation`, `workspace_root` field on `ConversationEngine`, command registry entry, and 6 tests (4 module + 2 engine). |
| 2026-07-02 | Closeout | F107 Month-1 closeout. All workspace gates pass. F103 deferred, F106 verified, F108 default flipped to `true` per maintainer direction. |

## Verification Evidence

- `cargo test --workspace` — all green.
- `cargo test -p talos-conversation` — 88/88 pass (includes `/agile` engine tests + governance_summary module tests).
- `cargo test -p talos-tui` — 199/199 pass (includes Ctrl+A/Ctrl+E cursor tests).
- `cargo test -p talos-config` — 96/96 pass (includes `loopback_only` default `true` tests).
- `cargo test -p talos-dashboard` — 19/19 pass (includes loopback_only no-token tests).
- `cargo test -p talos-cli` — config validation evidence tests pass.
- `cargo clippy --workspace -- -D warnings` — clean.
- `cargo fmt --all -- --check` — clean.
- `scripts/validate_project_governance.sh .` — 0 warnings.

## Variance And Residuals

- F101: README doc inconsistency was the only gap; subcommands were already shipped in I045.
- F102: Added `Config::validate()` to `run_config_set` before save (was missing at CLI path).
- F103: **Deferred with UX spec.** TUI `/config` write UI remains future work.
  - **Current coverage**: `talos config list/get/set` CLI subcommands (I045+F102) handle all
    editing needs; `/model` picker handles runtime model switching; `/status` shows model/provider
    at a glance.
  - **Gap**: No in-TUI config editing without dropping to a terminal. Users who live in the TUI
    full-screen must exit or open a second terminal to change config.
  - **Deferral rationale**: A TUI config editor requires careful UX design (inline form vs.
    command-driven), permission considerations (writing config from within the agent loop), and
    test infrastructure for multi-field form interactions. These exceed Month-1 scope.
  - **Reactivation trigger**: User demand for in-TUI config editing, or when the model picker
    proves insufficient for provider/credential management.
  - **Recommended future slice**: Read-only `/config` that dumps the masked config (same as
    `talos config list`) as a first step. Write-capable `/config set key value` as a second step.
- F104: Evolution panel toggle moved from `Ctrl+E` to `Ctrl+G`.
- F105: Implemented as a `governance_summary` module inside `talos-conversation` (not via `talos-cli`'s governance.rs) to avoid cross-crate coupling. Reads manifest, board disposition, open iterations, active backlog items, and validation harness status. Dashboard governance route (F106) enriched with manifest + iteration data.
- F106: **Enriched** — dashboard `/governance` route now includes manifest summary, board disposition, and open iteration IDs (previously board-only). Auth-gated or loopback-only per F108, and redacted at the response boundary.
- F108: `loopback_only` default changed from `false` to `true` per maintainer direction (second commit). ADR-031 amended.

## Retrospective

- Month-1 delivered in a single session. Config, governance visibility, input navigation, and dashboard auth model all landed with workspace-wide test/clippy/fmt/governance gates green.
- The `/agile` command was implemented as a self-contained `governance_summary` module in `talos-conversation` rather than routing through `talos-cli`'s `governance.rs` to avoid cross-crate dependency. The tradeoff is a small amount of parsing logic duplication (board section + open-iteration parsing); consolidating this into a shared module is a future cleanup task if a third consumer appears.
- F103 (TUI `/config`) was deferred rather than rushed. The CLI config commands are sufficient for editing; a TUI read-only slice can land in a later iteration if demand appears.
