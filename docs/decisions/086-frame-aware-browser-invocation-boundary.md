# ADR-086: Frame-Aware Browser Invocation Boundary

## Status

Proposed for WEB-007-F / #618. Maintainer direction and independent Agent-role design review
support this boundary; it has no production or implementation acceptance effect until the
governance candidate is merged. I295 owns implementation evidence.

## Context

WEB-007 / #452 proposes an opt-in host-executed browser contract. Its V1 proposal is tab and
element oriented, so it cannot authorize child-frame actions or safely reuse top-frame grants.
Current tool execution evaluates a permission profile before execution admission, path-bound
authorization cannot express an origin-bound browser invocation, and some tool argument ingress
paths discard duplicate JSON keys before Tool admission.

## Decision

1. Negotiate `talos.browser.executor/v2` explicitly. V1's 20-operation definition is not
   silently widened. V2 has the closed 21-operation schema, opaque generation-bound references,
   frame-local outputs and budgets in the [WEB-007-F contract](../proposals/WEB-007-F-frame-aware-browser-contract.md).
2. A trusted bound host executor owns lifecycle, frame identity and origin evidence. Model
   arguments cannot assert authority. Exact typed admission and current-state checks precede
   permission evaluation and run again before dispatch. Stale or unavailable context fails closed.
3. An invocation-owned, one-shot prepared ticket binds the canonical request, resource, executor,
   epochs, nonce and expiry. The dedicated browser permission evaluator defaults to deny and
   permits only an exact browser-aware policy decision or per-invocation human approval. Generic
   ToolNature/tool-name, Network, path, session and workspace-trust Allow rules grant no browser
   authority. An approved request cannot be swapped, replayed or retargeted after navigation.
4. V2 ingress rejects duplicate JSON keys before conversion to `serde_json::Value` and performs
   the complete closed-schema and semantic check itself. An entry point lacking raw-argument
   integrity proof must not register or execute v2. In particular, standalone MCP remains v2
   unavailable until its raw-ingress boundary is proven; serializing a parsed Value is insufficient.
5. Bound executors receive one admitted call and must reject document changes before mutation or
   release of observation. No automatic retry, fallback or ambiguous mutation replay. Unsupported
   document-bound actions fail closed. Host and later native implementations must pass the same
   conformance fixtures; fake-executor evidence does not prove native iframe behavior.
6. Composition is explicit and unregistered by default. WEB-005 read-only ingestion and TOOL-014
   disclosure cannot mint or borrow interactive browser authority. No browser binary, credentials,
   profile or CDP transport enters the shared protocol boundary.

## Compatibility And Migration

Add a separate opt-in prepared invocation API without changing existing path authorization or
legacy AgentTool behavior. Agent, embedded Runtime and MCP composition roots must either support
the full browser path or reject v2 registration/execution. Public API consumers receive a
migration note and external host example before publication. The detailed operation schema,
limits, output projection and lifecycle cases remain in the linked contract and are not weakened
by this summary.

## Validation And Reversal

I295 must prove exact schema/typed admission parity, zero approval and executor calls for invalid
or stale requests, child-origin isolation, one-shot ticket behavior, lifecycle races, bounded
projection, opt-in composition and a host executor end-to-end path. Locked workspace validation,
exact-head CI and independent security/API review are required. Native-browser iframe behavior
remains a separate implementation evidence gate.

If an ingress cannot preserve raw arguments or a backend cannot bind actions to the selected
document, disable that v2 route or operation rather than relaxing validation or permissions.
A materially different public API, grant model or V1 compatibility decision requires an ADR
amendment or superseding decision before implementation.
