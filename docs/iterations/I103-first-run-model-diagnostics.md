# Iteration I103: First-Run Model And Diagnostics

> Document status: Planned
> Published plan date: 2026-07-07
> Planned objective: Execute Month 2 of the 2026-07-07 four-month developer operating plan by
> making first-run provider setup, model selection, and diagnostics usable for controlled trials.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a new developer can configure a standard provider, browse/select a model, and
> produce a redacted diagnostic report without editing source files.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| D110 | MODEL-006/MC-001 | Planned | I102 closeout or explicit activation | Standard providers skip base-url prompts; custom providers require a URL. |
| D111 | MODEL-006 | Planned | D110 | Large model inventories are bounded and searchable in CLI/TUI surfaces. |
| D112 | VALIDATION-001/CONF-001 | Planned | D110 | Redacted diagnostics report config, provider, credential source, data dirs, and validation adapters. |
| D113 | Developer operating plan | Planned | D110-D112 | First-run docs and setup evidence are synchronized. |

### Scope

- Verify and polish `/connect` standard-provider and custom-provider flows.
- Keep model browsing responsive for large catalogs without reintroducing runtime `catalog.db`.
- Add or refine a redacted diagnostic path using existing config and validation services.
- Update first-run docs only after behavior is verified.

### Non-Goals

- No runtime catalog database.
- No provider credential schema change.
- No OAuth/device-flow implementation.
- No site deployment or release action.

### Acceptance

- Given a built-in standard provider, when a user runs `/connect`, then Talos asks only for the
  needed credential source and does not ask for a base URL.
- Given a custom provider, when a user runs `/connect`, then Talos requires an explicit base URL and
  preserves secret masking.
- Given a large model list, when a user browses or filters it, then output remains bounded and
  provider-qualified.
- Given a diagnostic command is run, then secrets are masked and local paths/config status are clear.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo test -p talos-cli connect`
- `cargo test -p talos-cli model`
- `cargo test -p talos-config provider`
- `cargo check --workspace`
- Manual or integration evidence for a large model list and redacted diagnostics
- `cargo test --workspace` at closeout
- `scripts/validate_project_governance.sh .`

### Documentation To Update

- `README.md`
- `README.zh-CN.md` if user-facing setup text changes
- `docs/backlog/active/MODEL-006-interactive-model-catalog-browser.md`
- `docs/backlog/active/VALIDATION-001-internal-validation-service.md`
- `docs/BOARD.md` after owner docs

### Risks And Rollback

- Risk: setup docs drift from actual `/connect` behavior.
- Rollback: keep docs changes behind verified command evidence and leave uncertain behavior as a
  known limit instead of documenting it as supported.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-07 | Planning | Created as Month 2 shell for the four-month developer operating plan. |

## Verification Evidence

- Planned.

## Variance And Residuals

- Planned.

## Retrospective

- Pending.
