# ARCH-034-R04-AG4: Symbol Traversal Containment

| Field | Value |
|---|---|
| Parent | ARCH-034-R04 |
| Finding | I181 AG-4 / Arborium traversal boundary |
| Status | Ready - governance claim proposed |
| Priority | P1 |
| Selected Iteration | I182 (Planned; claim not effective) |
| Preserved behavior | Symbol tool inputs, normal-tree JSON, result ordering, language detection, skip filters, parser fallback, and read-only permission classification |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Bound only the two directory traversals in `crates/talos-tools/src/symbol.rs`: do not follow directory symlinks; enforce reviewed depth, candidate-file, per-file-byte, and aggregate-byte budgets before parser admission; preserve normal-tree result order and object shapes; append one explicit final JSON notice object only when a budget skips or truncates work; add symlink-cycle, oversized-file, depth/count/byte-budget, and normal-tree compatibility tests. |
| Claimed At | Not applicable |
| Source Issue | None |
| Governance Claim PR | Pending |
| Authorization Mode | Independent review required |
| Authorization Evidence | Pending independent security review of the finalized exact-head claim. |
| Implementation PR | Not started |
| Last Updated | 2026-08-08 |
| Handoff / Release Condition | Claim is ineffective until the finalized governance-only PR is independently reviewed and merged to `main`; implementation must start from that merge or later `main`. |

## Problem And Boundary

`find_symbol` and directory-mode `list_symbols` recurse using `Path::is_dir()`, which follows
directory symlinks. A cycle can therefore recurse until stack overflow, which cannot be recovered
with `catch_unwind`. Both walks admit an unbounded number of files and bytes before synchronous
tree-sitter parsing.

The implementation boundary is limited to `crates/talos-tools/src/symbol.rs`, specifically the
two directory walkers, their traversal state, the directory-mode file-admission paths, and focused
tests in the same module. This child does not own parser panic/deadline containment (AG-5).

## Proposed Safety Budget

| Budget | Proposed value | Basis |
|---|---:|---|
| Directory depth | 64 | Finite stack bound; subject to independent security review |
| Candidate files | 10,000 | Matches `search_engine` bounded-search precedent |
| Per-file bytes | 10 MiB | Matches `search_engine` parser-input precedent |
| Aggregate admitted bytes | 50 MiB | Matches `search_engine` aggregate-input precedent |

The independent reviewer must explicitly accept or correct these values before claim merge. Limits
remain internal constants; no public input or configuration schema is added.

## Scope

- Inspect entries with non-following file-type/metadata APIs and never descend through a directory
  symlink.
- Share one small traversal-accounting mechanism between `find_symbol` and directory-mode
  `list_symbols` without changing direct-file behavior outside the admitted-byte check needed by
  those directory paths.
- Preserve the existing `should_skip_dir` filter set and native `read_dir` result order.
- Check metadata length before `read_to_string` and tree-sitter admission.
- On any skip or budget exhaustion, retain admitted results and append exactly one final JSON
  notice object with stable reason/count fields; do not mutate existing result objects.
- Add deterministic Unix symlink-cycle coverage and cross-platform fixtures for oversized files,
  depth/file/aggregate limits, and ordinary-tree compatibility.

## Non-Goals

- No parser `catch_unwind`, blocking adapter, async deadline, thread, subprocess, or cancellation
  work; AG-5 owns parser panic/deadline containment.
- No changes to `find_references` or `list_imports`, public input schemas, tool names/descriptions,
  language mappings, permissions, dependencies, Cargo manifests, ADRs, or TUI highlighting.
- No global filesystem-walker abstraction and no reuse/refactor of the separate grep engine.
- No sorting, canonical-output redesign, or silent skipping without a notice when a safety budget is
  triggered.

## Acceptance And Validation

- A directory symlink cycle completes without recursion through the link and reports the skip.
- A file larger than 10 MiB is not read or parsed and produces an explicit oversized notice.
- Depth 64, 10,000-file, and 50 MiB aggregate limits terminate deterministically and produce one
  final truncation notice with the applicable reason/count.
- For a fixture below all limits with no symlinks, `find_symbol` and directory `list_symbols`
  produce byte-identical JSON to the pre-change behavior, including result order and object shape.
- Existing `should_skip_dir` names, language detection, unsupported-file behavior, parser fallback,
  and direct-file tool contracts remain unchanged.
- Focused `talos-tools` tests, locked workspace preflight, Unix/Windows CI, both governance
  validators, and independent security re-review pass.

## Rollback / Residual

Revert the bounded traversal implementation if normal-tree compatibility changes. AG-5 remains the
owner for parser panic/deadline containment; AG-1 through AG-3 and AG-6/AG-7 remain separate R04
children and receive no authority from this claim.
