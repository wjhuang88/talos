# TOOL-004: Spike - Evaluate Ripgrep As The Built-In Grep Engine

| Field | Value |
|---|---|
| Type | Spike |
| Priority | P2 |
| Status | Complete (research, 2026-06-28) |
| Timebox | 2-3 engineering days |
| Depends On | ADR-010 dependency boundary; current TOOL-003 grep implementation |

## Question

Should Talos replace its current `regex + walkdir + std::fs::read_to_string` grep implementation
with ripgrep technology while preserving a self-contained, permission-aware Rust tool?

## Confirmed Baseline

The current `GrepTool` is not based on ripgrep and does not invoke host `rg`. It uses `regex::Regex`,
`walkdir::WalkDir`, `glob::Pattern`, and whole-file UTF-8 reads in
`crates/talos-tools/src/search_tools.rs`.

## Options

1. Embed ripgrep's Rust crates such as `grep-searcher`, `grep-regex`, and `ignore`.
2. Invoke an installed `rg` executable as a structured fallback/bridge.
3. Keep the current implementation and selectively add missing semantics/performance safeguards.
4. Use an alternative Rust-native search library only if evidence clearly beats the ripgrep crates.

## Evidence To Gather

- Search throughput and peak memory on small, large, binary-heavy, and mixed-encoding workspaces.
- `.gitignore`, hidden-file, symlink, binary, encoding, and include-glob behavior.
- Streaming/cancellation support and enforcement of match, byte, time, and file-count budgets.
- Workspace-root/path-escape guarantees and behavior under concurrent file changes.
- Cross-platform behavior, MSRV, license, dependency count, binary-size/build-time impact, and
  maintenance cadence.
- Panic/native-code boundary risk under AGENTS Hard Constraint #9.
- API fit for grouped string output, line numbers, future context-only results, and deterministic
  tests.
- Whether external `rg` would violate the self-contained primary-path rule and what replacement
  trigger would be required if retained as fallback.

## Expected Output

- [x] Benchmarks and behavior matrix for all viable options.
- [x] Recommended primary/fallback design with rejected options and reasons.
- [x] Dependency/security review and ADR decision if new long-lived crates or host assumptions are
      recommended.
- [x] One or more executable implementation Stories, or an explicit decision to keep the current
      engine.
- [x] TOOL-001/TOOL-003, Product Backlog, roadmap, and user-facing behavior documentation impacts
      are identified.

## Research Result (2026-06-28)

Recommendation: replace the current `GrepTool` internals with ripgrep's library components in a
separate implementation story, while preserving the current user-facing `grep` tool contract.

Approved primary target:

- `grep-searcher`
- `grep-regex`
- `grep-matcher`
- `ignore`

Rejected as runtime primary paths:

- top-level `ripgrep` crate: registry/source inspection showed it packages the `rg` CLI binary
  (`ripgrep 15.1.0`, binary target `rg`) rather than a library target suitable for Talos embedding;
- host `rg`: useful as a behavior/performance reference, but not acceptable as Talos's primary
  path because `TOOL-001` requires search to work without host `rg`;
- current implementation indefinitely: too weak on ignore semantics, streaming/cancellation,
  mixed-encoding handling, and budget enforcement.

Decision recorded in ADR-025:

- `docs/decisions/025-ripgrep-library-search-engine.md`

Executable follow-up story:

- `docs/backlog/active/TOOL-011-ripgrep-backed-grep-engine.md`

`TOOL-007` is now unblocked for design work from a search-engine direction standpoint. If the
maintainer wants the actual grep implementation stabilized before the holistic audit, activate
`TOOL-011` first; otherwise `TOOL-007` can proceed using ADR-025 as its search direction.

## Evidence Matrix

| Option | Evidence | Decision |
|---|---|---|
| Current `regex + walkdir + read_to_string` engine | Source audit confirms whole-file UTF-8 reads, manual hidden-dir skip, include glob on file name only, match-count cap only, and no `.gitignore` handling. `cargo test -p talos-tools grep_tool_tests` passed 10 tests. | Keep only until `TOOL-011`; not the long-term engine. |
| Ripgrep library crates | `cargo info` on 2026-06-28 confirmed `grep-searcher 0.1.16`, `grep-regex 0.1.14`, `grep-matcher 0.1.8`, and `ignore 0.4.26`, all `Unlicense OR MIT` and sourced from ripgrep subcrates. | Preferred implementation target. |
| Top-level `ripgrep` crate | `cargo info ripgrep` reported `ripgrep 15.1.0`; local registry `Cargo.toml` inspection found binary target `rg` and no library target. | Reject as embedded API. |
| Host `rg` executable | `/opt/homebrew/bin/rg` exists locally. Reference timings: `RuntimeBuilder` query across `crates docs` in 0.026s wall time; broad `fn ` query across three crates in 0.012s wall time. | Reference/benchmark only, not runtime dependency. |
| Alternative Rust-native library | No candidate beat the ripgrep subcrates on semantics, maintenance lineage, and API fit during this Spike. | No separate alternative selected. |

## Behavior Matrix

| Behavior | Current Engine | Ripgrep Library Target | Host `rg` |
|---|---|---|---|
| Works without host utility | Yes | Yes | No |
| `.gitignore` / `.ignore` support | No | Yes via `ignore` | Yes |
| Binary file handling | Talos binary sniff before full read | Searcher-level and Talos-level skip required | Yes |
| Mixed encoding | Failed reads are skipped silently | Must degrade with explicit skipped-file accounting | Yes, reference only |
| Streaming/cancellation fit | Poor; collects files and reads full content | Better; implementation must wire budgets/cancellation | Process timeout only |
| Workspace boundary control | Talos resolver before walk | Talos resolver before `ignore` walk | Requires command wrapper discipline |
| Dependency surface | Existing `regex`, `walkdir`, `glob` | Adds ripgrep subcrates; no PCRE2/SIMD by default | Adds host assumption |
| Output contract | Grouped text | Preserve grouped text in first slice | CLI output would need parsing |

## Documentation Impact

- `TOOL-001`: direction remains compatible with native file/search tools that do not require host
  `rg`.
- `TOOL-003`: earlier rationale for direct `regex` remains historical for the first implementation;
  ADR-025 supersedes it for the next grep engine slice.
- Product Backlog and Board should mark `TOOL-004` complete and make `TOOL-007` the next H2 design
  item, with optional `TOOL-011` implementation before or during that audit.
- README changes are not required until the user-facing `grep` behavior changes in `TOOL-011`.

## Non-Goals

- Implementing or changing `GrepTool` during this Spike.
- Adding an unconditional host `rg` dependency.
- Expanding grep output into TUI history or changing hidden tool-output policy.
- Building a persistent search index.

## Required Reads

- `docs/backlog/active/TOOL-001-portable-file-search.md`
- `docs/backlog/active/TOOL-003-posix-tool-set.md`
- `docs/decisions/010-git-search-tool-dependency-boundary.md`
- `docs/proposals/builtin-workspace-search-tools.md`
- `crates/talos-tools/src/search_tools.rs`
- `crates/talos-tools/Cargo.toml`
