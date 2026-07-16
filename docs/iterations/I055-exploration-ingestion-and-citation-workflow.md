# I055: Exploration Ingestion And Citation Workflow

**Status**: Complete (2026-06-29)
**Created**: 2026-06-26
**Depends On**: I054 exploration storage foundation

## Objective

Make the exploration library usable from Talos workflows by adding permission-aware ingestion and a
citation-preserving synthesis path.

## Published Baseline

### Selected Stories

- RES-001 ingestion and citation workflow.
- WEBFETCH-001 compatibility boundary where existing fetch tools are reused.

### MVP Deliverable

A user can ingest local or fetched text into an exploration run, extract claims, and produce a
synthesis that cites stored source IDs.

### Scope

- Add local-file ingestion path first.
- Reuse existing permission-aware web/fetch tools only through approved boundaries.
- Add claim extraction interface with deterministic test fixtures.
- Add synthesis/citation assembly with evidence/inference distinction.
- Keep ingestion disabled or dry-run where network permission is absent.

### Non-Goals

- No crawler.
- No paid API dependency.
- No document conversion stack beyond existing supported inputs.
- No vector/graph dependency.

### Acceptance

- Given local source text, ingestion creates source chunks and searchable claims.
- Given fetched content through existing permission flow, ingestion records source provenance.
- Given a synthesis, output distinguishes cited evidence from inference.
- Given disabled network tools, exploration still works with local sources.
- Given missing citations, synthesis validation fails.

### Validation Plan

- Local ingestion tests.
- Mock fetch ingestion tests.
- Claim/synthesis citation tests.
- Permission-disabled tests.
- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `scripts/validate_project_governance.sh .`

### Documentation To Update

- `README.md`
- `README.zh-CN.md`
- `docs/backlog/active/RES-001-exploration-library.md`
- `docs/backlog/active/WEBFETCH-001-web-and-document-fetch-tools.md`

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-06-26 | **Activation** | I055 activated. Dependencies met: I054 in Review (exploration storage foundation, commit `7e15706`). Scope: local text ingestion (chunking + FTS), mock fetch ingestion with provenance, deterministic claim extraction, citation-preserving synthesis assembly with evidence/inference distinction. No crawler, no paid APIs, no network in tests. |
| 2026-06-26 | **Implementation** | All acceptance criteria delivered. `ingest_text()` with paragraph-based chunking + overlap + SHA-256 hash. `ingest_fetched()` with URL/timestamp provenance. `extract_claims()` deterministic sentence-based extraction. `create_synthesis()` with citation validation. CLI `talos explore ingest/search`. 8 tests including full offline pipeline. Runtime verified: ingested README.md (92 chunks), FTS search returns snippets. |

## Verification Evidence

### Workspace Gates (2026-06-26)

- `cargo fmt --all -- --check` — clean
- `cargo check --workspace` — clean
- `cargo clippy --workspace -- -D warnings` — clean
- `cargo test --workspace` — all pass, 0 failures
- `scripts/validate_project_governance.sh .` — 0 warnings

### End-to-End Runtime Evidence (ITERATION-WORKFLOW §3a)

- `talos explore ingest --file README.md`: 92 chunks created, source + run persisted.
- `talos explore search --query "session" --limit 3`: 3 results with snippets from ingested content.

### Changed Files

| File | Change |
|---|---|
| `crates/talos-exploration/src/ingestion.rs` | NEW: ChunkingConfig, IngestionReport, FetchedContent, ingest_text/fetched, extract_claims, create_synthesis, 8 tests |
| `crates/talos-exploration/src/lib.rs` | Added `pub mod ingestion;` |
| `crates/talos-cli/src/exploration_cli.rs` | NEW: ExploreCommand::Ingest/Search CLI handlers |
| `crates/talos-cli/src/main.rs` | Added `mod exploration_cli`, `Explore` variant, dispatch |
| `crates/talos-cli/Cargo.toml` | Added `talos-exploration` dependency |

### I057 Acceptance Remediation (2026-06-26)

I057-S3a fixed a UTF-8 panic: search snippet truncation at `lib.rs:472` used `&text[..197]`
(byte slice) which panics when byte 197 falls inside a multibyte character. Replaced with
`text.chars().take(197).collect::<String>()` (character-safe). Regression test covers Chinese
(3-byte) and emoji (4-byte) text through FTS search.

I057-S3b added resource budget: `ChunkingConfig` now has `max_file_bytes` (default 10 MB) and
`max_chunks_per_source` (default 10,000). `ingest_text()` rejects oversized input with
`FileTooLarge` error. CLI checks file metadata before `read_to_string`. 3 regression tests
cover budget exceeded, within budget, and chunk cap exceeded.

Changed files: `crates/talos-exploration/src/lib.rs`, `crates/talos-exploration/src/ingestion.rs`,
`crates/talos-cli/src/exploration_cli.rs`.
