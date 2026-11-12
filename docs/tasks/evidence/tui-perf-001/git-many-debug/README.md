# Debug Git: 10000 untracked files

All six paired cases passed evaluated CPU/PSS/input/animation gates.
CPU median10.404%→4.471%. Each30s steady window completed15 scans;
max scan66.404ms and frame-weighted snapshot age max2.005s, below5s expiry.
Baseline52263c62; corrected candidate539adfd2 debug (sibling receipts).
Animation/history10000/input probes/process-tree enabled,35s runs,5s warmup.
No concurrent builds/tests/experiments. Fixture has10000 untracked files.
Observed age does not prove external-change publication, stale-result race
or native terminal behavior. Histograms are preserved in snapshot-age.json.
