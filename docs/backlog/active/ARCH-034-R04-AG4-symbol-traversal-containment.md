# ARCH-034-R04-AG4: Symbol Traversal Containment

| Field | Value |
|---|---|
| Parent | ARCH-034-R04 |
| Finding | I181 AG-4 / Arborium traversal boundary |
| Status | Ready - governance claim merged; implementation authorized |
| Priority | P1 |
| Selected Iteration | I182 (Planned; claim effective on `main`) |
| Preserved behavior | Symbol tool inputs, user-supplied root symlink resolution, normal-tree JSON, result ordering, language detection, skip filters, parser fallback, and read-only permission classification |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 architecture session 2026-08-08 |
| Work Slice | Bound only the two directory traversals in `crates/talos-tools/src/symbol.rs`: use non-following entry types for descent, skip directory-entry symlinks including file symlinks, preserve following of the user-supplied root path, enforce reviewed depth plus admitted-file/per-file/aggregate-byte budgets with a cap-plus-one bounded read before parser admission, preserve normal-tree result order and object shapes, append at most one discriminated final JSON notice only when work is omitted, and add the focused compatibility/security tests below. |
| Claimed At | 2026-08-08 |
| Source Issue | None |
| Governance Claim PR | #176 |
| Authorization Mode | Independent review |
| Authorization Evidence | Independent re-review comment `5226341754` approved claim head `0d4bd0882c45fccd0bc02e9868bcefaae751f3f1`; PR #176 merged by squash as `36980ecc5a238e17db38ddef99c66235851fcd48` after merge-time CAS. Implementation/security review comment `5230395611` independently approved implementation head `4b96882307173ded8264aa1c45cce129707ff65f` with no blocking findings after focused tests, both governance validators and exact-head CI `31266112256`. The review was posted through the shared `@wjhuang88` account but explicitly attests a distinct natural-person reviewer from the Codex implementer; the disclosure is the compensating audit control pending GOV-004. |
| Implementation PR | #177 |
| Last Updated | 2026-08-09 |
| Handoff / Release Condition | Claim is effective on `main` at merge `36980ecc5a238e17db38ddef99c66235851fcd48`; implementation must start from that merge or later `main` and requires its own independent review. |

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
| Files admitted for parsing | 10,000 | Count increments only after language detection, entry-type/symlink checks, and a successful bounded read admit the file to tree-sitter |
| Per-file parser input | 2 MiB | Deliberately lower than the 10 MiB scanner precedent because this boundary constructs a parse tree and AG-5 deadlines are excluded |
| Aggregate bytes admitted for parsing | 50 MiB | Sum of bytes returned by successful bounded reads and admitted to tree-sitter; unsupported and skipped files do not count |

Depth 64, 10,000 admitted files, and 50 MiB aggregate bytes were accepted by review
`5226241652`; the 2 MiB parser cap is the requested safer correction. Limits remain internal
constants; no public input or configuration schema is added. These byte budgets do not impose a
wall-clock bound on a pathological admitted parse; AG-5 retains that residual.

## Scope

- Use `DirEntry::file_type()` (or equivalent non-following `symlink_metadata`) for descent and
  admission classification; admission requires strict `file_type().is_file()` (never
  `!file_type().is_dir()`), preserving regular-file semantics and excluding FIFOs, sockets, and
  device nodes; do not use `Path::is_dir()`, following `fs::metadata`, or
  `canonicalize` for entries reached during traversal.
- Never descend through a directory symlink and never admit a file symlink encountered during
  traversal. Preserve the existing behavior that the user-supplied root passed to
  `list_symbols_in_path` may itself resolve through a symlink before traversal begins.
- Share one small traversal-accounting mechanism between `find_symbol` and directory-mode
  `list_symbols`; direct-file paths remain outside this child.
- Preserve the existing `should_skip_dir` filter set and native `read_dir` result order.
- Run language detection after entry classification, then use a bounded read of at most 2 MiB + 1
  byte. Reject overflow before UTF-8 conversion and tree-sitter admission instead of trusting a
  racy metadata length. Increment admitted-file and aggregate-byte counters only after the bounded
  read succeeds and the file is accepted for parsing.
- On any skip or budget exhaustion, retain admitted results and append at most one final notice,
  always as the last JSON array element and with this exact shape:

  ```json
  {
    "talos_notice": "bounded_traversal",
    "reasons": ["symlink_skipped", "oversized_file"],
    "counts": {
      "symlink_skipped": 1,
      "oversized_file": 1,
      "depth_limit": 0,
      "file_limit": 0,
      "aggregate_byte_limit": 0
    },
    "admitted_files": 7,
    "admitted_bytes": 2048
  }
  ```

  `reasons` contains only reasons whose integer counter is nonzero, in the stable order of the
  five `counts` fields;
  `counts` always contains all five integer fields. The reserved `talos_notice` discriminator
  cannot be produced by `SymbolResult` or `SymbolInfo`. If every candidate is omitted, the array
  contains only this notice. A normal tree has no notice and retains the original homogeneous
  result array byte-for-byte.
