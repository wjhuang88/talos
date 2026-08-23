# PERM-006-C Current-Path And Migration Matrix

Status: ADR-067 Accepted; I221 implementation is Active/Claimed through governance PR #375 and is
locally converging toward its first stable Review candidate.

This matrix records the current permission authorities and the target migration boundary. It is
normative together with ADR-067. It does not authorize code changes.

| Surface | Current entry point | Current evaluator / approval authority | Current hook boundary | I221 target | Compatibility / validation gate |
|---|---|---|---|---|---|
| CLI print / headless | `talos-cli` print registry and `PermissionAwareTool` | CLI wrapper evaluates `PermissionEngine`; `Ask` is denied when no interactive handler exists | Wrapper dispatches proposal and permission hooks around its own evaluation | Agent-owned pipeline receives a surface adapter; headless `Ask` fails closed; execution receives only the admitted authorization | Keep denial messages and non-interactive behavior; test no-handler, timeout and cancellation paths |
| CLI interactive / TUI | TUI registry, `TuiPermissionAwareTool`, `TuiApprovalHandler` | TUI wrapper evaluates and mutates shared `PermissionSessionState`; approval UI chooses Once/Session/Deny | Wrapper and TUI bridge currently observe proposal, pre-check and post-check events | TUI supplies an `ApprovalResolver` adapter; Agent performs evaluation, resolver admission and final hook dispatch exactly once | Preserve shared Session transition state and prompt semantics; test concurrent prompts and stale revision rejection |
| Inline / RPC tool calls | Agent tool execution and inline composition adapters | Each composition path may currently wrap tools or evaluate before execution | `talos-agent/src/tool_execution.rs` dispatches proposal, before-check and after-check hooks | One Agent controller owns normalization, evaluation, resolver, admission and execution authorization | Inventory every wrapper; no second evaluator may remain; test one-evaluation invariant |
| Embedded Runtime SDK | `RuntimePermissionAwareTool` in `talos-runtime` | Runtime adapter evaluates `PermissionSessionState`; `ApprovalHandler` resolves `Ask` and then admits | Runtime currently delegates hook handling through the Agent execution path while owning a permission wrapper | Runtime injects one Session, context, resolver and trusted grant source into the Agent controller | Preserve additive `ApprovalHandler` compatibility; existing builders remain source-compatible; test no-handler fail-closed |
| Standalone MCP | `McpPermissionGate::evaluate_call` | MCP gate directly evaluates `PermissionEngine`; headless `Ask` is denied | MCP dispatches `OnToolCallProposed`, `BeforePermissionCheck`, then `AfterPermissionCheck` before returning | MCP becomes a surface adapter into the Agent controller; final hook observes the admitted decision | Additive public API first; retain headless deny behavior; test hook denial, Ask, cancellation and redaction |
| Sandbox fallback | Runtime/CLI fallback handlers | Separate fallback policy, not ordinary permission approval | Fallback approval is distinct from normal permission hooks | Remains a separate bounded authority; `Deny` always dominates and fallback cannot grant normal permission | Requires independent security review; no I221 implicit policy expansion |

## Canonical Ordering

Every surface must converge to one ordered flow:

1. Capture the original request and immutable execution context.
2. Normalize and validate permission-relevant input once.
3. Dispatch proposal and pre-permission hooks against the normalized projection.
4. Evaluate policy and trusted grants exactly once.
5. If the result is `Ask`, call the bounded surface resolver outside the Session lock.
6. Commit the resolver result with proposal/revision compare-and-swap; stale or closed state
   fails closed.
7. Admit the exact authorization against the same normalized request.
8. Dispatch `AfterPermissionCheck` exactly once with the final Allow/Deny that gates execution.
9. Execute only after the final hook permits the admitted authorization.

No surface may evaluate a second time after approval, mutate permission-relevant input after
approval, or execute from a projected value that differs from the normalized authoritative value.
Every invocation uses one total caller deadline. The resolver receives only the remaining budget;
no stage may reset or extend it, and cancellation propagates through evaluation, approval, admission
and final-hook dispatch.

## Migration Stages

| Stage | Deliverable | Exit evidence |
|---|---|---|
| M0 | Additive Agent controller and resolver contracts; preserve existing adapters | Public API review, no behavior change outside explicitly covered paths |
| M1 | Route CLI print/TUI and embedded Runtime through the controller | One-evaluation tests, Session CAS tests, exact authorization tests |
| M2 | Route inline/RPC and MCP through the same controller | Cross-surface matrix, hook-order tests, headless fail-closed tests |
| M3 | Remove or reduce policy-bearing wrappers; retain policy-free compatibility adapters only | Changed-file inventory, API migration note, security review |
| M4 | Closeout and publish migration evidence | Exact-head CI, independent permission/security/API review, merge-time CAS |

Rollback at any stage is to keep the prior adapter active and reject the stage's claim; no
persistent permission data is rewritten and no approval is widened by rollback.

## 2026-08-23 I221 Implementation Facts

- Production CLI print, interactive, TUI, inline/RPC, Runtime and MCP composition roots register
  raw tools and supply Session/context/resolver inputs to the Agent-owned pipeline.
- `PermissionSessionState::begin_invocation` performs the single authoritative evaluation and
  emits either admitted policy authority or revision-bound Once/Session proposals. Approval and
  execution admission use request identity and revision CAS without a second policy evaluation.
- Agent and standalone MCP normalize and validate the exact input before the permission profile,
  preview, authorization and execution path; the admitted authorization is passed to
  `execute_authorized_with_output`.
- Proposal, pre-check and final permission hooks are strict gates: Deny, Skip, panic or timeout
  prevents execution. Hook/log projections are structure-only and contain no raw value or concrete
  resource path; Runtime and interactive resolvers retain the tool-defined safe presentation for
  compatibility, without authority to replace the normalized request.
- Resolver absence, denial, timeout, cancellation, channel closure, error, panic and stale revision
  fail closed. Every call site supplies one total deadline; Session lock contention uses a
  non-blocking fence and resolver wait receives only the remaining budget. The non-TUI terminal
  resolver delegates to the existing event-loop stdin reader, so cancellation closes the pending
  response without leaving a detached reader. The final `AfterPermissionCheck` projection is Deny
  for every failed pipeline result.
- Standalone MCP retains the existing `evaluate_call` signature and adds `authorize_call` for exact
  execution authorization; no old caller is forced onto a breaking API.
- Legacy policy-bearing wrappers remain compiled only as test compatibility fixtures; production
  builds cannot select them. Serialized permission config, Runtime `ApprovalHandler`, sandbox
  fallback and TUI `/attach` boundaries are unchanged.
