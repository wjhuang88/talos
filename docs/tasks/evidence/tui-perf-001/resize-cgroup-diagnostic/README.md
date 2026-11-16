# Single-run resize resource diagnosis

Sampler source8994abc7; immutable Debug binary source57603793, SHA in receipt.
One35s run,5s warmup,10000history,500ms input probes,80/120-column resize.
Same guard: CPU100%,affinity0,memory high2G/max3G,swap0,Tasks128,Nice10.
Normal exit0; service37.953s,CPU5.221s,peak424.7MiB,swap0. Raw sampler,
metrics and manifest are gzip archives. No compilation or runtime change.

Across sampled endpoints leaf cpu.stat nr_throttled/throttled_usec remained0;
memory.high/max/oom events remained0; I/O pressure totals did not increase.
CPU some pressure total increased220638us, full90154us over the sampled window.
Input-to-submit maximum82.538561ms, below both Debug150/200ms percentile budgets;
57 steady history rebuilds total4039.686558ms, maximum80.178796ms.

This diagnostic did not reproduce the earlier budget misses in the same binary.
It neither explains the earlier failures (missing concurrent resource evidence)
nor proves final-source or repeated-round acceptance. Global I/O pressure alone
must not be equated with stalls in this leaf group. Ancestor limits and per-frame
causality remain unmeasured. Retain prior failures, do not add an unproven runtime
shortcut or claim all resize budgets passed based on this one observation.
