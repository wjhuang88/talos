# WEB-007-F: Frame-Aware Browser Contract

Status: Complete

| Field | Value |
|---|---|
| Story ID | WEB-007-F |
| Type | Architecture / API / Security Intake |
| Parent | WEB-007 / #452 |
| Source | #618; #520 Request B |
| Priority | P1 |
| Status | Complete / Closed — implementation merged by PR #683; native browser/CDP remains downstream residual |
| Selected Iteration | I295 |
| Implementation PR | #683 (merged) |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / gpt-6 |
| Work Slice | WEB-007-F / #618 frame-aware v2 public contract, prepared invocation, browser-specific permission integration and host conformance only |
| Claimed At | 2026-10-06 |
| Source Issue | #618 |
| Governance Claim PR | #669 |
| Authorization Mode | Independent review |
| Authorization Evidence | PR #683 exact-head CI and Agent-role security/API review passed; merge-time CAS completed. |
| Implementation PR | #683 (merged) |
| Last Updated | 2026-10-09 |
| Handoff / Release Condition | Host contract complete; native browser/CDP/process-carrier remains downstream residual. |

## Goal And Ownership

Embedding products need interactive browser actions inside nested and cross-origin frames without
copying security-sensitive reference and authorization logic. This child owns the public frame
protocol, semantic admission, permission binding and shared conformance fixtures under WEB-007.
The [proposed contract](../../proposals/WEB-007-F-frame-aware-browser-contract.md) was accepted
in direction by the maintainer, and [ADR-086](../../decisions/086-frame-aware-browser-invocation-boundary.md)
records the decision candidate. Neither is an implemented API or code authorization before the
I295 claim reaches main.

WEB-007 retains its unclaimed V1 and native-delivery scope. For separately negotiated v2 only,
WEB-007-F owns the public browser contract, opt-in ManagedBrowserTool host adapter, prepared
invocation and permission integration selected by I295. Process carrier and native browser
delivery remain separate stories that must consume this child's accepted contract and run its
conformance suite. Neither transport nor CDP code may redefine model schemas or authorization.

## Scope And Constraints

Hard: closed schema before permission; opaque scoped identity; independent child-origin authority;
zero execution calls for references already stale at admission/dispatch; exactly-once delegation
for admitted, authorized, still-current actions; no retry; bounded redacted projections; opt-in.

Selected Soft choice: separately negotiated frame protocol v2; preserve #452's 20-operation V1
schema. No crate name is reserved. Browser protocol types and fixture specifications belong in
the eventual WEB-007 protocol package; adapter admission/projection belongs in talos-tools;
permission integration requires review of talos-core/talos-permission APIs before activation.

Selected proposed integration: an additive invocation-owned prepared path derives typed browser
resources after exact admission and consumes an exact request-bound one-shot authorization.
Current AgentTool admission returns only an execution mode and path authorizations are insufficient.
The proposal specifies root migration and fail-closed legacy entry points. Actual backend support
for document-bound actions remains an implementation assumption requiring real-browser evidence;
unsupported operation/document pairs must fail closed, never use global input fallbacks.

## Exclusions And Dependencies

The 2026-09-29 proposal/intake phase excluded production code; I295 now selects only the opt-in
v2 host implementation above, effective after claim merge. Still excluded: native dependencies,
browser distribution, credentials/profile handling, selectors/evaluate, default registration,
product UI, replay and any permission bypass.
WEB-005/BROWSER-001 read-only ingestion and TOOL-014 disclosure retain their own contracts.
BROWSER-001 completion is not frame-interaction evidence. V1 parent intake remains unresolved and
is not a prerequisite for the separately negotiated v2 host path; I295 still requires its own
effective claim and independent API/security review. Process-plugin/native delivery can follow
independently and cannot claim completion using only a fake executor.

## Acceptance And Delivery Gates

- [x] Maintainer accepted the proposed versioned schema, frame-origin resource and one-shot
  admission direction; independent Agent-role static design review approved the corrected
  proposal. This does not replace exact-head implementation security/API review.
- [ ] The complete fixture matrix in the proposal is included unchanged or with reviewed rationale
  in implementation acceptance, for both host and process-backed executors where delivered.
- [x] Inventory Active/Review/Planned/Blocked iterations and record dispositions in I295.
- [x] Select runnable I295: opt-in host executor composition with deterministic frame state,
  full registry → permission → execution → projection tests and an external consumer example.
- [x] Record an effective target-branch Collaboration Claim and actual governance PR number: #669 merged as 10f3715a77c4b1215455e89d26252d6b8bfa48e3.
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

2026-10-08 integration follow-up: local Agent/provider/Runtime/MCP paths and runnable deterministic
host example now exist. Independent uncommitted review returned REQUEST CHANGES; the current
I295 checkpoint records the remaining screenshot, lifecycle, typed-error and conformance gaps.
Provider compat/strict raw ingress received a local correction with 137 provider tests and
Clippy passing. This is neither an implementation approval nor #618 completion.

2026-10-08 current checkpoint: #669 merged on 2026-10-06; I295 claim is effective.
Local protocol/request/ticket code remains partial, unpushed and unintegrated with execution.
The maintainer requested completion and merge before handoff. The I295 closure ledger owns the
host-contract acceptance; no native-browser delivery is claimed.

## Completion Evidence

PR #683 merged all implementation changes to `main` as `06a5f6e6244f1b3dc9719033090f87ef485b1860`.
All six exact-head CI jobs passed for head `1385a7e72c1261596f6fd8b49a9535fc71fbb3b8` against
base `9a7b4b8cf014058c04d6b83fa5d9a35e7236ec03`; required Agent-role security/API review approved
the exact candidate. The accepted scope is the opt-in host contract and conformance boundary.
Native Chromium/CDP/process-carrier delivery remains a separately owned residual.

Completion Commit: `06a5f6e6244f1b3dc9719033090f87ef485b1860`

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

## Independent Review Follow-Up

The first API/security subagent reviewed head `7eec170666e7cdee0f2c9049ab58a372075ec137`
against base `7331dbfa5ba9554868b9b92c73b022876f880acf` and returned REQUEST CHANGES:
complete operation/output bounds, choose prepared API, define inventory/navigation resources,
exclude descendant content and require document-bound backend actions. The revised proposal
addresses all five; fresh exact-head review remains required. The governance subagent approved
that original head only as an intake proposal, not accepted contract or implementation delivery.
Both reviews are independent Agent roles using shared workspace/account, not separate humans.

2026-10-06: independent Agent-role static review of `main@9712d458` returned REQUEST CHANGES.
The proposal did not account for provider/MCP paths that discard duplicate JSON keys before tool
admission, the current registry's shallow validation, or broad legacy permission rules. The
proposal now requires raw-ingress validation or v2 rejection, full v2 admission, and a separate
fail-closed browser evaluator. This is a design correction only; fresh independent acceptance,
the selected iteration and effective claim remain pending. No production behavior or #520
downstream delivery is claimed.

2026-10-06: the same independent Agent role re-reviewed the local corrected proposal and returned
APPROVE for design only. It confirmed raw-argument fail-closed routing, full v2 admission and
dedicated browser permission evaluation; it did not review a committed exact head or production
code. The maintainer then accepted that v2 direction and explicitly authorized non-overlapping
I295 work alongside I293/I294. I295 is only a proposed selection until its atomic claim and
activation merge; the implementation and #618 closure gates remain unchecked.
