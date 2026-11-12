# Release Git: 10000 untracked files

Six paired cases completed; all generic CPU/PSS/input/animation gates passed.
CPU median 2.267% baseline to0.976% candidate. Each candidate30s steady window
completed15 scans; maximum scan20.490ms. Frame-weighted snapshot age observed
258 times per round, maximum1.994s, below5s expiry. Histograms retained separately.
Baseline52263c62; candidate539adfd2 release, sibling build receipts. History10000,
animation,input probes,process-tree on,35s runs with5s warmup,three rounds.
Fixture /tmp/talos-p0-git-fixtures-v2/many-untracked verified10000 untracked files.
No concurrent builds/tests/experiments. These samples do not prove external-change
publication, stale-result races, native terminal behavior or hard scan cancellation.
