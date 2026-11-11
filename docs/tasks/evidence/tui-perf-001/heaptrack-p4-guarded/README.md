# Guarded P3/P4 allocation comparison

Three serial alternating pairs, 10000 history, transcript-commit at 1 Hz,
35 seconds plus startup/shutdown. P3 and P4 receipt-bound symbol Release binaries
are identified by SHA in manifest.json; source receipts are in
../transcript-release/ and ../transcript-p4-release/. P4 predates the ANSI helper
extraction. Heaptrack version/package provenance matches ../heaptrack-transcript/.

Median allocation calls: 13386688 -> 765612. Reported peak heap: 43.56M -> 26.11M
in every pair. These are whole-run profiler totals, not steady-frame costs.
All six runs exited 0. Steady output bytes differ by one sampled update in two
P3 runs (5655 versus 5351); this harness did not capture exact business event
counts. Do not claim an exact per-event reduction from these totals.

Resource limits: CPU100%, MemoryHigh2G, MemoryMax3G, swap0, TasksMax128, Nice10.
Service exited normally0, runtime3min50.941s, CPU48.746s, peak453.8M, swap0.
Capped profiler CPU/latency/PSS are not ordinary performance-budget evidence.
Finite allocation-volume results do not prove absence of leaks under all loads.
Raw profiles, stacks, histograms and sampled process data are archived together.
