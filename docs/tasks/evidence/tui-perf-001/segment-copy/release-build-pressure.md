# Symbol-enabled Release build pressure

The first Release build used CARGO_PROFILE_RELEASE_DEBUG=1 and STRIP=none,
with CPUQuota100%,affinity0,MemoryHigh2G,MemoryMax3G,SwapMax0. Confirmed running
but progress stalled: CPU usage61.034s to61.471s across observations, memory
~2.19GiB, memory.high221402 events, PSI full avg10=95.26/avg60=90.07 percent.
No OOM or max events. This was verified reclaim pressure, not inferred from silence.
Stopped explicitly using systemctl; child terminated SIGTERM15. The systemd
wrapper exit0 is NOT build success. Wall658.854s,CPU61.515s,reported peak2.1GiB.

Replacement uses repository default Release profile (fat LTO,1codegen unit,
strip=true, no extra debug symbols) under identical limits. No optimization,
CPU quota or memory limit relaxed. It requires separate immutable binary hash and
profile receipt. Suitable for ordinary runtime latency; not source-symbol profiling.
Replacement session63003/unit talos-perf-segment-release-plain pending at this note.

Replacement completed successfully:333.189s wall,306.126s CPU,peak2GiB,swap0. Immutable binary/profile receipt and three valid Release samples are now archived.
