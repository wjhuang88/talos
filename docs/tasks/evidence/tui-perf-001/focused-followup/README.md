# Focused performance follow-up

Source: `1eabf2964b5a81005f626644bd99053000518fdb`.
Immutable Debug SHA256: `0dc196e186926af4c0ee52ce56d4ec4174cb574ceffedece292aa7a54d53bf2c`.
Synthetic Linux PTY; initial 120x40, alternates 80x30 and 120x40 every 500ms;
10000 deterministic history entries; 35s per run including 5s warmup.
CPUQuota=100%, CPUAffinity=0, MemoryHigh=2G, MemoryMax=3G,
MemorySwapMax=0, TasksMax=128, Nice=10. No build or test suite run concurrently by this task.

## Resize

Three retained samples passed repository sample and metrics validators. Exact
150ms exceedance counts establish Debug P95 <=150ms in each sample. Conservative
P99 upper bounds are 127.729724, 160.067017 and 155.034721ms, all <=200ms.
Steady observed windows cover 29.863, 29.681 and 29.557 seconds. Counts are
58, 58 and 57. No CPU throttling or memory high/max/OOM event was observed.
These samples measure resize-event receipt to submitted frame; keyboard probes
were disabled. They do not establish simultaneous keyboard responsiveness,
Release performance, real terminal emulator cost, or causal improvement.

Collection incident: two initial follow-up runs used the same filename due to
shell variable interpolation. The overwritten run is excluded; the surviving
last run was copied byte-for-byte from resize-r-{sample,metrics}.json to r3.
A separate replacement run supplies r2. Thus retained chronological order is
r1, r3, r2. No missing run is represented as evidence.

## Heap follow-up

Two serial 305s profiles (5s warmup) were collected: repeated resize and
one committed message per second, both initially 10000 history entries.
Heaptrack 1.5.0 (previously verified Arch package) is external instrumentation.
Profiler CPU/PSS and latency must not be compared with ordinary budget runs.
Five-minute observations can identify sustained growth under these workloads;
they cannot prove absence of all leaks. Growing-history memory must account for
retained messages, not be judged against a constant total-memory target.

Status: this focused measurement batch completed; overall owner remains Review. Native Windows,
other terminal and deferred manual acceptance are not closed by this report.

Completed repeated-resize heap profile: 307.71s profiler runtime; nine 30s windows from t=30 to300 all have median live heap 25,943,809 bytes (first-to-last delta zero), maximum observed window heap 26,219,584 bytes. Actual resize actions and rendered state changes validated. This supports stable live heap for this finite fixed-history workload. Shutdown-reported allocations still live (15.90KB) require attribution before being labelled application leaks. Growing-history result follows.

Completed growing-history profile: 307.70s runtime, peak heap26.32MB. Nine30s windows increase by exactly25,050 bytes per window (835 bytes per second/message); first-to-last median delta200,400 bytes over240s. This is consistent with retained-history growth, but attribution of all835 bytes and cache-specific occupancy were not measured, so global boundedness/leak acceptance remains open. No superlinear growth observed in this five-minute workload.

Both heap profiles passed sample coverage and actual-state metrics validation. Service normal exit0, wall621.973s, CPU135.301s, peak363.8MiB, swap0. Heaptrack instrumentation and compressor included in service totals. Sampler launcher PSS is not application PSS; use heap profile for memory conclusions. No product code changes, builds or full test suites in this follow-up.

Residuals: cache-specific attribution for growing history; Release/keyboard/native measurements are not added by this batch; native Windows and second terminal remain unverified, previously deferred manual checks remain deferred. Overall task stays Review.

## Concurrent keyboard probes: failed P95 gate

Three additional runs inject a character/backspace pair every500ms while resizing,
using the same binary and guards. All sample/metrics validation passed; no probes
were skipped. Input sample counts113/113/112, >150ms counts22/29/39: all three
fail exact Debug P95<=150ms. P99 upper161.539418/166.999754/167.536079ms passes200ms.
The resize-only result above is valid only for its narrower workload; the original
resize-input gate remains FAILED. No threshold adjustment or passing relabel.
Service112.442s, CPU24.994s, peak53.5MiB, swap0. Next action is source/stack diagnosis
of full width-reflow under concurrent input, not another identical timing run.

Archive notes: large JSON/text files are gzip-compressed losslessly; decompressed
contents are the original artifacts. The 357MB replay executable is not committed;
its source/hash are recorded above, and local immutable artifact remains available.
Scripts/wrappers preserve original absolute paths for provenance, not portability.
checksums.json hashes the archived bytes. Summaries are reproducible from raw
metrics/samples and massif files after decompression.
