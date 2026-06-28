# 025: Ripgrep Library Search Engine

## Status

Accepted

## Context

`TOOL-004` evaluated whether Talos should replace the current `grep` implementation, which uses
`regex::Regex`, `walkdir::WalkDir`, `glob::Pattern`, and `std::fs::read_to_string`, with ripgrep
technology while preserving a self-contained, permission-aware Rust tool.

The current implementation is simple and already passes its unit tests, but it has known limits:

- it does not respect `.gitignore` / `.ignore`;
- it collects the complete file list before searching;
- it reads each candidate file as full UTF-8 text;
- it has a match-count cap but no explicit file, byte, elapsed-time, or total-output budget;
- it skips hidden directories by Talos helper rules rather than using standard ignore semantics;
- it has no streaming/cancellation integration for long searches.

The `ripgrep` crate exists, but registry inspection on 2026-06-28 showed it is the CLI package:
`ripgrep 15.1.0`, binary target `rg`, Rust version `1.85`, license `Unlicense OR MIT`, and no
library target suitable as Talos's direct embedded API. Ripgrep's reusable implementation surface
is instead split across library crates such as `grep-searcher`, `grep-regex`, `grep-matcher`, and
`ignore`.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
| --- | --- | --- | --- |
| Talos search must work without host `find`, `grep`, `rg`, or shell features. | Hard | `TOOL-001` acceptance | No |
| Write-capable tools must go through the permission pipeline; grep remains read-only. | Hard | AGENTS.md Hard Constraint #4 | No |
| No arbitrary native dependencies without ADR. | Hard | AGENTS.md Hard Constraint #1 | No |
| External dependencies that may panic or involve native/process boundaries must degrade safely. | Hard | AGENTS.md Hard Constraint #9 | No |
| Search should be faster and more complete than the current whole-file UTF-8 implementation. | Soft | `TOOL-004` / product quality | Yes |
| Host `rg` is a useful behavior/performance reference. | Soft | Local developer environment | Yes |
| Ripgrep library crates can provide the right semantics without importing the CLI package. | Assumption | Registry/source inspection | Validate during implementation |

## Reasoning

Calling host `rg` would be the fastest way to inherit ripgrep behavior, but it violates Talos's
self-contained primary-path rule. It can also drift by host version and installation, and it would
force Talos to parse CLI output while preserving permission, timeout, and workspace-boundary
guarantees. Host `rg` is therefore useful only as a benchmark and behavior reference.

Depending on the top-level `ripgrep` crate is also the wrong abstraction. It packages the `rg`
binary, not a stable library API for embedding.

The ripgrep library crates are a better fit. `grep-searcher` provides the fast line-oriented search
engine, `grep-regex` and `grep-matcher` connect regex matching to that engine, and `ignore`
provides standard `.gitignore` / `.ignore` walking semantics. Registry metadata gathered on
2026-06-28:

| Crate | Version | License | Role |
| --- | --- | --- | --- |
| `grep-searcher` | `0.1.16` | `Unlicense OR MIT` | line-oriented search engine |
| `grep-regex` | `0.1.14` | `Unlicense OR MIT` | Rust regex matcher integration |
| `grep-matcher` | `0.1.8` | `Unlicense OR MIT` | matcher trait |
| `ignore` | `0.4.26` | `Unlicense OR MIT` | ignore-aware directory walk |
| `grep-cli` | `0.1.12` | `Unlicense OR MIT` | optional CLI-style utility helpers; not required by default |

Local reference evidence from the Talos repository on 2026-06-28:

- `cargo test -p talos-tools grep_tool_tests`: 10 grep tests passed.
- `rg --files crates docs | wc -l`: 496 files in the measured scope.
- `time rg -n "RuntimeBuilder" crates docs --glob '*.rs' --glob '*.md'`: 0.026s wall time.
- `time rg -n "fn " crates/talos-tools crates/talos-agent crates/talos-cli --glob '*.rs'`:
  0.012s wall time, with output truncated by the terminal harness.

