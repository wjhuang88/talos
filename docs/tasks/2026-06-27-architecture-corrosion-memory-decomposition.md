# 2026-06-27 Architecture Corrosion And Memory Decomposition Task

**Status**: Complete
**Owner story**: `ARCH-012`
**Iteration**: `I059`
**Requested outcome**: Create a separate long-running task for architecture optimization, judge
architecture corrosion, split oversized modules where the evidence is concrete, and close the loop
with validation and governance sync.

## Success Criteria

- [x] A product-site requirement exists for GitHub Pages/custom domain planning (`WEB-002`).
- [x] Architecture corrosion is judged with evidence, not line count alone.
- [x] One oversized module is split in a behavior-preserving way.
- [x] Targeted tests pass.
- [x] Workspace quality gates pass.
- [x] Governance docs are synchronized.
- [x] Residual work is recorded.

## Corrosion Assessment

`crates/talos-memory/src/lib.rs` was selected because it combined five independent concerns in one
2141-line file:

- public domain/error/status types;
- SQLite connection, schema migration, retrieval scoring, retention, and maintenance;
- deterministic entity extraction;
- memory prompt injection and hidden-output filtering;
- 1000+ lines of tests.

This is a concrete architecture-corrosion case because future work on memory storage, prompt safety,
or entity ranking would all collide in the same file. The chosen fix is a module split, not a
behavior redesign.

## Implementation Record

| Area | Before | After |
|---|---|---|
| Crate root | `lib.rs` 2141 lines with all concerns mixed | `lib.rs` 39 lines, docs + modules + re-exports |
| Types | In root | `types.rs` |
| Store/schema/retrieval | In root | `store.rs` |
| Entity extraction | In root | `entities.rs` |
| Prompt formatting/filtering | In root | `prompt.rs` |
| Tests | In root | `tests.rs` |

Current `talos-memory/src` line counts after split:

| File | Lines |
|---|---:|
| `consolidation.rs` | 547 |
| `entities.rs` | 138 |
| `lib.rs` | 39 |
| `prompt.rs` | 156 |
| `store.rs` | 630 |
| `tests.rs` | 1053 |
| `types.rs` | 142 |

## Validation Evidence

- 2026-06-27: `cargo test -p talos-memory` passed: 48 tests, 0 failures.
- 2026-06-27: `cargo fmt --all -- --check` passed.
- 2026-06-27: `cargo check --workspace` passed.
- 2026-06-27: `cargo clippy --workspace -- -D warnings` passed.
- 2026-06-27: `cargo test --workspace` passed on rerun. The first full run had a transient
  `mcp_client_e2e` evidence assertion; targeted rerun and second full workspace run both passed.
- 2026-06-27: `scripts/validate_project_governance.sh .` passed with 0 warnings.
- 2026-06-27: `git diff --check` passed.

## Residuals

- `crates/talos-config/src/lib.rs` and `crates/talos-cli/src/mode_runners.rs` are the next concrete
  oversized production candidates, but they need separate owner stories.
- `crates/talos-tui/src/scrollback.rs` remains under ARCH-011 watchlist promotion rules.
- `crates/talos-agent/src/tests.rs` is large but test-only; do not promote without maintenance
  friction evidence.

## Resume Instructions

This task is closed. Future architecture work should start a new owner story, most likely for
`crates/talos-config/src/lib.rs` or `crates/talos-cli/src/mode_runners.rs`, and should repeat the
same audit-before-split gate instead of continuing under ARCH-012.
