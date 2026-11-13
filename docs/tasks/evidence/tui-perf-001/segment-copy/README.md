# Full-segment projection candidate

Source1eabf296 plus archived source.patch.gz (decompress to reproduce the patch digest); exact binary and patch digests in
receipt.json. Copies a complete styled segment when scalar-width accumulation
fits the current row; falls back to existing wrapping otherwise. No new cache.
42 history projection tests passed. Local implementation commit: `f725b74b`. This is not merged delivery.

Three serial Debug input+resize samples:10000 history,35s including5s warmup,
500ms character/backspace probes, alternating80x30/120x40. Same guard as failed
reference:CPU100%,affinity0,MemoryHigh2G/Max3G,swap0,Tasks128,Nice10.
All sample/metrics validators passed; no skipped probes. Exact P95<=150ms passes
all3 (over150ms:1/110,0/109,0/112); conservative P99upper150.102260/149.815583/
149.082021ms passes200ms. Each observed57 resize actions and61 keyboard probes.
Service112.536s,CPU20.039s,peak53.5MiB,swap0. Previous failed source used
same scenario/settings but runs were not interleaved: do not infer an exact causal
speedup from wall/CPU deltas. These samples support the fixed Debug budget only.

Long segments that wrap incur an additional prefix width scan before
fallback. It stops when display cells exceed the remaining width, but zero-width
characters mean there is no fixed character-count bound; a long zero-width prefix
may be traversed twice. Release and other-workload regression gates remain required. Existing
heap/native results precede this patch; do not represent them as final-source
acceptance. Full locked workspace/preflight, cargo-xwin, independent review and
exact-head CI remain pending. Large artifacts use lossless gzip compression.

Independent read-only incremental review APPROVE at source diff SHA256
`a431031f0a0c7a2ddb19143aa70518e615c911ea865d7e551feadb87d40f4195`,
base1eabf296. No confirmed correctness blocker; reviewer ran no tests/builds.
The new scalar-segment equivalence test still uses the new fast path for individual
scalars, so it is not an independent implementation of the old algorithm. Existing
42 projection regressions supplement it; final-source gates remain separate.

Final-source checks2026-11-13: initial standard preflight101 (one Git maintenance
cleanup descendant remained RN); exact compiled failure rerun0 unchanged, xwin0.
Full preflight is not labelled passed. Compile/check/Clippy/governance succeeded;
remaining workspace/SDK gates pending. Raw logs and receipts are preserved here.

Final local gates completed2026-11-13: locked workspace tests exit0, 5319 passed, 0 failed, 0 ignored across148 summaries; external SDK fixture exit0. Combined service549.510s, CPU273.523s, peak2GiB, swap0. Source unchanged atf725b74b. Together with prior check/Clippy/format/governance/xwin passes, local gates are satisfied. Initial preflight invocation remains failed101 and its Git cleanup observation remains recorded; standalone unchanged test and subsequent full workspace passed. Independent evidence review found no blocker and verified raw hashes/threshold counts; Board now distinguishes baseline failure from candidate pass. Remote gates and residual performance/native acceptance remain open.
