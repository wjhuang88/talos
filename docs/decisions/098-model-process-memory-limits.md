# ADR-098: Model-Selectable Per-Process Memory Limits

Status: Accepted contract, effective only on independently reviewed claim PR #854 merge
Date: 2026-11-05
Owner: [I316 / PROCESS-001](../iterations/I316-model-process-memory-limit.md)

## Contract

Expose optional `memory_limit_mib` integer 64..65536, default policy 2048 MiB,
with request-to-step inheritance and explicit step override. Resolve once into a
typed policy retaining whether the value was explicit. Reject invalid input before
side effects. Permission admission binds the full request, never just the command.
An omitted step inheriting an explicit request retains explicit enforcement semantics;
inheritance must not downgrade it to the omitted-value best-effort policy.

Linux uses RLIMIT_AS (per-process virtual address space). macOS uses its available
RLIMIT_AS interface, but successful enforcement is not assumed by platform name.
Explicit setter failure refuses startup; omitted Unix values preserve the existing
best-effort 2048 MiB policy and must expose that limitation in tool evidence/docs.
Do not raise an inherited hard limit: choose the stricter existing bound or fail
explicit setup with a clear diagnostic; report the effective bound if stricter.

Windows uses JOB_OBJECT_LIMIT_PROCESS_MEMORY / ProcessMemoryLimit, per-process
committed virtual memory. Set the checked byte value together with the existing
KILL_ON_JOB_CLOSE flag before assigning/resuming the suspended child. Setter or
assignment failure prevents payload resume and follows existing owned cleanup.
Default 2048 MiB on Windows is new enforcement, not a claim about historical
behavior. This tightening may reject previously successful large-memory commands.

Neither mechanism is RSS nor aggregate process-tree memory. Descendants inherit
applicable process policy/Job membership under existing containment guarantees;
many individually bounded processes can still consume larger aggregate resources.

## Native boundary and migration

Extend existing child-side setrlimit and Windows Job configuration sites, not a
second launcher. Native policy changes require independent security review and
reviewed source-inventory fingerprints. Any additional getrlimit usage is limited
to a valid stack-local rlimit pointer, child-only inspection, checked return and
async-signal-safe pre-exec operations: no allocation, locks, formatting or panic.
No parent-process resource mutation or unchecked integer conversion is authorized.
Any effective-limit evidence returned from pre-exec uses a pre-prepared safe channel
or existing mechanism, without child-side allocation, formatting or locking.
Prepared scope, handle acquisition/transfer/release, assigned-before-resume, bounded
IO and cancellation must remain intact. Foreground exec must use existing platform
ownership rather than a raw spawn that bypasses enforcement.

Adding fields to public BashInput/ExecInput/ExecStep breaks Rust struct literals;
ship only in the next pre-1.0 minor, alongside explicit migration examples using
None for inherited defaults. JSON callers omitting the field retain default policy.
No 0.10 patch release or broad unsafe authority is granted.

## Evidence and alternatives

Apple upstream XNU commit `f6217f891ac0bb64f3d375211650a4c1ff8ca1ea`,
`bsd/kern/kern_resource.c` and `osfmk/vm/vm_map.c`, routes RLIMIT_AS to a VM map
size limit and rejects values below current map size. Local Darwin 25.5.0 Python
probes rejected 2/16/64 GiB settings; these do not prove all Rust child launches
fail or that macOS lacks enforcement. Native actual-launch tests must decide.
Windows SDK JOBOBJECT_EXTENDED_LIMIT_INFORMATION documentation defines committed
memory, independently from JobMemoryLimit. Do not substitute a watchdog/RSS poll
and call it an equivalent hard bound.

Before completion, prove actual child enforcement where setter succeeds and zero
payload launch on explicit setup failure. Unresolved platform coverage stays open;
no blanket Unsupported shortcut satisfies the original scope. Default evidence
must distinguish best-effort policy from confirmed enforcement.

Independent API/native-security design review on 2026-11-05 approved this contract
with the explicit-inheritance and safe evidence-channel requirements above. This is
not implementation approval or effective activation; finalized claim review and
target-branch merge remain required.
