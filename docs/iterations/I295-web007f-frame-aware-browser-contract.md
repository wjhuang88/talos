# Iteration I295: Frame-Aware Browser Host Contract

> Document status: Active / Claimed; proposed atomic claim and activation are ineffective until PR #669 merges.
> Published plan date: 2026-10-06
> Planned objective: deliver an opt-in host-executed frame-aware browser v2 contract with exact admission, origin-bound permission and shared conformance evidence.
> Baseline rule: preserve this objective and acceptance; different outcomes need a new iteration.
> MVP deliverable: an embedded host can register a bound test executor, discover a child frame, authorize one exact action, and receive a bounded result; stale or unauthorized calls never mutate.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / gpt-6 |
| Work Slice | WEB-007-F / #618 only; frame-aware v2 public contract, prepared invocation, browser-specific permission integration and host conformance |
| Claimed At | 2026-10-06 |
| Source Issue | #618 |
| Governance Claim PR | #669 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer accepted the corrected v2 contract and explicit non-overlapping parallel scheduling on 2026-10-06; independent design review approved the uncommitted contract. Exact-head governance review and CI remain merge gates for PR #669. |
| Implementation PR | Not started |
| Last Updated | 2026-10-06 |
| Handoff / Release Condition | Claim and activation require PR #669 on main. Implementation starts only from its merge or later main; no overlap with I293/I294 owners. |

## Published Baseline

### Selected Story And Dependencies

WEB-007-F / #618 is the sole selected child of WEB-007 / #452. ADR-086 and the linked
frame-aware contract govern this slice. The parent V1 proposal remains unimplemented and is not
claimed Complete by I295. This slice may publish separately negotiated v2 without pretending
that a V1 executor or native plugin exists. Process carrier and native browser are separate
unclaimed delivery stories; their real-browser evidence is required before #520 Request B can
claim production-native fulfillment.

### Scope

- Exact 21-operation v2 request and bounded typed output contract, opaque frame/snapshot/element
  identity, canonical origins and session/epoch invalidation.
- Opt-in host `BrowserExecutor` and `ManagedBrowserTool`; prepared one-shot invocation path with
  browser-specific, invocation-only permission across Agent and embedded Runtime.
- Provider raw-argument integrity for v2; standalone MCP must reject v2 until equivalent raw
  ingress proof exists. Other tools keep their existing paths.
- Shared deterministic host conformance fixtures, including approval/navigation and
  post-dispatch races, child-origin isolation, projection secrecy and no mutation replay.
- External host example and user/API migration documentation.

### Non-Goals

No default registration, Chromium/CDP/native plugin, process carrier, account or profile policy,
credential handling, selectors/script/evaluate, WEB-005/TOOL-014 change, V1 delivery, release or
publication. I293/I294 search work and their owner files remain outside this Work Slice.

### Acceptance

- A host can explicitly compose v2, discover a frame, obtain an exact authorization and execute
  one current-document action with a bounded typed projection.
- Cross-origin child action cannot inherit a parent or generic Network grant. Invalid schema,
  duplicate keys, stale/foreign refs, expired/reused/swapped tickets and lost lifecycle state
  cause zero unauthorized executor calls and zero mutation.
- Approval-time navigation fails before dispatch; post-dispatch navigation fails before wrong-
  document mutation; transport ambiguity never triggers replay.
- Same-origin, cross-origin, nested, delayed, duplicate, rebuilt and detached frame fixtures plus
  all mandatory cases in the contract pass against the host executor path.
- V2 remains absent by default and unavailable on standalone MCP until raw-ingress proof exists;
  V1 definition and legacy tool behavior remain unchanged.

### Planned Validation

- Focused protocol, provider, permission, Agent, Runtime, MCP and host fixture tests with `--locked`.
- `./scripts/release_preflight.sh` (includes locked workspace checks/tests).
- Both governance validators and `git diff --check`.
- Exact-head CI and independent permission/security/API review; merge-time CAS.
- External host example exercised with a deterministic bound executor. Native browser evidence
  is explicitly not claimed.

### Documentation To Update

- Embedded Runtime SDK and tool composition usage/API migration guide; browser host example.
- WEB-007-F owner, parent WEB-007 dependency note, Product Backlog, iterations README and Board.

### Risks And Rollback

The principal risks are pre-permission parser bypass, broad legacy grant reuse, stale-document
mutation and leaking child content into parent projections. Fail closed per route/operation.
Rollback is removal of explicit v2 composition; legacy/default inventories remain unchanged.

## Iteration Inventory And Parallel Disposition

At selection on 2026-10-06: I293 and I294 are Active under separate SEARCH-001-C/D claims and
continue unchanged; the maintainer explicitly authorized non-overlapping I295 parallel work and
owner-first union updates to shared derived views. I277, I290 and I291 stay Review with their
existing acceptance gates. I249 stays Planned and deferred. I164 remains Paused. No owner,
acceptance or implementation authority from those items transfers to I295.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-10-06 | Proposed | PR #669 proposes atomic claim and activation; both are ineffective until its finalized exact head merges to main. |

## Verification Evidence

None yet for implementation. Design-only Agent-role review approved the corrected proposal on
an uncommitted diff over `9712d458`; it is not exact-head code review or runtime evidence.

## Completion Evidence

Pending. A future owner-first closeout must cite an already-existing implementation SHA.

## Variance And Residuals

Native iframe operation, V1 base contract and standalone MCP raw-ingress support remain separate
gates. The owner cannot claim them delivered from host fake-executor tests.
