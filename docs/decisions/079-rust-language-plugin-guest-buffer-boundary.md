# ADR-079: Rust Language Plugin Guest Buffer Boundary

**Status:** Accepted — I278 / CAP-001-D; effective through PR #575 merge `2d4e064f`.

Implementation note (2026-09-19): language-provider default fuel is 2,000,000,000, separately
bounded by the existing 500ms wall deadline and 64MiB instance memory ceiling. Cold Arborium
Rust query initialization measured ~897M fuel; a 13KiB input measured ~1.054B. The former 1M
fixture budget could not execute real grammars. Explicit caller budgets and generic Tool fuel
are unchanged. The I278 owner records measurement scope and near-limit safe-fallback behavior;
these figures are not portable latency guarantees or permission grants.

## Context And Verified Evidence

I278 must deliver real, independently buildable optional Rust/Python language Providers under
repository-root `plugins/`. At `d6b566be`, both language fixture modules return the constant zero;
they exercise fallback and lifecycle, not source-dependent language processing.
`WasmLanguageProvider::execute_payload` writes each request at linear-memory address zero.
That legacy convention cannot safely serve as a Rust slice pointer and can overwrite a compiled
guest's runtime data. ADR-032 approves dependency-internal unsafe, not new Talos-authored unsafe.

This decision extends the guest transport, not the language JSON contract, Bundle identities,
permissions, default distribution or Plugin activation policy. ADR-027/032/072/073 remain binding.

## Decision

### Isolated Rust Source And Build

Concrete Rust and Python language implementations live under `plugins/languages/rust/` and
`plugins/languages/python/`. They compile as Rust `cdylib` guests for `wasm32-unknown-unknown`,
outside the default host build/dependency graph. Shared guest-only source may live under
`plugins/languages/`; it must not depend on CLI/TUI/Desktop or pull optional parsers into the host.
Use a separately locked build and explicit package command. Generated artifacts are not sources.
The package command produces verified Bundle manifests with actual artifact SHA-256 values.
Installation remains separate from explicit activation.

### Additive Buffer Handshake

Keep ABI-v1 legacy modules working on the new host. New Rust guests return **ABI version 2**
from `talos_language_abi_version`; version 2 requires
`talos_language_alloc(request_len: i32) -> i32`. Version 1 retains the legacy offset-zero
transport; version 2 always uses the allocation handshake. Missing/wrong allocator signatures
for version 2 are errors, never grounds to retry through the legacy path. Existing hosts only
accept version 1 and therefore reject these new guests before writing request bytes. Unknown
versions remain rejected. The highlight/symbol JSON schemas and symbol protocol version stay
unchanged; memory transport version is not the symbol JSON version.

| Host | Legacy ABI-v1 guest | New ABI-v2 guest |
|---|---|---|
| Existing v1-only host | Existing zero-offset behavior | Reject at version validation; never invoke request transport |
| I278 host | Retain zero-offset compatibility | Require allocated request buffer and exact v2 signatures |

This is an additive host capability and explicitly incompatible new artifact for old hosts,
not a claim of reverse compatibility. Build/package instructions must state the minimum host
support; executable version rejection, not that documentation, enforces the boundary.

For the new guest, each invocation uses a fresh isolated instance:

1. Establish import rejection, store memory/table/instance limits, fuel and a total execution
   deadline **before instantiation**, including start functions and any allocator execution.
2. Bound the encoded request before allocation. Invoke `alloc` once; the guest creates an
   initialized owned byte buffer and retains ownership until the invocation is discarded.
3. Require a nonzero pointer and validate the complete request range against current guest
   memory after allocation. Copy the request through Wasmtime's checked safe memory API while
   guest execution is stopped and no guest Rust reference to the buffer is held across the call.
4. Invoke the existing `talos_language_run(ptr, len) -> i64` once. The guest checks that the
   arguments exactly match its own retained allocation before parsing. It accesses that owned
   buffer using safe Rust, rather than constructing a slice from arbitrary caller pointers.
5. The guest retains its owned response until instance disposal. The packed upper 32 bits are
   the response offset and the lower 32 bits are the length, as in the existing ABI.
6. Check output length against policy **before copying**, then validate the memory range.
   Decode through the existing typed host contract. No free export or ownership transfer to
   the host is needed because the entire instance/store is discarded after the request.

The same controls apply to **admission/version probing**, including direct callers of public
`validate_language_provider_abi`: it instantiates a guest and executes its version function.
Import rejection, memory/table/instance limits, fuel, total deadline and panic containment must
exist before any admission start/version function runs. There is no unbounded preliminary
instance or probe. Keep the public validation signature compatible, with safe finite defaults;
internal admission with explicit limits may use a stricter bounded helper. Old hosts already
reject a well-formed v2 version result, but this does not retroactively fix their admission
resource gaps. Newly shipped guests must have no start section and a constant version export.

