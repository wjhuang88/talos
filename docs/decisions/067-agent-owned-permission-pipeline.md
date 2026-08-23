# ADR-067: Agent-Owned Permission Pipeline And Migration Contract

- Status: Proposed
- Date: 2026-08-23
- Owners: PERM-006-C / I220; implementation reserved for I221
- Related: Issues #55 and #59; ADR-064; ADR-065; ADR-066

## Context

PERM-006-A and PERM-006-B established structured reports and first-class scoped grants, but the
repository still has several composition roots that perform permission evaluation, approval and
execution gating independently. The CLI wrappers, TUI bridge, embedded Runtime adapter and MCP
gate do not share one final-decision contract. ADR-065 therefore deferred hook transport and
version migration to this decision.

The existing split creates four concrete risks:

- an approval can be issued for a projected value while execution receives a later mutation;
- a second evaluator or wrapper can disagree with the evaluator that produced the approval;
- `AfterPermissionCheck` can observe a compatibility decision rather than the decision that gates
  execution;
- a closed, timed-out or concurrently changed Session can admit a stale approval.

The complete current-path inventory is recorded in
[`PERM-006-C-CURRENT-PATH-MIGRATION-MATRIX.md`](../reference/PERM-006-C-CURRENT-PATH-MIGRATION-MATRIX.md).

## Decision

### 1. One Agent-owned orchestration authority

I221 will introduce one Agent-owned controller for a tool invocation. Surface composition roots
may provide only an adapter containing the Session state, explicit context, optional bounded
approval resolver and trusted grant source. They must not evaluate policy, compile grants, issue
authorization or execute a policy-bearing wrapper independently.

The controller owns normalization, evaluation, `Ask` resolution, grant admission, final hook
dispatch and the authorization passed to execution. A compatibility adapter may remain only when
it is policy-free and its removal is tracked by the migration stage.

### 2. One authoritative normalized request

Permission-relevant input is normalized and validated before evaluation or approval. The exact
normalized value, tool identity, provenance and permission profile form the authoritative request.
The resolver sees a safe summary or preview; it never supplies a replacement request. After
approval, mutation of any permission-relevant field invalidates the pending authorization and
must restart evaluation rather than silently reusing approval.

### 3. Bounded resolver authority

An `ApprovalResolver` is a surface adapter with one operation: resolve an already-evaluated
`Ask`. It may return `Once`, `Session` or `Deny`. It cannot evaluate policy, compile a grant,
change a rule, issue execution authorization or execute a tool. `Deny` remains dominant over any
resolver result. `AlwaysApprove` is retained only as a compatibility label mapped to the bounded
Session scope; it is not a general bypass.

The resolver runs outside the Session lock. Its result is committed only if the proposal identity,
permission revision, Session lifecycle and normalized-request digest still match. A mismatch,
closed channel, timeout, cancellation, resolver error or poisoned state fails closed. Each
invocation has one caller-provided total deadline; evaluation, resolver wait, admission and final
hook dispatch consume that same deadline. The resolver receives only the remaining budget, may not
reset or extend it, and cancellation propagates through every stage.

### 4. Final execution gate and hook semantics

The canonical sequence is proposal hook, pre-check hook, one evaluation, optional resolver,
revision-CAS admission, then one `AfterPermissionCheck` dispatch carrying the final decision that
will gate execution. Execution starts only after that hook permits the admitted authorization.

`AfterPermissionCheck` is not an advisory observation and cannot be dispatched before approval
resolution. Existing hook consumers receive a compatibility projection until a separately reviewed
additive/versioned transport exposes the full structured report. No secret, raw input, concrete
resource path or resolver token may enter a hook projection or log.

### 5. Surface contracts

- CLI print/headless and standalone MCP deny unresolved `Ask` when no resolver is available.
- TUI supplies a resolver adapter backed by the existing shared Session transition state.
- Embedded Runtime preserves the additive `ApprovalHandler` builder API and adapts it to the
  bounded resolver; its safe no-handler default remains denial.
- Inline/RPC paths use the same Agent controller and may not retain a second evaluator.
- Sandbox fallback remains a separate bounded authority. Permission `Deny` always wins; fallback
  approval cannot grant ordinary permission or become a permanent broad allow.

### 6. Additive compatibility and migration

The first implementation stage adds the controller/resolver path and adapters without changing
serialized permission configuration or durable Session data. Existing Runtime constructors and
`ApprovalHandler` remain source-compatible. MCP receives an additive entry point before any old
gate is removed. Public breaking removal or enum/constructor changes require a future versioned
migration and release decision.

The staged migration and rollback contract is normative in the matrix. Every stage must carry its
own exact changed-file inventory, tests, permission/security/API review and merge-time CAS.

## Rejected Alternatives

### Keep surface evaluators and synchronize them by convention

Rejected: convention cannot prove one-evaluation or final-hook invariants and permits drift between
approval and execution.

### Let the resolver return a modified tool request

Rejected: it would make the approval surface an authority and permit approval of one value followed
by execution of another.

### Dispatch `AfterPermissionCheck` before resolving `Ask`

Rejected: observers would see a provisional compatibility result rather than the final execution
gate.

### Make `AlwaysApprove` a global bypass

Rejected: it would bypass scoped grants, Deny dominance and human review boundaries.

## Consequences

- Permission orchestration has one auditable authority and one final execution gate.
- Existing adapters require staged migration and temporary policy-free compatibility wrappers.
- Runtime and MCP public APIs gain additive bridges before any source-breaking cleanup.
- Closed, stale, cancelled and timed-out approvals deny by construction.
- I221 must provide cross-surface tests and prove no alternate evaluator remains.

## Acceptance And Validation

Acceptance requires an independent permission/security/API review of this ADR and matrix, both
governance validators, YAML/diff/EOF checks, and exact-head CI. Acceptance authorizes only the
decision and I221 implementation boundary; it does not activate I221, change permission behavior,
modify Runtime/MCP code, implement TOOL-024, alter `/auto`, release or publish crates.

## Reversal Triggers

Revisit this ADR if a later design proves one-evaluation/final-hook invariants with a simpler
authority boundary, if additive compatibility cannot be maintained for a supported public API, or
if a security review finds that the resolver or hook transport can widen authorization.
