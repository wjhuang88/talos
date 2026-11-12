# Borrowed stored-line evaluation (not final acceptance)

The patch borrows stored StyledLine values while preparing projection rather
than cloning text/segments into an intermediate vector. Other semantic blocks
remain owned; wrapping, scalar offsets, fill and output ownership are unchanged.
41 TUI projection tests passed;3 unrelated session tests matched the filter too.
Source is4ac7e729 plus source.patch.gz (receipt hash is decompressed patch bytes).

One serial baseline/borrowed pair under CPU100%,affinity0,high2G/max3G,swap0,
Tasks128,Nice10;10000history35s/5s warmup,500ms input,cgroup sampling.
Normal exit0;75.639s wall,15.623s CPU,56.7MiB service peak,swap0.
Baseline receipt: ../final-debug/binary-receipt.json. Raw reports archived here.
History rebuild total7705.861376→5354.397645ms;CPU25.7013→18.2155%.
Input P95 upper175.075714→134.217727ms;P99 upper175.075714→149.436189ms.
Candidate passes150/200ms in this pair. Fixed ordering and observed host drift
preclude stable-benefit claims from one pair. Confirm before accepting the
optimization; full final-source gates, heap and native acceptance remain open.

## Reversed-order confirmation

Two additional borrowed-first pairs completed normally0,151.697s wall,
30.611s CPU,799.1MiB service peak,swap0, same guard and sampling parameters.
Raw evidence is in reversed-order/. History totals borrowed/baseline were
4220.274866/8022.597450ms then7351.487130/6054.027317ms. Borrowed P95/P99
uppers134.217727/134.217727ms then162.606683/162.606683ms; baseline uppers
169.305907/169.305907ms then134.217727/140.332813ms. Relative benefit reverses
in the last pair and candidate does not establish every-round P95 acceptance.
Do not claim a stable30% improvement from the first pair or task completion.
Independent read-only review found no semantic/lifetime defect in the Cow
change; that does not prove performance. Next investigate allocation counts or
function-local cost rather than another identical whole-process timing matrix.

## Profiler resize attempt excluded

The first heaptrack pair at target/tui-perf-artifacts/heaptrack-borrowed-resize-4ac7e729
completed normally, but is excluded from resize/allocation acceptance. Inspection
of heaptrack's launcher showed it retains a parent shell above the replay;
the sampler sent SIGWINCH only to that parent PID. TUI resize delivery was not
established. The sampler now targets the dedicated process group created by
start_new_session, with a real launcher/child regression proving signal delivery.
Three PTY probe tests passed. The patch happened while that profiler run was
active, so those samples must not be treated as a comparable pair or rehabilitated
by their exit codes. Raw files remain for audit; no performance claim is made.

## Corrected allocation profile

After sampler fix b7923cf7, the corrected pair explicitly verified actual TUI
resize counters. Baseline/candidate steady resize60/58, history rebuild57/56;
whole-run allocation calls21282136/14082162, peak heap26.37M/26.37M (rounded).
Different event counts and startup/shutdown prevent exact per-event claims.
Both profiles show String::push in project_line_chunks as a major realloc stack
(4640000/4560000 calls on the listed stack); borrowing removes intermediate
clones but does not remove this remaining expansion cost. Raw profiles, stacks,
massif, metrics, manifest and samples are under heaptrack/.
Normal service exit0,80.183s wall,40.534s CPU,410.2MiB peak,swap0, same limits.
Profiler timing is not latency-budget evidence. Reported end-of-process retained
bytes are not proof of runtime leaks; bounded-memory acceptance remains separate.
