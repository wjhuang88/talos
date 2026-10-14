# Proposed: Test-Only Current-Process Handle Type Evidence

Status: Proposed / Unclaimed. No PSS implementation, new native binding/feature or unsafe code
is authorized by this packet. Maintainer scope acceptance, a separate effective diagnostic
claim/ADR and protected review must precede implementation. SEARCH-001-C/I293 scope and
ADR-068/090 are not silently widened; #716 remains unmerged.

## Concrete Failure

#716 CI3175 failed the shared counter151->162; CI3176 failed isolated73->76. CI3177 trace
retains the same73->76: first Pipe73->74; all later Pipe/Job/AttributeList samples74; first
Assignment74->76; remaining Assignment/Resume samples76. Forty exact injected-error checks
completed; final +2 assertion failed. There is no per-attempt accumulation in this run, but a
first-use leak or legitimate internal initialization is not established. Original failed runs,
cold worker,40 faults and +2 remain evidence; no warmup/threshold increase/retry-to-green.

The current mechanically bounded correction checks the BOOL returned by existing cleanup calls,
after all cleanup attempts. It adds no native call/binding/unsafe block/rights/dependency change.
It can identify a failed close/termination, not the type or source of otherwise unaccounted handles.

## Recommended Additional Observation Contract

Use the documented Windows Process Snapshotting API inside a separate, test-only owned worker:

- Target only GetCurrentProcess, never caller-selected PID, OpenProcess or another process.
- Use the existing windows-sys0.61.2 package with an explicitly reviewed diagnostic feature closure
  for ProcessSnapshotting. Record any manifest/feature/lock consequences before implementation.
  No new crate, external binary/driver, administrator escalation or privilege enabling.
  Cargo feature unification must be checked: a dev-only declaration is not proof that the feature
  stays outside production build graphs. The observer code itself must remain cfg(test,windows).
- Capture handle metadata only. No VA clone/VA-space, process memory, thread/context/IPT capture,
  breakaway, remote duplication/free, process environment or file/registry contents.
- Prefer documented ObjectType where valid. If TypeName needs NAME_INFORMATION, that flag also
  collects object names internally: acknowledge this transient snapshot collection explicitly.
  Copy only bounded handle value/type into the diagnostic record; never access, format, copy,
  log or persist ObjectName, type-specific unions or a raw native entry/snapshot dump.
- TypeNameLength is bytes, not characters and not necessarily NUL-terminated. Check flag validity,
  null/alignment, even length and <=128bytes, and copy while the walk marker remains alive.
  Unknown/unsupported data is recorded as insufficient evidence, not a guessed type.
- One owner for snapshot and walk marker, with checked explicit release and panic/error cleanup;
  no concurrent use/double free. Release results are evidence, not silently ignored success.
- Proposed caps: <=1024 entries per snapshot, <=4 snapshots per worker, <=32 changed entries in
  emitted differences and <=16KiB output. Over-limit/errors fail diagnostic collection; the caps
  bound copied records, not all internal allocations of the Windows snapshot implementation.
- The existing outer Job owns the worker and descendants; preserve60s event waiting and failure
  termination/10s drain. No claim that this hard-bounds every native cleanup tail.
- Preserve a separate uninstrumented cold counter worker and its original40/+2 assertions.
  Snapshot collection must not move that baseline or turn a failed cold run green. Use separate
  controls to expose snapshot-created resources and observation effects before attribution.
- Handle values can be reused. A value/type difference alone is not proof of a new leaked object
  or a legal lifetime; reconcile it with cleanup evidence, reproducibility and primary API facts.

## Acceptance And Reversal

The added observation must distinguish helper failure, observation-created resources and the
original growth, or explicitly remain inconclusive. Native Windows tests cover valid type
records, bad/oversized metadata handling, release/error paths and observer cleanup. Full CI,
independent exact-head native/unsafe/security review and CAS remain mandatory. No default search,
production launcher, public API, security policy, dependency-version or release change.

If the API/privileges/metadata cannot meet this boundary, stop this diagnostic without falling
back to a privileged external observer or undocumented native API. Remove temporary observation
code after its evidence is recorded or retain it only under a separately reviewed test contract.
Diagnostic success does not close the original defect or authorize #716 merge by itself.

## Primary Sources

- [PssCaptureSnapshot](https://learn.microsoft.com/en-us/windows/win32/api/processsnapshot/nf-processsnapshot-psscapturesnapshot)
- [PSS_CAPTURE_FLAGS](https://learn.microsoft.com/en-us/windows/win32/api/processsnapshot/ne-processsnapshot-pss_capture_flags)
- [PSS_HANDLE_ENTRY](https://learn.microsoft.com/en-us/windows/win32/api/processsnapshot/ns-processsnapshot-pss_handle_entry)
- [PSS_HANDLE_FLAGS](https://learn.microsoft.com/en-us/windows/win32/api/processsnapshot/ne-processsnapshot-pss_handle_flags)
- [PssFreeSnapshot](https://learn.microsoft.com/en-us/windows/win32/api/processsnapshot/nf-processsnapshot-pssfreesnapshot)

These sources describe the API/flags/lifetimes, not this runner's +3 root cause or guaranteed
privilege/platform compatibility. The proposal uses no live PSS evidence yet.
