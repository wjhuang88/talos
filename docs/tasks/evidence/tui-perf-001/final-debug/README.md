# Final-source Debug performance follow-up

Binary sourcefec7a2e9, SHA in binary-receipt.json; guarded CPU100%, affinity0,
memory high2G/max3G,swap0,Tasks128,Nice10. Six serial runs: resize and growing
transcript,10000history,three rounds35s/5s warmup,500ms input probes,cgroup on.
Normal exit0;3m46.842s wall,36.758s CPU,58.1MiB service peak,swap0.

Transcript P95 upper4.194303ms in all rounds, P99 worst5.539786ms: passes20/35ms.
Resize P95 upper176.769150/179.034760/176.834676ms: fails150ms; P99 below200ms.
Raw reports, histogram metrics, coverage and manifest are compressed here.
No sampled leaf CPU throttling in any run. These are actual failed acceptance
observations; process success is not budget success, and PSS is not heap proof.

Prior same-limit single-run binary had57 history rebuilds totaling4039.686558ms;
current first round also57 totaling9475.168860ms. Both have zero sampled leaf
I/O stall increase and memory-high events; CPU-pressure some deltas220638us
versus299420us do not explain the multi-second difference by themselves.
Cargo artifact profile/features match. TUI runtime source diff only changes
input exit finalization; history projection and replay source are unchanged.
These facts do not establish the cause. A close-in-time fixed-binary comparison
is needed before attributing the difference to the exit repair or adding more
custom optimization. All prior failures remain valid evidence.
