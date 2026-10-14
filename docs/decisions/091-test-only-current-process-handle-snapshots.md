# 091: Test-Only Current-Process Handle Snapshot Boundary

## Status

Proposed. Maintainer accepted the scope packet on2026-10-14 at14:50 Asia/Shanghai. This draft
becomes Accepted only with finalized TEST-003/I302 atomic claim merge, independent protected
native/security review, exact-head CI and CAS. It does not authorize implementation beforehand.

## Context

#716 CI3175/3176/3177 failed Windows resource assertions. Isolated cold73->76 trace grows at first
Pipe(+1) and first Assignment(+2), then stays stable through40 exact injected faults. Existing
cleanup-return checks can prove known calls succeeded, not which remaining handles grew or why.
Do not warm up/raise thresholds/retry-to-green on an unproved initialization hypothesis.
ADR-068 owns production Job launch; this decision adds only a test diagnostic ABI boundary.

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

Caps: <=1024 entries/snapshot, <=4 snapshots/worker, <=32 emitted changed entries and <=16KiB
diagnostic output. Over-limit/API/data errors produce bounded failure, not partial-complete/zero
evidence. Caps bound copied data/work, not all PSS internal allocations. Handle value/type changes
are observations; reuse and observation effects prohibit automatic leaked-object attribution.

### Ownership And Failure

One owner each for snapshot/marker, no concurrent descriptor use. Create initializes outputs;
capture/walk/marker/free statuses are checked. Marker frees before snapshot; explicit checked
cleanup also has error/panic fallback, with no double free. Local snapshot frees use the same
current-process pseudohandle. A cleanup failure remains evidence/failure; do not silently succeed.
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
