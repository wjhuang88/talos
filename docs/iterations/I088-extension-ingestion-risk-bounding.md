# Iteration I088: Extension And Ingestion Risk Bounding

> Document status: Planned
> Published plan date: 2026-07-03
> Planned objective: Execute weeks 9-12 of the 2026-07-03 four-month hardening plan: improve local
> plugin/hook diagnostics and bounded document/HTML ingestion without expanding permission or browser
> boundaries.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: extension and ingestion surfaces are more useful while remaining local, explicit,
> read-only where applicable, and test-backed.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| H120 | PLUGIN-001/CMD-002/HOOK-001 | Mixed | ADR-027..030 | Local plugin diagnostics and hook listing |
| H121 | DIST-001 | Research | H120 | Optional asset distribution policy |
| H122 | WEBFETCH-001/TOOL-014 | Planned/Complete | TOOL-013 | Bounded document/HTML extraction slices |
| H123 | Hardening plan | Planned | H120-H122 | Month-3 closeout for extension and ingestion risk |

### Scope

- Keep plugin work local, explicit, read-only, and provenance-carrying.
- Add hook diagnostics without executable hook carriers unless a later ADR expands the boundary.
- Define optional asset manifests, cache, checksum, offline/mirror, and failure behavior.
- Improve bounded document/HTML extraction while excluding PDF/Office/OCR/browser automation.

### Non-Goals

- No remote plugin install or marketplace.
- No write-capable plugin tools.
- No browser cookies/storage/profile access.
- No PDF/Office/OCR dependency.

### Acceptance

- Given configured local plugin packages, diagnostics show manifests, capabilities, provenance, and
  validation errors.
- Given hook diagnostics run, builtins and config-declared placeholders are distinguishable.
- Given document/HTML extraction runs, output is bounded, permission-gated, and clearly reports
  unsupported binary formats.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test -p talos-plugin`
- `cargo test -p talos-tools`
- `cargo test -p talos-conversation -p talos-tui` when slash commands change
- `cargo test --workspace` at closeout
- `cargo clippy --workspace -- -D warnings` at closeout
- `scripts/validate_project_governance.sh .`

### Documentation To Update

- `docs/backlog/active/PLUGIN-001-wasm-runtime-plugins.md`
- `docs/backlog/active/CMD-002-command-taxonomy-realignment.md`
- `docs/backlog/active/HOOK-001-config-introduced-hooks.md`
- `docs/backlog/active/DIST-001-optional-runtime-asset-distribution.md`
- `docs/backlog/active/WEBFETCH-001-web-and-document-fetch-tools.md`
- `docs/tasks/2026-07-03-four-month-product-hardening-plan.md`
- `docs/BOARD.md` after owner docs

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-03 | Planning | Created as the I088 shell for extension and ingestion risk-bounded work. |

## Verification Evidence

- Planned.

## Variance And Residuals

- Planned.

## Retrospective

- Pending.
