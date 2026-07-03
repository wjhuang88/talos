# TOOL-011: Ripgrep-Backed Grep Engine

| Field | Value |
|---|---|
| Type | Story |
| Priority | P2 |
| Status | Complete via I090 |
| Depends On | `TOOL-004`; ADR-025; current `GrepTool` tests |
| Owner Boundary | H2 architect-owned tool-family work |

## Outcome

Talos `grep` uses ripgrep's library components for ignore-aware, bounded, streaming-friendly
workspace search while preserving the existing read-only, permission-aware tool contract.

## Current Selection

I090 activated 2026-07-04 to audit the existing `crates/talos-tools/src/search_engine.rs` and
`search_tools.rs` implementation against this acceptance list before adding code. If the existing
ripgrep-backed path already satisfies the safe slice, I090 should close this story as delivered or
record only precise residual gaps.

I090 A5 found real stabilization gaps and closed them on 2026-07-04. `grep` now reports bounded
search statistics, skips binary and oversized files with compact summary counts, enforces file,
input-byte, output-byte, and elapsed-time budgets, rejects workspace escapes before walking, and
keeps symlink traversal disabled by default. The runtime path remains self-contained and does not
invoke host `rg`.

## Scope

Replace the current `regex + walkdir + read_to_string` grep internals with a Rust-native search
engine based on:

- `grep-searcher`
- `grep-regex`
- `grep-matcher`
- `ignore`

Do not depend on the top-level `ripgrep` CLI crate. Do not invoke host `rg` on the runtime path.

## Acceptance Criteria

- [x] Existing `GrepInput` fields remain backward compatible: `pattern`, `path`, `include`,
      `max_results`.
- [x] `grep` respects `.gitignore` and `.ignore` by default.
- [x] Search works outside Git repositories and without host `rg`.
- [x] Workspace path escape is rejected before walking.
- [x] Symlink behavior is explicit and tested; first slice does not follow symlinks by default.
- [x] Binary and oversized files are skipped by default and reported in a compact summary.
- [x] Invalid UTF-8 / mixed-encoding files do not fail the whole search.
- [x] Match count, file count, input byte count, output byte count, and elapsed time are bounded.
- [x] Cancellation or timeout produces a controlled tool error or truncated result, not a process
      hang.
- [x] Dependency errors and panics are contained at the integration boundary and returned as tool
      errors.
- [x] Output remains grouped, compact, line-oriented text compatible with current model prompts.
- [x] Deterministic tests cover ignore rules, include glob, binary skip, oversized skip, invalid
      encoding, truncation, path escape, and no-match behavior.
- [x] A small benchmark/smoke harness records old-engine vs ripgrep-backed behavior on a fixed
      fixture and a Talos-repo query.
- [x] `cargo test -p talos-tools grep_tool_tests`, `cargo check --workspace`, and
      `cargo test --workspace` pass.

## Non-Goals

- No top-level `ripgrep` crate dependency.
- No host `rg` fallback.
- No PCRE2 support.
- No SIMD feature enablement.
- No persistent search index.
- No TUI rendering changes; `TUI-014` owns grep scrollback summary rendering.
- No broader tool-family redesign; `TOOL-007` owns holistic audit after this direction is fixed.

## Design Notes

- Use `ignore::WalkBuilder` or equivalent ignore-aware walker as the file-discovery path.
- Keep search read-only under the existing permission policy.
- Preserve compact text output until `TOOL-007` decides whether result handles or structured
  search output are needed.
- Treat host `rg` only as a local benchmark/reference tool during development.
- Keep optional `grep-searcher` SIMD acceleration and PCRE2 disabled unless a later ADR approves
  the expanded dependency/review surface.

## Required Reads

- `docs/decisions/025-ripgrep-library-search-engine.md`
- `docs/backlog/active/TOOL-004-ripgrep-engine-evaluation.md`
- `docs/backlog/active/TOOL-001-portable-file-search.md`
- `docs/backlog/active/TOOL-003-posix-tool-set.md`
- `docs/backlog/active/TOOL-007-tool-set-design-audit.md`
- `crates/talos-tools/src/search_tools.rs`
- `crates/talos-tools/Cargo.toml`

## Residual Work Destination

After this lands, `TOOL-007` should audit the stabilized search tool as part of the full built-in
tool family and decide whether search result handles, progressive tool loading, or output
compression should change the user-facing contract.
