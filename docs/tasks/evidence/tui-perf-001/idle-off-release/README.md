# Release idle without metrics

All18 paired samples completed. Histories100/1000/10000, three rounds,35s
runs including5s warmup. CPU median measured0.000% for each candidate size,
within0.10% budget; baseline medians0.202/0.234/0.235%. PSS checks passed.
Zero reflects procfs tick resolution, not literally zero instructions.
No builds/tests/other experiments during sampling. Baseline52263c62 and
candidate539adfd2 release binaries use sibling source receipts. No workspace
or input probes; this does not establish Git refresh or native terminal behavior.
