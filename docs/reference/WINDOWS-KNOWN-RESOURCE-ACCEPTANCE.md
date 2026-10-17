# TEST-004 Windows Known-Resource Acceptance

Effective authority: I305 / TEST-004 / #729, claim and ADR091 amendment merged in #730 at
c81782b6d922ec1b8bfa62e9c4a2914d746680bb. Implementation follows that merge. This is a test-only
acceptance boundary, not search activation or proof that process-wide growth is harmless.

## Native Site Inventory

The cross-platform `process_boundary::native_inventory` tests fingerprint the ENTIRE four source
files: process_boundary.rs, resource_acceptance.rs, handle_lifecycle_control.rs and handle_snapshot.rs.
Any source change, including imports/aliases/wrappers/macros/unsafe/transfer sites, fails closed
until the explicit baseline is refreshed after independent protected source review. Mutation cases
cover new uncovered acquisitions in each file, removed/misplaced actual acquisition observation,
raw-close bypass and disabled actual delete omission. The noncryptographic fingerprint is a change
detector, not semantic resource proof. A changed inventory test itself also requires source review;
no source-based check can authorize its own intentional weakening.

| Source / sites | Acquired or borrowed resource | Release / transfer / proof boundary |
|---|---|---|
| Launcher first and second CreatePipe | Four successful-return endpoints | Each observation immediately follows native success; original error cleanup calls are wrapped, real CloseHandle BOOL checked. Failed outputs are not adopted by audit. |
| Launcher CreateJobObjectW / SetInformationJobObject | One Job, kill-on-close configured before process | Known fault cleanup checked close. Successful transfer to WindowsJobControl and existing Drop/TerminateJobObject remain outside the all40 fault proof. |
| Attribute query/init/update | Query has no owner; only successful init creates a lifecycle | Fixed outer HANDLE_LIST storage lives through Delete in test builds; actual void Delete invocation recorded separately from backing allocation. Existing production failed-init/failed-output gaps remain residual, not silently repaired. |
| Launcher CreateProcessW | Successful process and primary-thread returned handles | Original flags preserve suspended creation; fault paths never resume an unassigned process. Real TerminateProcess BOOL plus owned-handle <=5s signalled wait, then real close. |
| AssignProcessToJobObject / ResumeThread | No new returned owner | Existing injected short-circuit points unchanged; fault-specific role masks prove reached stages. |
| Successful launcher closes and from_raw_handle | Thread, write endpoints, initialized attributes; read endpoints become std/Tokio files | Inventory and protected source review cover sites; no new successful-launch cleanup claim. Job/process transfer to supervisor remains existing ADR068 scope. |
| WindowsJobControl / supervision / pump | Transferred Job/process/readers | Existing Job terminate/drop, infinite leader wait, GetExitCodeProcess, close and file RAII; unchanged production behavior, not new all40 proof. |
| Inheritance fixture CreateEventW / CloseHandle | Unrelated inheritable Event | Existing concurrency fixture, separate from audited launch resources; unchanged. |
| handle_lifecycle_control direct acquisitions/cleanup | Pipe/Job/attributes/process/thread | Existing success-only adoption, initialized-only deletion, stable heap list, checked consumed release and panic fallback; mode7 count control remains diagnostic, not process-exit proof. |
| handle_snapshot capture / marker / walk / free | Current-process pseudohandle borrowed; owned snapshot and marker | Existing status checks, marker-before-snapshot cleanup, consume-before-release/fallback; observer self-target/output limits unchanged. Count API is read-only. |
| resource_acceptance | No independent native API or historical-descriptor authority | Fixed observer receives actual values/statuses and synchronous native closures from live launcher cleanup; closes never derive authority from historical values. |

## Executable Acceptance

Each of all40 original exact fault attempts has its own generation and role mask. Pipe expects2,
Job5, initialized-attributes6, Assignment8 and Resume8 resources, repeated8 times:232 resources,
208 checked closes,24 initialized Deletes,16 successful terminations and16 signalled owned-process
waits. Per-attempt validation precedes aggregation; generation sum820 supplements sequential source
coverage. An API failure/timeout or missing/mismatched event fails after remaining cleanup. The
raw41 count samples and independent final sample remain diagnostics, without a +2 oracle.

Mode8 is another exact worker under the existing assigned-before-resume outer Job. Actual operation
paths skip one normal close, one termination and one initialized Delete; the retained local owner
or native timeout0/live observation must detect the omission before a separate recovery call.
Recovery finishes inside the original resource frame, before assertions/serialization. A second
release request must be rejected by the consumed ownership guard before another native call.
Actual invocation counts and statuses distinguish detection from recovery success. No blind double
CloseHandle, uncertain-descriptor retry, PID lookup or new observer API is used. A failure cannot
become a successful receipt merely because its bookkeeping balances.

All9 worker outcomes are collected before parent assertions. Existing60s worker wait,10s failure
terminal drain, bounded output and kill-on-close Jobs remain; total suite time can increase.
Test-only native closures contain no allocation/assertion/user callback while owners are pending;
fixed observer operations cannot propagate ordinary errors into unfinished cleanup. Final failure
assertions and JSON occur after recovery. Existing production paths and native helper panic guards
are preserved. No universal OS-call hard deadline or process-wide leak absence is claimed.

## Limits And Required Evidence

Source inventory plus real native successes/negative sensitivity establish only enumerated,
successful-return resources in these injected fault paths. Unknown/native-private resource lifetime,
failed-call output ownership, production failed-init cleanup and successful-launch broader lifecycle
remain outside this proof. TEST-003/#717 retains unexplained +3 and untracked/native resources.
I302/TEST003, SEARCH-C/#643 and SEARCH-001/#624 stay open. Do not infer legal cache ownership,
OS leak absence, original global repair or automatic #716 merge.

Native positive/negative evidence and exact-head protected review/full CI are pending until their
actual receipts exist. Local cross-target compilation is not Windows execution. Original CI3213
and all earlier failures remain in I302/#717.


CI3219 on6bd0ddfb: the real outer resource fixture and audit/inventory assertions passed; run failed
only the synthetic CRLF checkout fixture (462/1 Windows). Successful harness output did not expose
raw numeric/mode8 receipts. Canonicalizing synthetic input corrects double-CRLF construction;
actual fingerprint/native sources/acceptance stay unchanged. Fresh full CI remains required.


## Bounded Acceptance Closure — 2026-10-17

Implementation #716 merged as b5b8241e2d8fa72d6e469bd96ccbdf54ff851c69; exact198cb751/tree29ac46f2
passed all6CI3221 jobs, native tools463/0 and downstream gates, protected review/CAS6233550377.
TEST004/I305 closes only this enumerated-resource assertion contract. Successful raw counters/JSON
remain captured; source-enforced acceptance is the evidence. TEST003/#717 remains OPEN for the
unexplained/private/untracked lifetime and failed-output/init/success-path limits above. No Search
activation or global leak/cache proof follows. Earlier pending/failure checkpoints are historical.
