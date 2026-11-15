# Debug idle without metrics

All 18 paired samples completed. Histories 100/1000/10000, three rounds,
35s runs including 5s warmup. Candidate CPU medians measured 0.000% across
all sizes, within 0.20% budget; baseline medians 1.346/1.278/1.276%.
PSS checks passed. Zero reflects procfs tick resolution.
Baseline source 52263c62; candidate 539adfd2 uses corrected debug receipt.
No concurrent builds/tests/experiments; no workspace or input probes.
This does not establish Git or native-terminal acceptance.
