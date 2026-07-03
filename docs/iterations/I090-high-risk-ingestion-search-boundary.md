# Iteration I090: High-Risk Ingestion And Search Boundary

> Document status: Active
> Published plan date: 2026-07-04
> Planned objective: define and implement the first safe, bounded ingestion/search slice without
> weakening permission, dependency, or prompt-cache boundaries.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a runnable/testable ingestion or search boundary slice that proves bounded
> behavior and explicit unsupported-format handling.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `WEBFETCH-001` Phase 2+ | Web/document fetch tools | Planned / design-ready | TOOL-012/013/014 complete | Bounded extraction design and first safe local extraction slice or explicit deferral. |
| `TOOL-011` decision | Ripgrep-backed grep engine | Planned | ADR-025 | Decide whether grep stabilization must land before broader ingestion; implement only if necessary. |

### Scope

- Decide and document supported first formats: text, HTML, JSON, CSV, Markdown-like resources.
- Keep PDF, Office, OCR, browser automation, crawler behavior, and anti-bot bypass out of scope.
- Preserve `fetch_url` vs `save_url` boundary: context ingestion must not silently write files.
- Use permission facets for network/write paths where touched.

### Non-Goals

- No heavy document-conversion dependency.
- No remote crawling, browser sessions, or credentialed fetches.
- No marketplace/plugin-based extractor loading.

### Acceptance

- Given a safe local or fetched supported document,
  When extraction runs,
  Then the model-facing output is bounded, classified, and deterministic.
- Given an unsupported binary/PDF/Office/image input,
  When extraction runs,
  Then Talos reports metadata and unsupported status without crashing or dumping bytes.
- Given a write/save path is requested,
  When the tool is evaluated,
  Then existing permission facets are applied and no silent persistence occurs.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- Runtime smoke for the selected extractor/search path.
- `scripts/validate_project_governance.sh .`

### Documentation To Update

- `docs/backlog/active/WEBFETCH-001-web-and-document-fetch-tools.md`
- `docs/backlog/active/TOOL-011-ripgrep-backed-grep-engine.md` if touched
- `docs/BOARD.md`
- README only if user-visible commands change

### Risks And Rollback

- Risk: accidental broadening into browser/PDF/Office/OCR scope.
- Rollback: keep phase design-only and reject runtime extractor until ADR/dependency gate exists.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-04 | Activation | Activated after I085 was explicitly paused with only MC107 real terminal `/connect` walkthrough remaining. Non-terminal inventory disposition: I085 Paused; I086-I089 remain planned product-hardening shells; I091-I093 remain planned direct-owner shells. Initial evidence scan shows `document_extract`, `fetch_url`/`save_url`, and ripgrep-backed `grep` implementation already exist, so A3 starts with acceptance audit before adding code. |

## Verification Evidence

- Activation governance pending.

## Variance And Residuals

- Pending.

## Retrospective

- Pending.