These timings are reference evidence for ripgrep's expected behavior, not Talos runtime
benchmarks. The implementation story must add a deterministic benchmark/smoke harness before the
dependency swap is considered complete.

## Decision

1. **Use ripgrep library crates as the preferred implementation target for Talos `grep`.**
   - Target `grep-searcher`, `grep-regex`, `grep-matcher`, and `ignore`.
   - Do not depend on the top-level `ripgrep` crate for embedded search.
   - Do not invoke host `rg` on the primary path.

2. **Keep host `rg` as a reference only.**
   - It may be used in research, benchmarks, and behavior comparison.
   - It must not become a runtime fallback unless a separate ADR records the host dependency,
     command shape, timeout, unavailable-host behavior, and replacement trigger.

3. **Disable heavier optional behavior by default.**
   - Do not enable PCRE2.
   - Do not enable SIMD-specific features until binary-size, portability, and safety evidence
     justifies them.

4. **Preserve the user-facing `grep` tool contract in the first implementation slice.**
   - Tool name remains `grep`.
   - Existing input fields remain valid.
   - Output remains compact, grouped, line-oriented text unless `TOOL-007` later changes the
     tool-result design.

5. **Require safe integration boundaries.**
   - Enforce workspace-root bounds before walking.
   - Use `ignore` semantics by default.
   - Skip binary and oversized files by default.
   - Enforce match, file, byte, output, and elapsed-time budgets.
   - Treat dependency errors and panics as tool errors, not process exits.
   - Preserve cancellation compatibility with the agent turn loop.

## Rejected Alternatives

- **Current implementation indefinitely.** Rejected because it lacks standard ignore semantics,
  streaming search, mixed-encoding behavior, and budget controls expected from a mature agent
  search tool.
- **Top-level `ripgrep` crate.** Rejected because it is the CLI binary crate, not the right
  embedded API surface.
- **Host `rg` primary path.** Rejected because Talos search must work without host `rg` and must
  stay self-contained.
- **PCRE2 or SIMD features in the first slice.** Rejected until a later benchmark and dependency
  review proves the benefit outweighs portability and review cost.

## Implementation Guardrails

- Implement through `TOOL-011`.
- Keep `GrepTool` read-only and permission-auto-allow under the existing policy.
- Do not change `glob`, `find_symbol`, or TUI rendering in the same implementation slice.
- Add tests for `.gitignore`, `.ignore`, hidden files, symlink handling, binary files, invalid
  encodings, oversized files, output truncation, and path escape.
- Add a small deterministic benchmark/smoke fixture comparing current and ripgrep-backed behavior
  before removing the old code path.
- Update `TOOL-007` inputs after implementation so the tool-set audit uses the final search
  direction.

## Reversal Trigger

Revisit this decision if:

- `grep-searcher` / `ignore` integration requires unsafe code, native dependencies, or unbounded
  behavior that violates Talos constraints;
- benchmarks show the ripgrep library stack does not materially improve semantics or performance
  over a smaller `regex + ignore` implementation;
- the dependency graph or binary-size impact becomes unacceptable for the core tool set;
- `TOOL-007` changes the tool family design so grep moves behind a different progressive-loading
  or result-handle architecture.

## Related

- [TOOL-004: Spike - Evaluate Ripgrep As The Built-In Grep Engine](../backlog/active/TOOL-004-ripgrep-engine-evaluation.md)
- [TOOL-011: Ripgrep-Backed Grep Engine](../backlog/active/TOOL-011-ripgrep-backed-grep-engine.md)
- [TOOL-007: Built-in Tool Set Design Audit](../backlog/active/TOOL-007-tool-set-design-audit.md)
- [ADR-010: Git and Search Tool Dependency Boundary](010-git-search-tool-dependency-boundary.md)
