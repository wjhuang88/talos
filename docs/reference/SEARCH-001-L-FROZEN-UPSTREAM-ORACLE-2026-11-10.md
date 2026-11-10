# SEARCH-001-L: Frozen upstream helper oracles — 2026-11-10

> Work Slice: [#901](https://github.com/wjhuang88/talos/issues/901), [I319](../iterations/I319-search-legacy-dependency-retirement.md).
> **Incremental test-only candidate**, not `rust-websearch` removal and not a new search provider.

## Fixed historical baseline

- Upstream `rust-websearch 0.1.2`, pinned source `Rust-Framework/rust-agent-framework@b30180a03609580e8b28dfb682284cb2ba249fba`.
- K1/#850 implementation #860 at `e32ea0a6220889e8346676d368110310171e2cce` passed CI3862 with actual-upstream runtime equality tests and independent API/network/security review.
- The new `search_backend/compat_helpers/frozen_oracle_tests.rs` **records expected outputs as byte-stable literal strings** for 9 entity cases, 10 HTML strip/cleanup cases, 16 redirect/percent-decoding cases, and the pinned 12-User-Agent FNV-1a64 fingerprint `0xe8d044e22b377885`.
- Intentional historical edge cases: limited seven-entry entity table, sequential `&amp;lt;` decoding, malformed percent encodings left unchanged, literal non-ASCII ignored in `uddg` unescape, NUL in decoded `%00`, NBSP whitespace collapsing and exact User-Agent ordering. **Do not improve semantics** inside dependency-retirement work without separate acceptance.
- Both **Talos-owned** helpers and the still-linked **actual upstream** crate are checked against each frozen expectation in the same test. Only after exact-head tests pass may a later L1 code slice remove direct upstream-oracle calls. No success is inferred merely from authoring this file.

## Safety and next dependency

This increment does not change network calls, protected SearchRouter, permission/endpoint authority, Cargo manifest/lockfile, SearXNG/Tavily defaults or old standalone `WebSearchTool`. The crate remains linked until its exported legacy path and transport traces have an independently reviewed replacement.

Prior SOURCE notice: `crates/talos-tools/LICENSES/rust-websearch-MIT.txt`; retain that notice after removing the crate. Required tests: `cargo test -p talos-tools --features network --locked` plus all platform CI/review on the candidate head.

L1 goal remains full build-package retirement with public API/behavior parity, not G-dependent global provider/route retirement. E/F/G/I and #624 keep their existing states.
