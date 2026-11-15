# Adjacent fixed-binary resize diagnosis

One serial prior/current pair, same guard and parameters as final-debug:
CPU100%,affinity0,high2G/max3G,swap0,Tasks128,Nice10;10000history35s/5s warmup,
500ms input,cgroup sampling. Manifest binds immutable binary SHA values;
prior receipt is ../resize-cgroup-diagnostic/binary-receipt.json, current receipt
is ../final-debug/binary-receipt.json. Normal exit0,76.254s wall,21.348s CPU,
426.5MiB service peak,swap0.

Prior history rebuild total9300.574956ms, current9155.893534ms. CPU30.7735%
versus30.8520%. Input P95/P99 upper173.459210ms versus175.079391ms. Both fail
the conservative P95-upper150ms gate, and pass P99-upper200ms. Histogram bounds
enclose quantiles; they do not prove the exact P95 exceeds150ms (lower134.218ms).

The older binary now resembles the current binary, rather than its earlier82ms
single-run observation. This comparison does not support attributing the cost
to input-exit finalization. It does not identify why the same binary varied,
prove equivalence generally, or close final performance acceptance. Preserve
all runs; investigate the common full-history reflow cost, not a speculative
rollback of the correctness fix. No further rerun of this pair is required now.
