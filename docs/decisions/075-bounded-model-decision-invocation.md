# ADR-075: Bounded Model Decision Invocation

Status: Proposed
Date: 2026-09-13

## Context

Permission triage and protocol recovery both need a small, isolated model decision. Reusing the
full agent turn would inherit session context and tools, create recursive tool loops, and make
latency or cancellation boundaries unclear.

## Proposal

Provide one typed, bounded invocation primitive with explicit context/provenance, a dedicated
tool allowlist (possibly empty), deadline, cancellation token, token budget, retry cap and
correlation id. The primitive returns `Decision`, `Abstain`, or `Failure`; callers remain
responsible for policy and tool continuation. It must not inherit session, memory, permission
authority, or hidden tools, and diagnostics must be secret-safe.

Automatic protocol selection prefers Native only after capability evidence; otherwise it uses a
validated compatibility path. Protocol errors may be sent to the primitive for classification,
but authentication, timeout and general business failures do not imply fallback. Executed or
execution-unknown writes are never replayed. Permission Deny remains authoritative.

## Compatibility and migration

This proposal changes no behavior or public API. Separate child iterations must first define the
provider capability probe and migration adapter. Existing Native/TalosStrict/Compat configuration
remains supported until an implementation and compatibility test matrix are accepted.

## Required review

Permission, protocol and public SDK changes require independent security/API review. No production
implementation is authorized by this proposal alone.
