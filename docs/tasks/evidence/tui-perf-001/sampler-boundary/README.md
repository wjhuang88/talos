# Sampler deadline and heap follow-up

Runtime: bounded projection `293ce0a2`; sampler fix `b007a547`.
Debug SHA-256: `d836607a95d7329b64ad9ddf6ca9e4bc75ccb28067a67684b4bd9d20af3fab47`.
All measurements use CPUQuota=100%, CPUAffinity=0, MemoryHigh=2G,
MemoryMax=3G, MemorySwapMax=0. No native terminal acceptance implied.

Corrected sampler stops action/probe dispatch at the measurement deadline.
Four PTY regression tests pass, including delayed process exit; independent
review approved the fix. New resize run: 570 driver actions / 570 effective screen-size changes (589 raw resize events),
coverage passes; nine 30-second heap-window medians are all 25,883,811 bytes.
Service: 313.857s wall, 127.496s CPU, peak 721.3MiB, swap 0.
The earlier 569/571 failure remains in ../bounded-segment. Its cause is not
retroactively established because old per-action timestamps were unavailable.

Transcript attribution reuses the existing bounded-segment transcript profile;
no additional transcript benchmark was run. The recorded heaptrack v3 events
reconstruct live bytes and reconcile all 30,362 exported timestamps with Massif
(760,592 allocation events). Between 29.997s and 269.999s, net live bytes grow
200,160: history_projection paths 105,360; StreamRenderState paths 87,120;
stream_opening_lines paths 7,680. Stack membership is mutually exclusive in
that priority order. This is a finite allocation-path delta, not proof of Rust
object ownership, universal leak freedom, or per-message allocations in general.
The fixture gives 834 net bytes/message over 240 messages.

The parser source retains original artifact-directory paths. To reproduce,
decompress attribute-heap.py and point its source variable at decompressed
bounded-segment transcript profile and Massif files. Trace frames are retained
in heap-attribution.json.gz; category rule is in heap-categories.json.gz.
Upstream heaptrack 1.5.0 exporter investigation explains zero-byte Massif child
trees; no profiler patch was used. Independent review verified grouping and raw resize counts. It identified stale
hardcoded coverage in the analysis script; fixed to validate current raw data.
The first talos_tui-matching symbol can be a generic allocator symbol, not a
Talos-owned function. Full event reconstruction was not independently repeated; independent review
approved this evidence batch after verifying raw coverage, grouping, archived
scripts and21 file checksums. This is not whole-task approval.

Unresolved: Debug transcript 479ms input tail; final-head CI; native resources,
deferred operations, second terminal and native Windows. Task remains Review.

## Bounded thread diagnostic

One35s transcript run,5s warmup, same immutable Debug binary and resource guards.
The diagnostic-only sampler adds10ms procfs thread CPU/scheduler/fault snapshots
and PTY output-read timestamps; no production code changed. 2,814 steady
observations, maximum gap16.662ms;60 input probes,0 skipped. InputP99 upper4.763355ms,
frame preparation max0.103463ms. Maximum observed per-thread interval runqueue
wait2.758326ms, no major faults in matched consecutive thread samples.
The479ms tail did not reproduce; observer overhead and absent exact per-frame
CPU timestamps prevent causal attribution. Prior failed acceptance remains open.
Service37.640s wall,2.584s CPU,166.6MiB peak,swap0. No repeated unchanged matrix.
