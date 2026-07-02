# Iteration I080: Frontline Month 1 — Config And Governance Visibility

> Document status: Planned
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
| F108 | WEB-001 | Planned | ADR-031 | Opt-in `[dashboard] loopback_only` config (default false keeps token auth) |

### Scope

- Add or harden user-facing config commands without changing config persistence semantics.
- Keep secret masking and `${ENV_VAR}` substitution intact.
- Add read-only governance status surfaces.
- Keep dashboard changes loopback-only and token-gated.
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
| 2026-07-02 | Execution | F101 config-subcommand README doc inconsistency fixed (two positional args form, not KEY=VALUE). F102 added `Config::validate()` call in `run_config_set` before save and added 9 CLI-level evidence tests for schema rejection, env var name round-trip, secret masking round-trip, and TOML serialize/parse round-trip. F104 added TUI composer `Ctrl+A` (line start) and `Ctrl+E` (line end) cursor navigation with 11 unit tests; evolution panel toggle re-keyed from `Ctrl+E` to `Ctrl+G` to free `Ctrl+E`; hint text and evolution panel title updated. F108 added opt-in `[dashboard] loopback_only` config (default `false` keeps token auth); `DashboardServer::with_loopback_only()` skips auth middleware when set; 5 new dashboard tests verify no-token access and redaction still applies. |

## Verification Evidence

- `cargo test --workspace` — all green (no failures across all crates).
- `cargo test -p talos-tui` — 199/199 pass.
- `cargo test -p talos-config` — dashboard loopback_only tests pass (4 dashboard tests, 96 total).
- `cargo test -p talos-dashboard` — 19/19 pass (5 new loopback_only tests added).
- `cargo test -p talos-cli` — config + wizard + governance tests pass (88 filtered, 0 failed).
- `cargo clippy --workspace -- -D warnings` — clean.
- `cargo fmt --all -- --check` — clean.
- `scripts/validate_project_governance.sh .` — 0 warnings.

## Variance And Residuals

- F101: README doc inconsistency was the only implementation gap; the subcommand form and flag form were both already shipped in I045.
- F102: Added `Config::validate()` call to `run_config_set` to satisfy "schema validation rejects invalid values before save" (was previously missing at the CLI path; `Config::load()` already validated on read).
- F104: Evolution panel toggle moved from `Ctrl+E` to `Ctrl+G` to make room for the standard readline line-end shortcut. The bind change is reflected in the composer hint text and the evolution panel title.
- F105: Not yet implemented. `/agile` slash command still missing from the command registry; the reusable governance parsing logic exists in `talos-cli/src/governance.rs` and is duplicated in `mode_runners.rs`.
- F106: Verification only — the dashboard `/governance` route already exists, is auth-gated, and applies `redact_snapshot` at the response boundary.
- F108: New opt-in config `[dashboard] loopback_only = true` skips the per-process bearer token. Default `false` keeps token auth as the safe baseline.
- F103: Deferred. TUI `/config` write UI remains a future slice. The CLI config commands cover the editing need, and the `/model` picker covers runtime model switching. A narrow read-only TUI `/config` slice could be added in a later iteration if demand appears.
- Month-1 closeout (F107) will be recorded after F105 lands or is formally deferred.

## Verification Evidence

- Planned.

## Variance And Residuals

- Planned.

## Retrospective

- Pending.
