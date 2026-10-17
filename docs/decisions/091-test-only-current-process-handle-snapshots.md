# 091: Test-Only Current-Process Handle Snapshot Boundary

## Status

Accepted contract proposal in #718; ineffective until target-main merge. Maintainer accepted
the scope packet on2026-10-14 at14:50 Asia/Shanghai. This contract becomes effective only with
finalized TEST-003/I302 atomic claim merge, independent protected
native/security review, exact-head CI and CAS. It does not authorize implementation beforehand.

## Context

#716 CI3175/3176/3177 failed Windows resource assertions. Isolated cold73->76 trace grows at first
Pipe(+1) and first Assignment(+2), then stays stable through40 exact injected faults. Existing
cleanup-return checks can prove known calls succeeded, not which remaining handles grew or why.
Do not warm up/raise thresholds/retry-to-green on an unproved initialization hypothesis.
ADR-068 owns production Job launch; this decision adds only a test diagnostic ABI boundary.

CI3178 additionally reached that final sample after all known cleanup BOOL checks passed; the
same growth remains. Successful known closes do not identify or qualify remaining resources.

## Decision

### Native Target And Feature Boundary

Only cfg(all(test,windows)) observer code may call documented Process Snapshotting functions:
PssCaptureSnapshot, PssWalkMarkerCreate, PssWalkSnapshot, PssWalkMarkerFree and PssFreeSnapshot.
GetCurrentProcess is the sole process target; never expose a PID/handle parameter to callers.
Borrow its pseudohandle without closing it. Use existing windows-sys0.61.2 with explicitly
reviewed diagnostic feature closure; verify actual production/test Cargo graphs and unify only
required features. A dev-dependency label does not prove isolation. No new package/version/lock,
public API, production entrypoint or launcher behavior is authorized.

Capture flags are hardcoded PSS_CAPTURE_HANDLES | PSS_CAPTURE_HANDLE_NAME_INFORMATION;
ThreadContextFlags=0. NAME_INFORMATION internally collects object names; the maintainer's
accepted scope includes this transient native collection, not disclosure. No basic/type-specific
union, handle tracing, VA clone/space, thread/context/IPT, breakaway, remote snapshot, memory,
other PID/OpenProcess, privilege enabling/escalation, external binary or driver.

### Output And Memory Boundary

Copy only numeric handle value and validated ObjectType/TypeName; never read/copy/format/log
ObjectName, unions, reserved fields, whole native Debug or snapshot dump. Check validity flags;
unsupported/unknown data stays explicitly unknown or fails collection, never guessed.
TypeNameLength is bytes; enforce even <=128, pointer null/alignment checks, length-delimited
UTF-16 validity and immediate bounded owned copy while marker lives. It need not be NUL-terminated.
No native pointer/reference escapes its owner.

Synthetic invalid-pointer fixtures test rejection predicates only; they must not dereference
fabricated addresses. Alignment/null/length checks are not arbitrary-address safety proof:
actual copying accepts only buffers returned by the currently owned live native walk marker.

Caps: <=1024 entries/snapshot, <=4 snapshots/worker, <=32 emitted changed entries and <=16KiB
diagnostic output. Over-limit/API/data errors produce bounded failure, not partial-complete/zero
evidence. Caps bound copied data/work, not all PSS internal allocations. Handle value/type changes
are observations; reuse and observation effects prohibit automatic leaked-object attribution.

### Ownership And Failure

One owner each for snapshot/marker, no concurrent descriptor use. Create initializes outputs;
capture/walk/marker/free statuses are checked. Marker frees before snapshot; explicit checked
cleanup also has error/panic fallback, with no double free. Local snapshot frees use the same
current-process pseudohandle. A cleanup failure remains evidence/failure; do not silently succeed.
Consume ownership before a release attempt. If it returns an error, do not retry that descriptor;
its state may be uncertain. Fallback cleans only still-owned resources not previously released.
Only documented successful end-of-walk terminates enumeration; other errors fail collection.

Run a separate owned snapshot worker under existing assigned-before-resume/kill-on-close outer
Job. Preserve60s event waiting and failure termination/10s terminal drain with continued bounded
output draining; native cleanup tails are not universally hard-bounded. No privilege escalation
or unsupported-host skip/fallback. Observer/helper failures must not leave descendants.

### Evidence And Original Regression

Preserve the independent no-snapshot cold worker, all40 exact fault checks, original baseline,
separate final count and +2 assertion. A failed original worker must not prevent safe collection
of another worker's diagnostic evidence, but remains failed afterward. Separate control records
expose snapshot/helper bookkeeping; do not silently move a measured baseline.
Unknown attribution is an honest diagnostic result. Successful collection does not prove legal
initialization/no leakage, complete TEST-003, fix the original defect, or authorize #716 merge.
Each requires its own acceptance/evidence/review and original full CI gates.

