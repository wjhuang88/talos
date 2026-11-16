# Allocation profile: Release animation / 10000 history

Heaptrack1.5.0, Arch package1.5.0-11, downloaded from official configured mirror;
package SHA2569b49c1e7b4accf9a02b558e9cecc0dd4a177342bf4da59583d791e505e153c8a
matched local pacman sync metadata. Extracted to/tmp only; no installed project
native dependency or changed system security settings. Tool primary documentation:
https://github.com/KDE/heaptrack and packaged CLI help. Startup LD_PRELOAD records
allocator calls/backtraces; no runtime attachment to user processes.

Three alternating baseline52263c62/candidate df7411d2 pairs. Release symbols,
animation,10000history,35s nominal with5s sampler warmup,no input or workspace.
Profiler totals include initialization and teardown, not just steady30s. Raw
profiles, text analyses, allocation-size histograms, massif snapshots, sampler
process-tree reports, wrapper commands and manifest are archived. replay.py
captures exact local invocation; supply equivalent tools/binaries/paths to repeat.

Median allocation calls851478 ->816698 (-4.08%). Requested byte totals from
size*count histograms127667737 ->124076306 (-2.81%). Histogram counts exactly
match profiler allocation totals for every run. Peak live heap is31.32MB in both
profiles (profiler rounded display), so no peak-heap reduction is established.
Source backtraces include initial history projection/ScrollbackLine cloning;
the whole-run totals do not distinguish those from steady frame allocation.
Use repeated transcript-commit profiling before making a P4 incremental-history
decision. Current evidence does not justify a broad history rewrite.

Profiler runtime~37.2–37.3s includes its startup/exit overhead versus35s fixture.
CPU/PSS from this wrapper run include launcher/interpreter/compressor and must
not be compared as ordinary TUI resource budgets. The reported15.73KB “leaked”
value is memory live at shutdown, not proof of an application leak. Native
terminal resources, longer bounded-workload stability and allocation profiles
for other scenarios remain separate evidence. No builds/tests/other experiments
were launched by this task during profiling.
