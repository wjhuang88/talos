# ADR-036: zstd Compression for Session Log Archival

## Context

SESSION-004 introduces a segment-chain archival model for session logs. When a session segment is
compacted, the old segment is frozen and compressed to reduce storage footprint. Text session logs
are highly compressible (repeated structure, natural language, code), so compression ratio directly
affects long-term storage cost for active users.

The project's Hard Constraint #1 ("Rust first — no arbitrary C/C++ bindings") requires an ADR for
any C dependency. zstd's reference implementation is C; the most widely used Rust crate (`zstd` by
gyscos/zstd-rs) compiles the C source at build time via `zstd-sys`. This ADR records the scoped
exception, following the pattern established by ADR-007 (libc), ADR-008 (bundled SQLite), and ADR-020
(tree-sitter via arborium).

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
| --- | --- | --- | --- |
| No arbitrary C/C++ bindings | Hard | AGENTS.md #1 | Yes — via ADR with scoped exception |
| External deps must not crash the process | Hard | AGENTS.md #9 | No |
| Compression ratio matters for long sessions | Soft | SESSION-004 archival requirement | Yes |
| Build simplicity (no extra C toolchain burden) | Soft | Developer experience | Yes |
| Compression is behind a trait, swappable | Assumption | SESSION-004 design | Yes — validated by COMP-001 |

## Reasoning

### Why compress session log archives

Session logs grow unboundedly for long-running agent tasks. A 100-turn coding session with tool
output can easily exceed 5 MB in compact text format. With segment archival, older segments are
frozen and rarely accessed — ideal compression candidates. Text data compresses 3–5× with zstd,
directly reducing `~/.talos/` storage growth.

### Why zstd over alternatives

| Format | Ratio (text) | Speed (compress) | Speed (decompress) | Pure Rust? |
| --- | --- | --- | --- | --- |
| **zstd (C ref)** | **3.5–5:1** | **Fast** | **Very fast** | ❌ C binding |
| lz4_flex | 2–2.5:1 | Very fast | Very fast | ✅ |
| flate2 (miniz_oxide) | 3–3.5:1 | Medium | Medium | ✅ |
| ruzstd (compress) | 2.5–3:1 | Slow | Fast | ✅ |
| zstd-pure-rs | 3.5–5:1 (bit-exact) | Unknown | Unknown | ✅ (v0.1.1, early) |

zstd offers the best balance of ratio and speed for the archival use case (compress once at archival
time, decompress rarely on resume/search). The C reference implementation is the gold standard,
production-tested at massive scale (Meta, CDN, Linux kernel).

### Why the C binding is acceptable now

1. **Scoped exception**: zstd is used only for segment archival compression behind a
   `SegmentCompressor` trait. It does not affect the core agent loop, permission pipeline, or
   network code.
2. **Static linking**: `zstd-sys` compiles the C source statically into the Talos binary. No system
   zstd installation is required, matching the ADR-008 bundled SQLite precedent.
3. **Failure isolation**: Compression failures degrade gracefully — if compression fails, the
   segment is archived uncompressed. If decompression fails, the segment is reported as corrupt and
   earlier segments remain accessible. Per Hard Constraint #9, the C boundary is wrapped so failures
   never crash the process.
4. **Existing precedent**: libc (ADR-007), SQLite (ADR-008), tree-sitter (ADR-020), and wasmtime
   (ADR-032) are already approved C dependencies. zstd follows the same governance pattern.

### Why not pure Rust zstd today

- `zstd-pure-rs` v0.1.1 is bit-exact with C but very early (50 downloads, single author, ~50
  `unsafe` blocks). Production reliability is unproven.
- `ruzstd` v0.8.3 has solid decompression (39M downloads, used by cargo) but compression quality is
  explicitly inferior to C ("does not yet reach the speed, ratio or configurability of the original").
- The Trifecta Tech `libzstd-rs-sys` has not completed its encoder (Milestone 4 unfunded).

Pure Rust options are tracked in COMP-001 for future migration when maturity improves.

## Decision

**Approve** `zstd` (gyscos/zstd-rs, v0.13+) as a scoped C-binding exception for session log archival
compression, following the ADR-008 pattern:

1. **Scope**: zstd is used only behind the `SegmentCompressor` trait in `talos-session`. It must not
   be imported in other crates.
2. **Static link**: Use `zstd-sys` with static compilation (default). No system zstd dependency.
3. **Graceful degradation**: Compression and decompression failures are caught and logged. A failed
   compression falls back to uncompressed archival; a failed decompression reports the segment as
   corrupt without crashing.
4. **Trait boundary**: `SegmentCompressor` trait allows swapping to a pure Rust implementation
   (COMP-001) without changing archival logic.
5. **Build flag**: If build-time C compilation becomes a problem on any target, the feature can be
   feature-gated (`archive-compress-zstd`) with `lz4_flex` as the pure-Rust fallback.

**Reject** for the first implementation:
- Pure Rust zstd (`zstd-pure-rs`, `ruzstd` compression) — insufficient maturity for the primary path.
- `lz4_flex` as the default — ratio is 40–60% worse than zstd, meaningfully increasing storage.
- No compression — long sessions produce unacceptably large archives.

## Reversal Trigger

Revisit this decision when ANY of the following becomes true:

1. `zstd-pure-rs` reaches v0.3+ with >1000 downloads and independent production usage reports.
2. `ruzstd` compression achieves within 10% of C zstd ratio at equivalent levels.
3. The Trifecta Tech `libzstd-rs-sys` encoder (Milestone 4) is completed and released.
4. zstd C compilation causes build failures on a supported target (macOS, Linux, Windows).
5. A security vulnerability is found in the C zstd library that affects the Rust static build.

When reversing, update COMP-001, swap the `SegmentCompressor` implementation, and retire this ADR.

## Related Documents

- `docs/backlog/active/SESSION-004-binary-session-log-format.md` — session log format (revised)
- `docs/backlog/active/COMP-001-pure-rust-compression-migration.md` — pure Rust migration watch
- `docs/decisions/008-sqlite-bundled-storage.md` — bundled SQLite precedent
- `docs/decisions/020-tree-sitter-code-analysis.md` — tree-sitter C dependency precedent
- `docs/decisions/032-wasmtime-dependency-security-review.md` — wasmtime C dependency precedent
