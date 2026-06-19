# TOOL-004: Spike - Evaluate Ripgrep As The Built-In Grep Engine

| Field | Value |
|---|---|
| Type | Spike |
| Priority | P2 |
| Status | Research |
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

- [ ] Benchmarks and behavior matrix for all viable options.
- [ ] Recommended primary/fallback design with rejected options and reasons.
- [ ] Dependency/security review and ADR decision if new long-lived crates or host assumptions are
      recommended.
- [ ] One or more executable implementation Stories, or an explicit decision to keep the current
      engine.
- [ ] TOOL-001/TOOL-003, Product Backlog, roadmap, and user-facing behavior documentation impacts
      are identified.

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
