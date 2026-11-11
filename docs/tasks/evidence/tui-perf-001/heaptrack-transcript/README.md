# Transcript revision allocator profile

Heaptrack1.5.0 (Arch package1.5.0-11, package SHA256
9b49c1e7b4accf9a02b558e9cecc0dd4a177342bf4da59583d791e505e153c8a),
three serial alternating Release pairs, history10000, transcript-commit1Hz,
35s plus startup/shutdown. Raw manifest, profiles, stacks, allocation-size
histograms, massif snapshots and process-tree samples are archived together.
Binary receipts and fixture patches are in ../transcript-release/; identical
fixture on baseline15ef42c1 and candidate e42b9c9c plus fixture-only patch.

Median allocation calls26139343 ->13386688; requested bytes3793180204 ->1947615179.
Each histogram count sum equals its profiler allocation total. All six peak
heap reports43.56M (rounded): allocation volume improves, peak does not.
These totals include initial10000-history construction and teardown, and must
not be reported as steady per-frame allocations. The large increase compared
with animation-only evidence corroborates revision-dependent projection cost;
this is not isolated causal attribution or leak proof.

Profiler launcher/interpreter/compressor overhead affects sampled CPU/PSS, so
those reports are not normal performance-budget evidence. Shutdown-live15.73K
is not proof of an application leak. Reproduce with replay.py after restoring
receipt-bound binaries and profiler at its documented paths. No concurrent
builds, tests or other experiments ran during these samples.
