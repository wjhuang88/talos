# ADR-097: Validation Host Execution API Migration

Status: Accepted contract; implementation and release gates pending
Date: 2026-11-04
Owner: [I315 / VALIDATION-003](../iterations/I315-model-validation-agile-tools.md)

## Context

The public synchronous `run_validation_plan` starts processes inside conversation,
waits before draining pipes, retains unbounded output before truncation, and cannot
accept caller cancellation or confirm descendant cleanup. Keeping it alongside
the owned adapter preserves a known unsafe execution path. Profile allowlisting
does not establish permission admission.

## Decision proposed for independent API/security acceptance

Remove `run_validation_plan` and its private subprocess runner. This is a breaking
Rust API change, to ship only with the next pre-1.0 minor release (0.11 or later),
not a 0.10 patch. No compatibility shim may retain the old subprocess behavior.

Use synchronous `run_governance_validation(workspace)` for internal governance
only. It accepts no caller-built plan, arbitrary command or host executor. Planning
and rendering APIs remain independent of process authority.

Host checks use async `run_validation_plan_with_executor`. The host supplies its
selected workspace, permission admission, invocation-local cancellation and a
bounded executor; it retains the aggregate cleanup owner through shutdown. The
service reconstructs fixed checks from the selected profile and cannot accept
caller-injected commands. Unix/Windows adapters live in tools and reuse existing
owned process supervisors. Unsupported platforms return explicit NotStarted
evidence for host checks; internal governance remains available.

## Migration

- Conversation `/validate governance`: call the internal-only synchronous entry.
- CLI: await the same platform-owned adapter used by model tools, register signal
  handling before execution, and await operation plus aggregate cleanup.
- Async SDK consumers: authorize the concrete plan before executing it, inject a
  `ValidationHostExecutor`, await results and retain shutdown ownership. The trait
  itself grants no authority and does not enforce an untrusted implementor's bounds.
- Synchronous SDK consumers: create and own a Tokio runtime at their application
  boundary, drive the async call and cleanup there. Conversation does not create
  a hidden runtime, global executor or nested `block_on` compatibility path.

Normal Cargo checks may run build scripts and write build artifacts. Output
overflow, timeout and uncertain startup/cleanup are failures, not successful
truncation. Unix ordinary owned process groups retain ADR-082's containment limits;
Windows retains ADR-068's Job lifecycle boundary. No new unsafe authority is added.

## Acceptance and evidence

Independent API/security design review accepted this contract on 2026-11-04
by reviewer `/root/i313_mem007_correction_review`. This authorizes local migration,
not implementation acceptance or release; reviewer did not run tests.
Before merge: migrate all repository callers, check no legacy process runner remains,
test the internal-only and injected paths, actual permission denial with an execution
marker, CLI cancellation, native platform lifecycle behavior, and both external SDK
modes through standard preflight. Record current exact-head CI/review in I315.
Existing 0.10 releases are unchanged; release notes must identify the removal and
replacement before publishing the next minor. Full T1 acceptance remains owned by
VALIDATION-003, including unresolved preparation bounds and native Windows evidence.

## QUALITY-001 workload budget correction — 2026-11-06

The fixed host policy allows 15 minutes per check and at most 2 MiB per output
stream; Cargo test commands use `--quiet`. These are Soft host policy choices:
30 seconds/16 KiB rejected routine cold builds and passing workspace progress.
The model still cannot select arbitrary commands, deadlines, environment or paths.
Overflow remains a failure, cancellation and cleanup remain owned, and evidence
summaries retain their smaller presentation bound. The real CLI Cargo fixture
includes a 31-second build script, a 24 KiB diagnostic and 400 tests, then verifies
an actual test failure is not converted into success. Review/validation is owned
by [QUALITY-001](../tasks/2026-11-06-quality001-delivery-repair.md); this amendment
does not itself claim implementation completion or release.
