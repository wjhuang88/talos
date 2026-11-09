# PROCESS-001: Model-Selectable Process Memory Limit

| Field | Value |
|---|---|
| Story ID | PROCESS-001 |
| Status | Complete / Closed |
| Source Issue | #853 |
| Owner | I316 |
| Depends On | I315/T1 complete (#847 implementation, #852 closeout) |

Completion Commit: `04022b5ada916f9c01f04b5fa19662d3eb8efe27` (PR #855).
Native CI `42034824146`, independent review and merge CAS passed; full evidence
and platform/API boundaries are recorded in I316.

## Deliverable

The model can select a subprocess memory limit on bash and exec without bypassing
permission, cancellation or cleanup. Default policy remains 2048 MiB. This is
handoff T6, not conversation memory or an aggregate process-tree quota.

## Acceptance

- `memory_limit_mib` is an optional integer, 64..65536 inclusive; 1 MiB = 1048576
  bytes. Checked conversion; reject null, fractional, negative, zero and oversized
  values before launch. Omission differs from explicit 2048 for platform evidence.
- Request omission resolves to 2048. Exec step omission inherits the request;
  explicit step overrides it. Apply independently to sequential/parallel steps and
  every existing pipeline step. Do not interpret a per-process limit as a sum.
- Cover shell and direct exec foreground plus supported single-command background.
  Preserve unsupported background compositions and prove zero launch on rejection.
- Use one typed resolved policy at the final existing spawn owner; remove duplicate
  fixed-limit application that would reset a background override. Foreground direct
  exec currently lacks shared hardening and must be covered, including Windows Job
  ownership through an existing adapter. Do not introduce a parallel launcher.
- Full normalized request remains subject to permission admission; changed limits
  cannot reuse prepared proofs. No new ordinary Auto exception or ambient authority.
- Follow [ADR-098](../../decisions/098-model-process-memory-limits.md): actual
  platform mechanism and failure must be visible. Explicit limit setup failure
  refuses payload startup. Never claim enforcement from schema or setter success
  alone. Preserve omitted-value compatibility as documented, not silent universal
  enforcement. Do not weaken unrelated hardening to make a command pass.
- Real Linux child limits and bounded allocation failure, Windows Job limit and
  bounded allocation failure, and native macOS setter/enforcement or zero-launch
  failure evidence are required. Include inherited hard limits and parent isolation.
- Test default, override, request/step inheritance, exact boundaries, invalid types,
  foreground/background, sequential/parallel/pipeline, cancellation and cleanup.
  Prove a background override is not reset to 2048 by a second hardening hook.
- Document public Rust input-struct migration for next minor release. README en/zh,
  schema/tool descriptions and SDK/process docs distinguish virtual address space,
  committed memory, RSS and process-tree limits. No release authorized here.

## Verification / closure

Pinned Rust and locked checks, warnings-denied Clippy, workspace tests, standard
preflight including both SDK modes, native Linux/Windows CI, independent API/native
process-security review, exact-head CAS/merge, and owner-first existing implementation
Completion Commit are mandatory. All required gates passed; I316 records merged implementation, local standard
preflight/all-feature tests, native CI, independent review and merge CAS.

## Required Reads

- ../../tasks/2026-10-10-session-todo-handoff.md (T6)
- ../../iterations/I316-model-process-memory-limit.md
- ../../decisions/098-model-process-memory-limits.md
- ../../decisions/007-process-hardening-unsafe.md
- ../../decisions/068-windows-job-object-process-ownership.md
- ../../sop/AGENT-COLLABORATION.md