## Implementation / Validation / Exit Gates

Implementation begins only after effective ADR/claim merge, then refreshes combined #716 head
before stable publication. Separate I293/I302 commits/owner receipts are allowed in that PR;
no transfer of Search, D/E-F, #705 latency, Git or storage ownership.
Native Windows ABI and worker fixtures cover valid/unknown/invalid/oversized metadata, cleanup,
observer effects and original failure retention. Check feature graphs and pinned locked full
preflight/Clippy/tests, SDKs and both governance validators. No new skip/ignore/lint waiver.
Independent protected exact-head review of every unsafe/ABI and resource lifetime plus full
required CI/CAS is mandatory. No release. Remove temporary observation after evidence or retain
only with explicitly reviewed test contract/cleanup; preserve all failed history and residuals.

## Primary Contracts

- [Capture](https://learn.microsoft.com/en-us/windows/win32/api/processsnapshot/nf-processsnapshot-psscapturesnapshot)
- [Capture flags](https://learn.microsoft.com/en-us/windows/win32/api/processsnapshot/ne-processsnapshot-pss_capture_flags)
- [Handle entry](https://learn.microsoft.com/en-us/windows/win32/api/processsnapshot/ns-processsnapshot-pss_handle_entry)
- [Handle validity flags](https://learn.microsoft.com/en-us/windows/win32/api/processsnapshot/ne-processsnapshot-pss_handle_flags)
- [Walk](https://learn.microsoft.com/en-us/windows/win32/api/processsnapshot/nf-processsnapshot-psswalksnapshot)
- [Marker free](https://learn.microsoft.com/en-us/windows/win32/api/processsnapshot/nf-processsnapshot-psswalkmarkerfree)
- [Snapshot free](https://learn.microsoft.com/en-us/windows/win32/api/processsnapshot/nf-processsnapshot-pssfreesnapshot)

These specify ABI/data lifetime; no native PSS observation or root-cause result is claimed yet.

## Maintainer-Accepted TEST-004 Amendment — 2026-10-16

Maintainer accepted #717 proposal6217718821 at22:39 Asia/Shanghai. This governance-only amendment
becomes effective ONLY with finalized I305/TEST-004 claim merge into main; before then the original
+2 rule above remains authoritative. It does not rewrite I302's original baseline or failed history.

After that merge, TEST-004/#729 may replace the +2 pass/fail oracle with acceptance for enumerated
Talos-owned resources returned successfully from the existing Windows native boundary. Preserve
all40 exact faults, original baseline/separate final raw count trace, direct controls and all failed
runs. No warmup/raised threshold/new skip. Unknown process-wide/native resource attribution stays
an OPEN TEST-003/#717 residual. No cache/no-leak/global-repair or automatic #716 merge claim follows.

The replacement requires complete independently reviewed/enforced native acquisition/transfer/
release/termination/attribute site inventory, actual outputs and API results, role+acquisition
generation, and real negative operation-path controls. A balanced ledger or mutated JSON alone
is insufficient. Failed outputs never become owned. Attributes delete only after successful init,
with stable value/backing storage through Delete; that void call proves invocation only.

TEST-004 uses existing ADR-068 WaitForSingleObject on a still-owned CreateProcessW-returned handle.
Positive TerminateProcess plus bounded<=5s wait must observe signalled exit before close. Timeout/
failure/unexpected result fails the gate while all cleanup continues. An intentionally omitted
termination is observed with timeout0/WAIT_TIMEOUT, then explicitly terminated/waited/closed.
No OpenProcess, external PID lookup, extra rights/privilege or new observation API is authorized.
Real omitted-close/delete fixtures retain ownership/storage for explicit recovery; duplicate-release
fixtures prove rejection BEFORE a second native release, never blind CloseHandle reuse/debugger
exceptions. No fixture resumes an unassigned child; retain outer kill-on-close Job containment,
original worker/output bounds and panic-contained native integration. Native wait/recovery failure
remains a failed/incomplete result, not false success or a retry of uncertain descriptors.

Only cfg(test,windows) instrumentation/assertions may change. No production launcher/search/API/
permission/dependency/lock change. Positive/all40 and real negative sensitivity plus protected
exact-head review, full native CI and final CAS precede acceptance. Any untracked native acquisition
or broader process-resource/OS lifetime claim remains outside this narrowed proof and is recorded
in #717. I305 selection and amendment authorize implementation only after effective main merge.