- Define notice counters mechanically: `symlink_skipped` counts traversed symlink entries refused;
  `oversized_file` counts bounded reads that return the cap-plus-one overflow byte. Root depth is 0;
  depths 1 through 64 are admitted, and `depth_limit` counts descents refused because they would
  create depth 65. `file_limit` or `aggregate_byte_limit` becomes 1 for the first otherwise-
  admissible file that would exceed its limit, after which traversal stops immediately; the other
  counters report only omissions observed before that stop.
- Add deterministic Unix symlink-cycle coverage and cross-platform fixtures for oversized files,
  depth/file/aggregate limits, and ordinary-tree compatibility.
- Keep directory-cycle, file-symlink-to-oversized-target, and symlinked-root tests under
  `#[cfg(unix)]`; keep regular oversized-file, depth, admitted-file-count, and aggregate-byte tests
  unconditional so Windows CI exercises every numeric budget.

## Non-Goals

- No parser `catch_unwind`, blocking adapter, async deadline, thread, subprocess, or cancellation
  work; AG-5 owns parser panic/deadline containment.
- No changes to direct-file `list_symbols`, `find_references`, or `list_imports`, public input
  schemas, tool names/descriptions, language mappings, permissions, dependencies, Cargo manifests,
  ADRs, or TUI highlighting. Their direct-file unbounded reads remain a named R04 residual.
- No symbol-output byte cap is added. Potentially unbounded serialization/output remains a named
  R04 residual; the 128 KiB search-engine output precedent is not silently imported into AG-4.
- No global filesystem-walker abstraction and no reuse/refactor of the separate grep engine.
- No sorting, canonical-output redesign, or silent skipping without a notice when a safety budget is
  triggered.

## Acceptance And Validation

- A directory symlink cycle completes without recursion through the link and reports the skip; a
  file symlink to an oversized target is not followed or admitted and cannot bypass the budget.
- A regular file larger than 2 MiB is bounded-read only through byte 2 MiB + 1, is not parsed, and
  produces an explicit oversized notice.
- Depth 64, 10,000 admitted-file, and 50 MiB admitted-byte limits terminate deterministically and
  produce one final notice with counters matching the definitions above.
- `list_symbols` on a symlinked root directory preserves existing root-following behavior while
  descendant entry symlinks remain skipped.
- When all work is omitted, the output is a one-element notice-only array distinguishable from a
  symbol result.
- For a fixture below all limits with no symlinks, `find_symbol` and directory `list_symbols`
  produce byte-identical JSON to the pre-change behavior, including result order and object shape,
  and explicitly contain no `talos_notice`.
- Existing `should_skip_dir` names, language detection, unsupported-file behavior, parser fallback,
  and direct-file tool contracts remain unchanged.
- Focused `talos-tools` tests, locked workspace preflight, Unix/Windows CI, both governance
  validators, and independent security re-review pass.

## Rollback / Residual

Revert the bounded traversal implementation if normal-tree compatibility changes. AG-5 remains the
owner for parser panic/deadline containment; AG-1 through AG-3 and AG-6/AG-7 remain separate R04
children and receive no authority from this claim. AG-4 provides no wall-clock bound for an
admitted pathological parse. Direct-file unbounded reads and unbounded symbol-output serialization
remain explicit R04 residuals and cannot be closed by this child.

## Independent Review Residuals (2026-08-09)

Review comment `5230395611` approved the exact implementation head and classified
the following as non-blocking, out-of-slice follow-ups. None may be silently
changed in I182:

1. `symlink_skipped` is counted before name/extension exclusion, so unsupported
   or already-skipped symlinks can produce a bounded-traversal notice without
   omitting otherwise admissible work. This matches the reviewed counter wording
   but conflicts with the broader "only when work is omitted" statement.
   [AG-10](ARCH-034-R04-AG10-symbol-notice-admissibility.md) owns CHANGE-CONTROL
   refinement of that observable contract.
2. Invalid UTF-8 in a supported file fails directory `list_symbols` but is
   skipped by `find_symbol`. The reviewer reproduced the same behavior on
   `main`, so it is not an I182 regression.
   [AG-9](ARCH-034-R04-AG9-symbol-decoding-consistency.md) owns any future parity
   decision and compatibility tests.
3. Absolute and parent-relative `path` values can escape `workspace_root`; root
   symlink containment also requires an explicit policy. This predates I182 and
   is outside its public-input Non-Goals.
   [AG-8](ARCH-034-R04-AG8-symbol-workspace-path-containment.md) is the separate
   security owner and requires an effective claim before implementation.
