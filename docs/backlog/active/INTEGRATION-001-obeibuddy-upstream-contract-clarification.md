# INTEGRATION-001: OBeiBuddy Upstream Contract Clarification

| Field | Value |
|---|---|
| Story ID | INTEGRATION-001 |
| Type | Integration contract clarification Spike |
| Status | Intake / Unclaimed |
| Source Issue | #520 |
| Selected Iteration | None |
| Implementation PR | Not started |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #520 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-09-09 |
| Handoff / Release Condition | Select a bounded contract investigation and establish an effective claim before execution; any resulting implementation belongs to separately governed domain owners. |

## Scope And Value

Resolve the two upstream questions in [Issue #520](https://github.com/wjhuang88/talos/issues/520)
without treating downstream integration assumptions as verified Talos guarantees. OBeiBuddy reports
pinning `fad6e24e4b716b90b776b358d92ff69015688adf`; this is a reported consumer baseline, not a
fresh compatibility test. Intake was checked against main `53fc57b3`.

1. Runtime: inspect the pinned and current shutdown APIs and tests for active provider/tool,
   awaiting-approval and continuation paths. Explain admission fencing, stopped work, durable
   finalization, unknown outcomes, deadline exhaustion, handle drop and recovery obligations.
   Reconcile Issues #45/#49 and provide exact commit/tag evidence; do not declare v0.9 production
   stability based on an Issue comment alone.
2. Browser: decide whether frame-aware work extends an existing owner or needs a separate child.
   Compare WEB-007/#452, BROWSER-001/#508, WEB-005 and TOOL-014 before selecting crate/API scope.
   Preserve the requested tab/frame/browser/snapshot-generation identity, stale/detached rejection,
   child-origin permission isolation, closed-schema admission, executor-boundary revalidation,
   bounded redacted output and no automatic replay constraints.

## Dependencies And Exclusions

Runtime findings must follow accepted ADR-063 and existing runtime/session authorities; this
Spike does not replace them. Browser discovery/read-only scope in BROWSER-001 does not authorize
interactive native browser operations. WEB-007 already owns related optional host-executed browser
intake; do not create duplicate implementation ownership.

No runtime/API/schema, permission, persistence, dependency, UI, release or publication changes.
No default browser registration, arbitrary JavaScript fallback, credentials/cookie extraction,
browser binary distribution or Talos ownership of OBeiBuddy accounts/profile/UI. No I253 scope
expansion or child activation. Public API or security changes require their own decision,
migration, selected iteration, effective claim and protected review.

## Acceptance And Evidence

- [ ] Each Request A question has a source/test-backed answer or an explicit unresolved gap.
- [ ] Each Request B decision identifies the existing owner or proposed bounded child, API boundary
      and required security/compatibility gates; no claim that an intake is delivered behavior.
- [ ] Same-origin, cross-origin, nested, delayed, duplicate-name, rebuilt and stale frame cases are
      retained in any resulting implementation acceptance, including no parent-authority inheritance.
- [ ] Respond separately to A and B with Accepted, Accepted with changes, Deferred or Rejected,
      reasons and exact evidence. A design document cannot certify implementation completion.
- [ ] SDK API/examples and browser contract documentation targets, residual domain owners and
      downstream migration instructions are identified before closing this clarification Spike.

## Required Reads

- [ADR-063](../../decisions/063-bounded-runtime-shutdown-finalization.md) and Issue #520 full body.
- [WEB-007](WEB-007-optional-managed-browser-tool-core.md) / #452.
- [BROWSER-001](BROWSER-001-read-only-browser-provider-connector.md) / #508.
- [WEB-005](WEB-005-browser-session-continuity-research.md) and
  [TOOL-014](TOOL-014-conditional-tool-backends.md).
- [ADR-072](../../decisions/072-capability-provider-bundle-boundary.md), plus owners and merged
  evidence for Issues #45/#49 when investigating Request A.

## Intake Checkpoint

2026-09-09: registered only. Neither request has been accepted as an implementation promise.
Owner-first backlog and Issue matrix synchronization tracks this separately from I253/#519.