All guest input/output borrows and mutations finish before returning an exported pointer.
No reentrant host imports are allowed; host copying and guest borrowing never run concurrently.

Do not call the legacy path after a failed new-ABI invocation, restart work with fresh fuel, or
extend the deadline between phases. Independent stores must not interrupt one another merely
because they share an engine. The implementation must prove request isolation and bounded
cleanup; a timer/worker per request may not accumulate after completed invocations.

### Narrow Unsafe Authorization

The exception permits **only Rust 2024 unsafe symbol-export attributes**, such as
`#[unsafe(export_name = "talos_language_alloc")]`, in the guest ABI modules under
`plugins/languages/`, compiled only for `wasm32-unknown-unknown`. Each export must document its
unique symbol, exact signature and confinement to a standalone guest module.

This does **not** authorize unsafe blocks, raw-pointer dereferencing, `static mut`, native FFI,
host unsafe, unchecked memory access or native dynamic loading. Input/output ownership and
parsing must use safe Rust. If safe owned buffers cannot satisfy the handshake, stop and amend
this decision with a concrete safety proof; do not silently broaden the exception.

### Resource And Failure Contract

No WASI or host-function imports. No ambient filesystem, environment, process or network.
Instantiation, allocator and run failures return bounded recoverable errors and existing
domain fallback, never host abort or permission bypass. Memory/table counts and sizes must be
limited before start/allocator/run; output and encoded input must be bounded before copying.
Guest panics may trap, but all relevant host dependency entrypoints remain panic-contained.
Permission gating and lifecycle withdrawal continue to govern real consumer access.

## Alternatives And Tradeoffs

- Keeping the fixed zero-address convention only serves hand-controlled fixtures and risks
  corruption in ordinary Rust guests; rejected for new implementations.
- Handwriting JSON parsing and language processing in WAT avoids Rust exports but creates an
  unnecessary assembly implementation; rejected as the production delivery strategy.
- Raw-pointer guest slices need a larger unsafe proof; unnecessary if the guest retains its own
  initialized buffers. No raw-pointer exception is requested.
- A full component-model/WIT migration would change more contracts than this source-delivery
  slice needs. Deferred; the additive handshake preserves existing artifacts.

## Validation And Acceptance Gate

Independent Agent-role security/API review must inspect this decision against the existing ABI
before acceptance. An effective I278 claim must then exist on `main` before implementation.
Decision acceptance is not proof that these implementation checks have passed:

- Build both real guests independently with the pinned toolchain and locked dependencies;
  inspect exports, imports and default-host dependency isolation.
- Package, verify/install and explicitly activate each artifact; exercise actual shared highlight
  and symbol-tool consumers with multiple different sources, including Unicode byte offsets.
- Compare legacy and new transport success/failure behavior; test missing/wrong allocator
  signature, zero/out-of-range pointer, allocation trap, start/run loop, fuel/deadline exhaustion,
  oversized input/output and initial/growing memory/table limits.
- Exercise admission start/version loops, oversized initial memory/table and direct public
  version-validation calls. Prove old-host rejection of actual v2 artifacts before request
  memory writes, new-host/v1 compatibility and new-host/v2 success; reject unknown versions.
- Test concurrent invocations, failed-then-successful calls, lifecycle withdrawal, corrupt/missing/
  incompatible artifacts and unchanged permission denial.
- Inspect every unsafe occurrence; only the reviewed guest export attributes may be added.
- Run focused and required workspace checks, then independent exact-head security/API review.

## Reversal Triggers

Revisit before implementation if the safe ownership scheme requires unchecked guest memory,
existing ABI-v1 compatibility cannot be preserved, per-invocation resource limits cannot be
enforced, or language consumer fidelity would require scope beyond I278. No hidden fallback
may turn unsupported behavior into a claimed successful language result.

## Decision Evidence

Independent Agent-role security/API reviewer `/root/i278_abi_decision_review` approved proposal
blob `1a6182b425741a0f6da0825b69c4ab8595241e72` against base
`d6b566be25205a5abe1202dabeb0cee09ea81cce` on 2026-09-18 after admission/probe and old-host/new-guest
compatibility findings were corrected. The maintainer authorized unattended single-maintainer
execution and independent subagent review. PR #575 binds final exact-head review, scoped CI and
merge-time CAS; acceptance is ineffective until merge. Shared account/workspace, Agent-role
separation only. This decision evidence does not substitute for implementation safety evidence.

2026-09-18 activation: #575 merged as `2d4e064f94a7c54d65444dde195c3069a8509ddd` after exact-head
CI `35359610144`, independent APPROVE `5731895048` and CAS `5731908465`. Acceptance is effective;
the preceding pre-merge checkpoint remains historical evidence.
