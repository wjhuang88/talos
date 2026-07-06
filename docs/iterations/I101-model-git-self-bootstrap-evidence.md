# Iteration I101: Model, Git, And Self-Bootstrap Evidence Closeout

> Document status: Active
> Published plan date: 2026-07-06
> Planned objective: close user-facing model catalog residuals, continue Git fallback tracking, and
> produce honest self-bootstrap qualification evidence.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: model-browser walkthrough evidence, corrected connect/setup behavior for
> standard versus custom providers, incremental large-catalog rendering, updated Git capability
> matrix, and REL-002 evidence packet that does not overclaim Talos-primary status.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `MODEL-006` | Model catalog UX | In Progress | MC-001, MODEL-005 | Real terminal browser walkthrough and docs closeout. |
| `GIT-001` | Embedded Git tools | P0-P2 complete; tracking active | ADR-010 | Updated capability/fallback matrix after current `gix` review. |
| `REL-002` | v1.0 self-bootstrap gate | Planned No-go | RUNTIME-001, GOV-003 | Honest self-bootstrap evidence classification. |

### Scope

- Perform and record a real terminal walkthrough for `--available-models-browser`.
- Confirm `--available-models` remains bounded/filterable and prints `provider/model`.
- Confirm `/model` only exposes configured/selectable models and `/connect` owns provider setup.
- Ensure standard catalog providers use their catalog-defined API/base endpoint and do not ask the
  user to enter a URL during connect/setup.
- Ensure only custom provider creation requires the user to enter a URL.
- Change large model-list browser rendering to incremental or viewport-windowed rendering so the
  full packaged catalog is not rendered all at once.
- Re-check `gix` capability against current lockfile/version and update GIT-001.
- Produce a final evidence packet classifying what Talos did itself versus what remains Codex- or
  host-agent-primary.

### Non-Goals

- No return to runtime `catalog.db` creation.
- No provider credential disclosure or real provider network request unless separately approved.
- No Git push/pull/checkout replacement unless `gix` capability is proven and scoped.
- No `v1.0.0` claim, release tag, crate publish, or GitHub Release.

### Acceptance

- Given the packaged model catalog is large,
  When a user opens the independent browser,
  Then they can navigate/search/select/setup without dumping all rows to stdout.
- Given a model row lacks credentials,
  When selected,
  Then it routes to setup instead of becoming an active unconfigured model.
- Given the user connects a standard packaged provider,
  When the provider has a catalog-defined API endpoint,
  Then Talos asks for credentials only and does not ask the user to enter a URL.
- Given the user creates or connects a custom provider,
  When no catalog endpoint exists,
  Then Talos requires a URL and validates/persists it through the existing config merge path.
- Given the packaged catalog contains thousands of rows,
  When the browser opens or scrolls,
  Then rendering is incremental or viewport-windowed and does not build/render every row at once.
- Given Git fallbacks remain,
  When GIT-001 is updated,
  Then each fallback has current keep/replace/defer rationale.
- Given final self-bootstrap evidence is recorded,
  When REL-002 is reviewed,
  Then the report explicitly says whether the session qualifies or remains No-go.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo test -p talos-cli models`
- `cargo test -p talos-cli connect`
- `cargo test -p talos-tools git`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `scripts/validate_project_governance.sh .`
- `git diff --check`

### Documentation To Update

- `docs/backlog/active/MODEL-006-interactive-model-catalog-browser.md`
- `docs/backlog/active/MC-001-model-catalog-modernization.md`
- `docs/backlog/active/GIT-001-embedded-git-tools.md`
- `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md`
- `docs/reference/REL-002-READINESS-REPORT-2026-07-04.md` or a successor evidence packet
- `docs/BOARD.md`

### Risks And Rollback

- Risk: terminal walkthrough evidence is unavailable in automation.
- Rollback: keep MODEL-006 in refinement and record the exact terminal blocker.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-06 | Planning | Created as Month 4 of the 2026-07-06 autonomy/permission/runtime hardening plan. Not active until I100 closes or is explicitly paused. |
| 2026-07-06 | Activation | Activated after I100 completed and was pushed. This phase selects MODEL-006 residuals, GIT-001 tracking, and REL-002 evidence classification. Standard catalog providers must not ask for URL during connect/setup; custom providers still require URL. Large model list rendering must be viewport-windowed or incremental. No runtime `catalog.db`, provider network request, release action, or Git fallback replacement is authorized without separate proof. |

## Verification Evidence

- `cargo test -p talos-cli models_browser`: passed, 8 tests.
- `cargo test -p talos-cli connect`: passed, 6 tests.
- `cargo test -p talos-tui connect_mode`: passed, 6 tests.

## Variance And Residuals

- C12/C13 are closed before C11 walkthrough evidence. This is intentional: deterministic setup and
  viewport rendering behavior can be validated independently, while real-terminal walkthrough
  evidence remains open under C11.

## Retrospective

- Pending.
