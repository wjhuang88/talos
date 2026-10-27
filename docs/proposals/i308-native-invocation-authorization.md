# I308 ordinary native invocation Auto authorization

Status: Accepted by the maintainer on 2026-10-27 in this handoff session.
This authorizes implementation of the exact scope below; runtime behavior remains
unimplemented until the acceptance evidence is established.

Owner: [I308](../iterations/I308-builtin-auto-review.md), PERM-007-H / handoff T8.

## Decision requested

Extend model-assisted AllowOnce to suitable ordinary built-in shell/direct-process
and network invocations using one review of the frozen operation and its ordinary
transitive effects. This is the same authorization granularity as an explicitly
approved shell command, not a promise of filesystem or destination isolation.
The maintainer's accepted Pull-only decision does not itself accept this extension.

## Concrete boundary

- Only implementation-owned built-in instances qualify. A matching tool name,
  caller-supplied tool, plugin or external MCP tool cannot claim this route.
- Bind the exact normalized arguments, tool/configuration identity, actual cwd
  where relevant, current user intent, session and policy/mode generations to a
  fresh invocation and digest. Execute the retained invocation once; changed
  identity/configuration/input needs new admission. Do not reconstruct approved
  input from redacted presentation arguments.
- Review discloses that shell/process work may execute configured programs,
  modify files and access network destinations. Network work may use retained
  backend/proxy/redirect behavior and expose request data to those destinations.
  Known targets are useful evidence, not an exhaustive transitive-effect manifest.
  This authorization explicitly accepts that lack of per-target confinement.
- Existing deterministic Allow and terminal Deny remain authoritative. A model
  can resolve eligible Ask only. Explicit mandatory-human policy remains human.
  If an active finer-grained restriction could be bypassed by undiscovered effects
  and cannot be enforced by the execution route, reject this route; neither model
  nor human approval overrides Deny. Test this check in the actual pipeline.
  Run this compatibility guard before deterministic Allow as well as model/human
  Ask paths. Failure is terminal for the broad invocation route, not another
  prompt that a human may waive.
- Keep separate tool-free assessment, bounded complete payload/output, secret
  handling, deadline/panic/cancellation guards, freshness checks, one-use admission
  and visible decision/failure reports. Uncertain, sensitive, unsupported or stale
  requests retain actionable human/headless fallback, never a persistent grant.
- Preserve each tool's existing process supervision and transport/output limits.
  Approval does not create rollback for command or network effects; cancellation
  may leave already-started effects and must report that limit truthfully.

## Relationship to existing contracts

[ADR-064](../decisions/064-bounded-model-assisted-auto-permission.md) needs an
explicit amendment for ordinary effectful command/network eligibility and unknown
transitive effects. This proposal does not silently declare them implemented.

[ADR-090](../decisions/090-invocation-bound-search-endpoint-authority.md) and I293
retain ownership of protected search transport and per-destination authority.
An invocation requiring prepared-network admission must continue to use that path;
failure cannot downgrade to this broader route. This proposal would authorize the
currently ordinary built-in network route with its broader boundary explicit.
Eligibility requires positive trusted registration and an explicit owner
disposition, never merely absent prepared evidence. WebSearch and search
compatibility/default transport remain excluded until I293 owner concurrence and
a merged ADR-090 scope amendment explicitly permit coexistence. Pre-migration
status alone cannot make an I293-owned operation eligible.
It does not activate a backend, change search
routing, retire dependencies or claim SSRF confinement. I293 coordination and an
explicit ADR-090 amendment documenting this coexistence precede activation.

On 2026-10-27 the maintainer (the recorded responsible actor for both owners) confirmed that
ordinary HTTP may use whole-invocation review while WebSearch, search compatibility/default
transport and all required prepared-network tools retain their dedicated authority with no
downgrade on failure. This supplies owner concurrence, not activation. The corresponding
ADR-090 coexistence amendment still must merge before ordinary production activation.

Reuse reviewed shared assessment/admission primitives where their contracts fit.
Do not turn the Pull-specific proof into a generic proof by relabeling tools, or
create an exhaustive effect-prediction framework as a prerequisite.

## Acceptance before delivery

1. Actual ordinary shell/direct-process and network tool dispatch after matching
   high-confidence model AllowOnce, with actual temporary-file/local-server effects.
2. Human-only/Deny, incompatible fine restrictions, uncertain/error/panic/timeout,
   cancellation, changed arguments/configuration/identity/generations and replay
   cannot produce unauthorized execution. Multi-facet admission is atomic.
3. Protected network tools cannot downgrade to ordinary execution; external
   same-name replacements cannot inherit native trust. Existing read fast paths
   incur no extra assessment.
4. Unix and native Windows process evidence, applicable product profile evidence,
   visible decisions and documentation describing actual limits. Windows build
   uses cargo xwin; cross-compilation is not native runtime proof.
5. Independent permission/security/API review, compatible public API/migration
   evidence, locked preflight, exact-head CI and merge-time CAS.

Accepting this proposal is a scope decision only. I308 and T8 remain incomplete
until these implemented outcomes and the rest of their original acceptance pass.

## Alternative

Keep ordinary effectful requests on human approval; that changes the reviewer,
not their confinement. Separately implement endpoint/path confinement before
enabling model approval. The latter provides narrower authority but is additional
engineering work. Human approval alone does not deliver the requested
ordinary-operation Auto behavior and cannot be reported as completed T8.
