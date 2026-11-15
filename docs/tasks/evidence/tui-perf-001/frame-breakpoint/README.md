# Per-frame debugger diagnostic

Unmodified immutable Debug replay SHA-256:
`d836607a95d7329b64ad9ddf6ca9e4bc75ccb28067a67684b4bd9d20af3fab47`;
runtime source `293ce0a2`. GNU GDB18.1; source breakpoints before preparation
body (`app/frame.rs:120`) and immediately before its timer drop (`:335`).
132 recorded frame pairs,0errors,0pending starts at exit; bounded to4096records.
A repeated same-thread start overwrites its prior start, isolated ends are ignored,
and truncation is not counted. This is not a general lossless-tracing guarantee.
CPUQuota100%, CPUAffinity0, MemoryHigh2G, MemoryMax3G, MemorySwapMax0.
Service40.144s wall,5.779s CPU,496.7MiB peak,swap0. No build or runtime source edit.

The only major-fault interval had33.135198ms wall,1.285636ms inferior-thread CPU,
0.581548ms runqueue wait,1majorfault and2minor faults. Most wall time therefore
was not charged as thread CPU or runqueue wait, but debugger all-stop overhead
is included. This is a useful fault/off-CPU lead, not measured fault duration or
causal proof of the earlier479ms event. The maximum whole-run interval44.360745ms
contained43.741241ms CPU and1876minor faults during initialization.

Replay coverage passed and60 input probes were dispatched without skips.
Debugger-run input maximum36.756401ms is diagnostic only, not passing acceptance.
Neither this instrumented run nor the previous procfs run erases the original
479ms failure. No repeated unchanged matrix or speculative production fix.

To reproduce, decompress trace.gdb and gdb-wrapper; update their artifact/binary
paths. Run the normal measure_tui_perf.py with the wrapper, transcript-commit,
history10000,seconds35,warmup5,input-probe,process-tree,cgroup and metrics enabled.
The sampler hashes the wrapper; the actual immutable binary hash is above.
Frame timestamps are relative to debugger Python initialization, not the sampler
origin; do not directly join their relative seconds. Frames record thread
scheduler runtime/wait and fault deltas via procfs while the inferior is stopped.
This adds observer overhead and does not trace fault addresses or off-CPU stacks.
The task remains Review; the prior long-tail cause and native gates remain open.

Independent Agent evidence review APPROVE (shared maintainer identity); no new
sampling during review. Both governance validators passed with0warnings.
