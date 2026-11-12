# Fitting-line fast-path evaluation: rejected

Three alternating baseline/candidate Debug resize pairs at10000 history,35s/5s
warmup,500ms input probes; CPUQuota100%, affinity0, memory high2G/max3G,
swap0,TasksMax128,Nice10. Source patch and candidate receipt retained here;
baseline receipt is ../p4-debug-guarded/receipt.json. Six normal exits0;
service3min46.161s,CPU52.705s,peak414.9M,swap0.

The proposed single-segment fitting-line path passed42 projection tests but
showed no consistent end-to-end improvement. History rebuild median7430.947ms
baseline versus8033.446ms candidate; counts57 each baseline,57/58/57 candidate.
Candidate P95 upper180.064/179.893/169.451ms all exceed150ms. Baseline also
missed that budget. Fixed baseline-first ordering and host drift limit attribution;
this is sufficient to reject an unproven optimization, not prove its universal
regression. The patch was removed from the worktree; no production change retained.
Do not treat this rejected experiment as completion evidence.
