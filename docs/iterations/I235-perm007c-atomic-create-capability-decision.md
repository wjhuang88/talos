# Iteration I235: PERM-007-C Atomic Create Capability Decision

> Document status: Complete / Closed
> Published plan date: 2026-08-28
> Planned objective: decide and independently validate the platform capability needed by I234 to
> perform ADR-064 directory-relative atomic no-clobber creation.
> MVP deliverable: an accepted ADR amendment or successor decision that names an implementable
> safe capability API, supported-platform behavior, dependency/unsafe boundary, migration and
> rollback contract. This iteration changes no executable behavior.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 Sol mainline permission session |
| Work Slice | Decision-only PERM-007-C prerequisite: evaluate safe directory-capability implementations, platform support, dependency and unsafe implications, API boundary, migration, adversarial evidence and rollback. No Rust/Cargo/dependency implementation, permission behavior, model resolver wiring, Dashboard, Desktop, release or publication change. |
| Claimed At | 2026-08-29 |
| Source Issue | #188 |
| Governance Claim PR | #432 |
| Authorization Mode | Independent review |
| Authorization Evidence | Exact head `f47ed04670f37f626a2eb24ccffd67a6b576b576` passed CI `33189300888` and independent permission/security/API review `5455600711`; PR #432 merged to `main` as `71acbe0cb60fa204f359d33301b1e2af70125750` |
| Implementation PR | Not applicable; decision-only iteration |
| Last Updated | 2026-08-29 |
| Handoff / Release Condition | Requires independent permission/security/API review; only an accepted decision may authorize a later implementation slice. |

The proposed claim and Active status were ineffective until PR #432 reached `main` through
merge-time CAS. That merge occurred as `71acbe0cb60fa204f359d33301b1e2af70125750`; the decision
is now accepted, while I234 remains separately responsible for implementation.

## Nonterminal Inventory And Disposition

| State | Iterations | Disposition |
|---|---|---|
| Active | I234 | Preserve the local resolver work; its production positive path is waiting on this decision. |
| Review | None | No review iteration is displaced. |
| Planned | I207, I208 | Preserve unclaimed steering work; no overlap. |
| Blocked | PERM-007-D | Preserve as blocked until I234 and cross-surface evidence complete. |
| Paused | I164 | Superseded; do not restore. |

## Published Baseline

### Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| PERM-007-C capability decision prerequisite | PERM-007-C / Issue #188 | Refinement / Unclaimed | ADR-064; discovery from I234 local implementation | A reviewed, implementable and rollback-safe directory capability contract. |

### Scope

- Compare a safe capability dependency (preferred) with platform APIs requiring new `unsafe`.
- Define Unix and Windows support, unsupported-platform fail-closed behavior, and feature/dependency
  placement without changing the existing default tool feature combination.
- Specify parent identity, relative path, no-clobber, target-appears and parent-swap guarantees.
- Define the later I234 implementation boundary, API compatibility, tests, migration and rollback.

### Non-Goals

- No production Rust or Cargo changes.
- No model calls, auto approvals, permission policy changes, grant changes or WriteTool behavior.
- No Dashboard/Desktop, release, tag or publication work.

### Acceptance

- Independent permission/security/API review validates the complete platform and threat matrix.
- The accepted decision identifies a concrete safe primitive or explicitly keeps a platform ineligible.
- Dependency/unsafe authorization, public API migration and rollback are explicit.
- Both governance validators, YAML parsing, EOF and diff checks pass.

## Decision Questions

1. Can a safe capability dependency provide an already-held directory identity and relative
   `create_new` on every supported platform without leaking ambient path authority?
2. If not, which platforms remain fail-closed and which require a separately accepted unsafe ADR?
3. How are the same capability object and parent identity injected into the resolver and WriteTool?
4. Which adversarial fixtures prove target collision, parent replacement, symlink/reparse rejection,
   traversal rejection and no write outside the held capability?

## Candidate Assessment

The current preferred candidate is `cap-std` 4.0.3, subject to independent review and a formal
decision record. Its `cap_std::fs::Dir` API opens files relative to an already-held directory
capability and maps `create_new` to platform no-clobber primitives. Its Unix and Windows backends
hold directory identity through the operation. The later implementation must wrap this API so
absolute paths, parent components and symlink/reparse traversal are rejected, and must verify the
exact supported-platform behavior in CI. `cap-std`'s internal platform `unsafe` remains an
external-dependency security surface and must be recorded and reviewed before adoption. WASI is
unsupported and remains fail-closed.

No dependency is added by this decision-only iteration. Until the candidate is accepted, I234's
production positive path remains unavailable.

## Completion Evidence

Completion Commit: `71acbe0cb60fa204f359d33301b1e2af70125750` (PR #432 merge; decision-only
implementation evidence). Exact-head CI `33189300888` and independent review `5455600711` are
bound to the pre-merge decision head `f47ed046`.

## Residuals

I234 remains incomplete until the accepted decision's implementation contract is realized and
verified. PERM-007-D remains blocked and separately governed.

## Closeout Checkpoint

On 2026-08-29, PR #432 was merged after merge-time CAS. The accepted decision authorizes a later
I234 implementation slice to evaluate `cap-std` 4.0.3 as the preferred directory-capability
primitive, subject to its own dependency/security review and exact implementation evidence. This
closeout changes no executable behavior.
