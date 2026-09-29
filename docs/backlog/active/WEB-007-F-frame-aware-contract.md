# WEB-007-F: Frame-Aware Browser Contract

| Field | Value |
|---|---|
| Story ID | WEB-007-F |
| Type | Architecture / API / Security Intake |
| Parent | WEB-007 / #452 |
| Source | #618; #520 Request B |
| Priority | P1 |
| Status | Intake / Unclaimed |
| Selected Iteration | None — API/security decisions below require acceptance first |
| Implementation PR | Not started |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #618 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-09-29 |
| Handoff / Release Condition | Accept versioning, trusted frame-origin admission and stale-reference boundaries through independent API/security review; then select a runnable iteration and merge its claim before code work. |

## Goal And Ownership

Embedding products need interactive browser actions inside nested and cross-origin frames without
copying security-sensitive reference and authorization logic. This child owns the public frame
protocol, semantic admission, permission binding and shared conformance fixtures under WEB-007.
The [proposed contract](../../proposals/WEB-007-F-frame-aware-browser-contract.md) is a review
candidate, not an accepted API or an implementation authorization.

WEB-007 owns the base browser contract and ManagedBrowserTool. Its separately claimed process
carrier and native browser delivery stories must consume this child's accepted contract and run
its conformance suite. Neither transport nor CDP code may redefine model schemas or authorization.
Host-supplied executors remain supported without adopting a process plugin.

## Scope And Constraints

Hard: closed schema before permission; opaque scoped identity; independent child-origin authority;
zero execution calls for references already stale at admission/dispatch; exactly-once delegation
for admitted, authorized, still-current actions; no retry; bounded redacted projections; opt-in.

Proposed Soft choice: separately negotiated frame protocol v2; preserve #452's 20-operation V1
schema. No crate name is reserved. Browser protocol types and fixture specifications belong in
the eventual WEB-007 protocol package; adapter admission/projection belongs in talos-tools;
permission integration requires review of talos-core/talos-permission APIs before activation.

Unvalidated assumption: trusted lifecycle state can be read synchronously before permission and
bound across approval to execution. Current AgentTool admission returns an execution mode, not
an opaque browser admission ticket. The proposal explicitly records this API gap; an unbound
lookup or mutable “last admitted request” cache is not an acceptable implementation.

## Exclusions And Dependencies

No production code, native dependencies, browser distribution, credentials/profile handling,
selectors/evaluate, default registration, product UI, replay or new permission bypass.
WEB-005/BROWSER-001 read-only ingestion and TOOL-014 disclosure retain their own contracts.
BROWSER-001 completion is not frame-interaction evidence. Base WEB-007 decisions and independent
API/security acceptance precede a runnable implementation claim; process-plugin/native delivery
can follow independently and cannot claim completion using only a fake executor.

## Acceptance And Delivery Gates

- [ ] Independent API/security review accepts the versioned operation schema, identity model,
  origin resource, admission ticket integration and lifecycle race handling in the proposal.
- [ ] The complete fixture matrix in the proposal is included unchanged or with reviewed rationale
  in implementation acceptance, for both host and process-backed executors where delivered.
- [ ] Inventory all Active/Review/Planned/Blocked iterations and record dispositions before selection.
- [ ] Select a runnable iteration: opt-in host executor composition with deterministic frame state,
  full registry → permission → execution → projection tests and an external consumer example.
- [ ] Record an effective target-branch Collaboration Claim and actual governance PR number.
- [ ] Implement and verify each acceptance fixture, pinned locked checks, full workspace tests,
  release preflight and independent exact-head API/security review before merge.
- [ ] Record implementation PRs and existing merged completion SHA(s) before Complete; only then
  reconcile #618/#520 downstream delivery. Neither this document nor a proposal closes #618.

## State Owners, Documentation And Residuals

This file owns #618 scope/readiness. WEB-007 owns parent dependencies; Product Backlog and Board
mirror this child. Future iteration owns execution evidence. All unresolved design/review and
activation items stay here; no unrelated iteration status changes are implied.

User-facing documentation targets for implementation: embedded Runtime SDK opt-in composition,
version negotiation, permission UI integration, stale-reference recovery and executor migration
examples. Exact existing doc paths must be selected with the implementation iteration.

## Required Reads

- [Parent WEB-007](WEB-007-optional-managed-browser-tool-core.md)
- [Contract and fixture proposal](../../proposals/WEB-007-F-frame-aware-browser-contract.md)
- [WEB-005](WEB-005-browser-session-continuity-research.md)
- [TOOL-014](TOOL-014-conditional-tool-backends.md)
- [BROWSER-001](BROWSER-001-read-only-browser-provider-connector.md)
- [Collaboration SOP](../../sop/AGENT-COLLABORATION.md)
- [Requirement intake](../../sop/REQUIREMENT-INTAKE.md)
- [Testing SOP](../../sop/TESTING.md)

## Evidence

2026-09-29: #618 and #452 inspected; no open PR reported at investigation time. Repository
AgentTool execution_admission and permission_profile inspected: both are synchronous, and the
admission result is an execution-mode value. Design remains Proposed, with no claim or iteration
selected and no browser runtime verification performed.

Local proposal validation: repository `scripts/validate_project_governance.sh .`,
`bash scripts/validate_collaboration_claims.sh .`, and the installed governance skill validator
passed with zero warnings. `bash scripts/assess_project_scale.sh .` reported high-risk,
release-managed, required worktrees; this proposal uses an isolated worktree. `git diff --check`
passed. Documentation-only intake: no Rust build, runtime fixture or security acceptance claimed.
Remaining closure gates are the unchecked acceptance rows above; #618 remains open.
