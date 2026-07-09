# COMP-001: Pure Rust Compression Migration Watch

| Field | Value |
|---|---|
| ID | COMP-001 |
| Type | Research / Tracking Story |
| Priority | P3 |
| Status | Tracking — no implementation until trigger conditions met |
| Source | ADR-036 reversal trigger conditions |
| Depends on | ADR-036 (zstd C binding), SESSION-004 (SegmentCompressor trait) |
| Blocks | Nothing — migration is optional, not blocking |

## Problem

ADR-036 approves the `zstd` C binding (gyscos/zstd-rs) for session log archival compression as a
scoped exception to the "Rust first" hard constraint. This is pragmatic: the C reference
implementation is the gold standard, and pure Rust alternatives are not yet mature enough for the
primary path.

However, the Rust ecosystem is actively developing pure Rust zstd implementations. This story tracks
their maturity so that Talos can migrate to a pure Rust compressor when one becomes viable, retiring
the C dependency and the ADR-036 exception.

## Tracked Candidates

### 1. `zstd-pure-rs` (mahogny)

| Field | Value |
|---|---|
| Crate | `zstd-pure-rs` v0.1.1 (April 2026) |
| Pure Rust | ✅ |
| Compress + Decompress | ✅ — all 22 levels, bit-exact with C zstd 1.5.7 |
| Maturity | ⚠️ Early — v0.1.1, ~50 downloads, ~50 `unsafe` blocks in library |
| API | Streaming, dictionary, parametric — full parity with C `zstd.h` |

**Assessment**: Most promising. Bit-exact output means zero-quality-loss migration. But v0.1.1 with
minimal adoption is a reliability risk. Needs v0.3+, >1000 downloads, and at least one independent
production usage report before adoption.

### 2. `ruzstd` (KillingSpark)

| Field | Value |
|---|---|
| Crate | `ruzstd` v0.8.3 (May 2026) |
| Pure Rust | ✅ |
| Compress + Decompress | ✅ — both supported |
| Maturity | ✅ Production — 39M+ downloads, used by cargo for crate decompression |
| Compression quality | ⚠️ "Does not yet reach the speed, ratio or configurability of the original" |

**Assessment**: Most battle-tested pure Rust zstd. Decompression is rock-solid. Compression works
but produces worse ratios than C. For session archival (compress once, decompress rarely), the
compression ratio gap is the main concern. If the gap closes to within 10% of C, this becomes viable.

### 3. `libzstd-rs-sys` (Trifecta Tech Foundation)

| Field | Value |
|---|---|
| Crate | `libzstd-rs-sys` v0.0.1-prerelease.2 |
| Pure Rust | ✅ — c2rust translation, cleaned up |
| Compress + Decompress | ⚠️ Decompression complete; compression encoder incomplete (Milestone 4 unfunded) |
| Maturity | ❌ Pre-release |
| Funded by | Chainguard, Astral, NLnet Foundation (decompression); Sovereign Tech Agency (dictionary builder) |

**Assessment**: The "official" pure Rust zstd effort. Highest credibility (nonprofit, funded,
reference test suite). But compression encoder is incomplete and seeking funding. When Milestone 4
completes and the crate reaches stable, this becomes the strongest candidate.

### 4. `zenzstd` (imazen)

| Field | Value |
|---|---|
| Crate | `zenzstd` v0.1.0 |
| Pure Rust | ✅ — `#![forbid(unsafe_code)]`, `no_std + alloc` |
| Compress + Decompress | ✅ — levels 1-15 stable; levels 16-22 have known corruption bug |
| Maturity | ⚠️ Early — v0.1.0, decoder built on ruzstd |

**Assessment**: Interesting for the `#![forbid(unsafe_code)]` guarantee. Levels 1-15 are stable and
fuzzed. For session log archival (where compression level 3-7 is typical), levels 1-15 suffice. But
early version and known bug at high levels are concerning.

### 5. `lz4_flex` (PSeitz)

| Field | Value |
|---|---|
| Crate | `lz4_flex` v0.13.1 |
| Pure Rust | ✅ — `safe-encode`/`safe-decode` features avoid all `unsafe` |
| Compress + Decompress | ✅ |
| Maturity | ✅ Production — used in Android AOSP |
| Compression ratio | ~2.5:1 for text (40-60% worse than zstd) |

**Assessment**: Already viable as a pure Rust fallback. The `SegmentCompressor` trait (SESSION-004)
supports swapping to lz4_flex at any time. The ratio gap is the only barrier to making it the
primary. If storage pressure is low and C-free build is required, lz4_flex is the safe choice.

## Migration Trigger Conditions

Migration from C zstd to pure Rust is justified when ANY candidate meets ALL of:

1. **Maturity**: stable release (v0.3+ or equivalent), >1000 downloads, at least one independent
   production usage report or sustained maintenance (>6 months).
2. **Compression ratio**: within 15% of C zstd at level 3-7 on text log data (benchmark required).
3. **Decompression speed**: within 30% of C zstd for session log segment sizes (1KB–10MB range).
4. **Reliability**: no known data corruption bugs; round-trip fuzz testing evidence available.
5. **API**: supports streaming compression/decompression (needed for segment-level operations).

When a candidate meets all conditions:
1. Create a benchmark comparing C zstd vs the candidate on Talos session log fixtures.
2. Implement the candidate as an alternative `SegmentCompressor`.
3. Feature-gate the candidate behind `archive-compress-<name>`.
4. If benchmarks and reliability are acceptable, make the candidate the default.
5. Update ADR-036 with reversal record; retire the C binding exception.
6. Update `zstd-sys` dependency removal in `Cargo.toml`.

## Non-Goals

- Do not migrate until trigger conditions are met — premature migration risks reliability.
- Do not add a pure Rust compressor as a second dependency alongside C zstd — pick one.
- Do not block SESSION-004 implementation on this story — use C zstd per ADR-036.

## Required Reads

- `docs/decisions/036-zstd-compression-dependency.md` — current C zstd decision and reversal triggers
- `docs/backlog/active/SESSION-004-binary-session-log-format.md` — SegmentCompressor trait design
- [Trifecta Tech blog post](https://trifectatech.org/blog/announcing-zstandard-in-rust/) — libzstd-rs-sys announcement
- [zstd-pure-rs on crates.io](https://crates.io/crates/zstd-pure-rs)
- [ruzstd on crates.io](https://crates.io/crates/ruzstd)
- [lz4_flex on crates.io](https://crates.io/crates/lz4_flex)
