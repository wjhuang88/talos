# Iteration I081: Frontline Month 2 — Extension And Distribution Discipline

> Document status: Planned
> Published plan date: 2026-07-02
> Planned objective: Execute weeks 5-8 of the 2026-07-02 frontline plan: plugin diagnostics,
> hook listing/diagnostics, read-only plugin fixture polish, explicit local plugin enablement UX,
> and optional asset distribution design.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: local plugin packages are visible and diagnosable without expanding Talos's
> permission, remote-install, or write-capable plugin boundaries.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| F110 | PLUGIN-001/CMD-002 | In Progress/Planned | T111 baseline | Plugin package diagnostics and listing |
| F111 | HOOK-001/CMD-002 | Planned | CMD-001 | `/hooks` builtin/config diagnostic surface |
| F112 | PLUGIN-001 | In Progress | F110 | Read-only plugin fixture polish |
| F113 | PLUGIN-001 | In Progress | F110/F112 | Explicit local package opt-in UX |
| F114 | DIST-001 | Research | PLUGIN-001 package format | Asset distribution proposal and ADR draft |
| F115 | PLUGIN-001/DIST-001 | In Progress/Research | F110-F114 | Security closeout |
| F116 | Frontline plan | Planned | F110-F115 | Month-2 closeout |

### Scope

- Make plugin packages inspectable through `/plugins` and/or CLI diagnostics.
- Keep local package loading explicit and disabled by default.
- Add hook listing/diagnostic UX without adding executable hook carriers.
- Produce asset distribution design without implementing online downloads.

### Non-Goals

- No remote plugin install.
- No plugin marketplace.
- No write-capable plugin tools.
- No Lua/dylib/Python/Node carrier.
- No automatic asset download.

### Acceptance

- Given a configured local plugin package, when diagnostics run, then Talos shows manifest validity,
  declared capabilities, provenance, and validation errors.
- Given hooks are registered, when `/hooks` runs, then builtins and config-declared placeholders are
  distinguishable.
- Given optional asset distribution is proposed, then the proposal covers manifest, verification,
  cache, mirror/offline, uninstall, and failure behavior.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test -p talos-plugin`
- `cargo test -p talos-tools` when plugin tool registration changes
- `cargo test -p talos-conversation -p talos-tui` when slash commands change
- `cargo test --workspace` at closeout
- `cargo clippy --workspace -- -D warnings` at closeout
- `scripts/validate_project_governance.sh .`

### Documentation To Update

- `docs/backlog/active/PLUGIN-001-wasm-runtime-plugins.md`
- `docs/backlog/active/CMD-002-command-taxonomy-realignment.md`
- `docs/backlog/active/HOOK-001-config-introduced-hooks.md`
- `docs/backlog/active/DIST-001-optional-runtime-asset-distribution.md`
- New proposal/ADR draft for optional asset distribution
- README/site only if user-facing commands become shipped
- `docs/BOARD.md` after owner docs

### Risks And Rollback

- Risk: plugin listing implies install support. Rollback: word UI as local explicit packages only
  and keep remote install unavailable.
- Risk: hook diagnostics are mistaken for executable hooks. Rollback: list only builtin hooks and
  close config-introduced hooks as schema-only residual.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-02 | Planning | Created as Month 2 shell for the frontline development plan. |

## Verification Evidence

- Planned.

## Variance And Residuals

- Planned.

## Retrospective

- Pending.
