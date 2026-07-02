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

## Verification Evidence

- Planned.

## Variance And Residuals

- Planned.

## Retrospective

- Pending.
