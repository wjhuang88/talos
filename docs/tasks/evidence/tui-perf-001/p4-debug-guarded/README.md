# P4 guarded Debug follow-up

Current-source candidate only: receipt source57603793; six serial runs, scenarios
transcript-commit and resize, history10000, three rounds35s/5s warmup,500ms input
probes and process tree. CPUQuota100%, affinity0, memory high2G/max3G, swap0,
TasksMax128,Nice10. Normal exit0,3min46.737s, CPU21.449s, peak419.4M/swap0.

Transcript input P99 worst upper9.295705ms fits Debug20/35ms budget. Steady
output counts94/94/91 reflect window sampling. Resize P99 upper202.267295ms in
round1 exceeds200ms; round2 P95 upper176.078983ms exceeds150ms. Other rounds
and raw data retained; six successful executions do not imply performance pass.
CPU capping and host scheduling preclude direct attribution to runtime regression
or comparison with earlier unrestricted pairs. Do not increase limits merely to
pass. PSS maxima38862KiB transcript and44615KiB resize are not heap stability
proof. Remaining resize budget diagnosis and growth/resize heap acceptance stay
open; this evidence supplies Debug observations but does not close those gates.

## Existing-data diagnosis

History rebuild accounts for about97% of nested draw wall time in each resize
round (see resize-attribution.json); layout totals are only0.07–0.11ms.
`HistoryProjectionCache::ensure_projection` discards the prior projection on
width change, then `project_history_tail` rebuilds all logical text and visual
rows. `project_line_chunks` walks scalars and reconstructs styled segments.
This localizes the observed wall cost to the intended full-width reflow path;
it does not distinguish CPU execution from descheduling inside that timer.
No cgroup throttle timeline was captured, so do not infer that quotas caused
the misses or that removing them would pass. Any next optimization must retain
width-dependent wrapping, selection offsets, fill and wide-scalar fallback,
and use the existing full-output oracles. Do not alter animation/Git policy to
address this width-reflow measurement.
