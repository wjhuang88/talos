# Iteration I082: Frontline Month 3 — Document Ingestion And Parser Footprint

> Document status: Planned
> Published plan date: 2026-07-02
> Planned objective: Execute weeks 9-12 of the 2026-07-02 frontline plan: bounded document capture,
> HTML/link extraction, follow-up link references, and tree-sitter parser feature gates.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: Talos can ingest common local/web text resources more usefully while the default
> binary/parser footprint is reduced or explicitly measured.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| F120 | WEBFETCH-001 | Phase 2+ Planned | TOOL-014 | Design update for bounded document capture |
| F121 | WEBFETCH-001 | Planned | F120 | `document_extract` for text/Markdown/HTML/JSON/CSV |
| F122 | WEBFETCH-001 | Planned | F120 | `fetch_url` HTML extraction and top links |
| F123 | WEBFETCH-001 | Planned | F122 | Link reference metadata without implicit content persistence |
| F124 | TOOL-008 | Planned | CODE-002 | Parser feature gates and graceful fallback |
| F125 | WEBFETCH-001/TOOL-008 | Planned | F121-F124 | Ingestion closeout and dependency rationale |
| F126 | Frontline plan | Planned | F120-F125 | Month-3 closeout |

### Scope

- Add deterministic extraction for common text-like local documents.
- Improve HTML fetch output with title/content/link summaries.
- Keep outputs bounded and model-facing context compact.
- Gate parser language sets behind Cargo features where feasible.

### Non-Goals

- No PDF/Office/OCR extraction.
- No browser automation or browser profile access.
- No whole-site crawling.
- No runtime parser download.
- No silent persistence of fetched content.

### Acceptance

- Given a supported local text-like file, when `document_extract` runs, then Talos returns bounded
  content, detected kind, metadata, and unsupported-format diagnostics where applicable.
- Given an HTML URL, when `fetch_url` runs, then Talos returns extracted readable content plus
  ranked top links and total hidden-link count.
- Given a parser is unavailable in the default feature set, when highlighting/symbol analysis runs,
  then Talos degrades gracefully instead of panicking.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test -p talos-tools`
- `cargo test -p talos-tui` when highlighting changes
- `cargo test -p talos-tools --all-features` if parser feature gates are added
- `cargo build --release -p talos-cli` size evidence for F124 if feasible
- `cargo test --workspace` at closeout
- `cargo clippy --workspace -- -D warnings` at closeout
- `scripts/validate_project_governance.sh .`

### Documentation To Update

- `docs/backlog/active/WEBFETCH-001-web-and-document-fetch-tools.md`
- `docs/backlog/active/TOOL-008-tree-sitter-on-demand.md`
- `docs/decisions/020-tree-sitter-code-analysis.md` only if parser loading decision changes
- README/site tool docs if user-facing tools change
- `docs/BOARD.md` after owner docs

### Risks And Rollback

- Risk: new extraction dependency violates Rust-first or size constraints. Rollback: use an
  internal minimal extractor for the first slice and record heavier formats as residual.
- Risk: parser feature gates break symbol tools. Rollback: keep all parsers enabled and close F124
  as measurement plus design only.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-02 | Planning | Created as Month 3 shell for the frontline development plan. |

## Verification Evidence

- Planned.

## Variance And Residuals

- Planned.

## Retrospective

- Pending.
