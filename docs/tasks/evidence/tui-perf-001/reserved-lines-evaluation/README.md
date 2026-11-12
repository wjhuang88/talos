# Borrowed lines and bounded segment capacity

Candidate: b7923cf7 plus source.patch.gz; receipt hashes decompressed patch bytes
and the immutable Debug binary. Baseline receipt: ../final-debug/binary-receipt.json.
Borrow stored styled lines during projection; reserve new segment strings up to
the smaller of remaining source bytes and four bytes per remaining viewport cell.
Zero-width runs can grow beyond the initial estimate. Scalar wrapping, offsets,
wide-character fallback and output ownership are unchanged.41 projection tests
passed. This is not whole-task completion or full-workspace validation.

Corrected heaptrack signal delivery and actual TUI metrics confirm baseline and
candidate each performed57 steady history rebuilds (resize events58/59).
Whole-run allocation calls21282134→11422251;temporary allocations6453014→4133016.
Rounded peak heap26.37M→26.26M. Whole-run counts include startup/shutdown and
are not exact per-rebuild counts; a finite profile is not an absence-of-leaks proof.
Normal service exit0,78.313s wall,35.471s CPU,403.3MiB peak,swap0.
CPU100%,affinity0,memoryhigh2G/max3G,swap0,Tasks128,Nice10. Profiler overhead
precludes using these latency values as ordinary budget acceptance.

## Ordinary Debug budget result

The three candidate-only resize rounds in `budget/` used 10000 history entries,
35 seconds per round, 5 seconds warmup and 500 ms input probes under the same
resource guard. P95/P99 histogram upper bounds respectively were
164.247866/164.247866, 170.772531/170.772531 and
160.545278/160.545278 ms. None establishes the frozen P95 <=150 ms gate;
all P99 bounds are below200 ms. Bounds are not exact percentiles, and this
candidate-only run cannot establish a causal latency improvement.
Service exited normally0:113.185s wall,22.428s CPU,430.2MiB peak,swap0.

Retain this change as a measured allocation reduction, not a resize-latency
fix. Do not repeat this matrix without a new diagnostic question. Final-source
checks, independent review, bounded-heap and native-terminal acceptance remain
separate requirements; the earlier full preflight does not cover this patch.

Independent incremental source review accepted the complete Cow/capacity diff:
UTF-8 iteration order, wrapping, style merging and logical offsets are unchanged;
the borrow does not escape projection. Capacity bounds apply to each initial
reservation, not aggregate memory. ASCII may reserve spare space, and zero-width
runs can grow. Review performed no execution and does not close latency gates.
