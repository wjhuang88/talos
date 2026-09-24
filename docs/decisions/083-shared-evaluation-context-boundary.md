# ADR-083: Shared Evaluation Context And Delivery Boundary

## Status

Accepted (2026-09-24). The maintainer accepted this boundary before implementation. It does not
authorize a new durable Mission/Evaluation schema or automatic evaluation.

## Context

The existing core contracts provide revision-bound `CompletionClaim`, independent evaluator and
`MissionGate` types. They intentionally do not create claims, identify the current Mission/Goal,
collect authoritative evidence or persist evaluation state. Desktop currently has no production
source for those values. Treating final model text, Todo status or a Desktop-generated UUID as a
verdict would let the executor certify itself and would make stale results appear current.

I284 needs an explicit evaluation action and an honest pass/fail/inconclusive/stale/current and
Delivery presentation, while the four-week baseline excludes a new durable evaluation store.

## Decision

When accepted, the shared Runtime owns the evaluation-context lifecycle and state transitions;
CLI, TUI and Desktop are clients that trigger an evaluation and render its projection. The UI
does not define completion, create authority identities, or infer Delivery eligibility.

1. Evaluation targets an explicitly identified, versioned Mission/Goal and immutable acceptance
   criteria. Criteria changes require an authorized new revision.
2. The executor may submit a completion claim and evidence references, but a claim is never a
   verdict. An independent evaluator receives a fresh bounded context and the shared gate alone
   determines Delivery eligibility.
3. Deterministic code validates identity, criterion coverage, evidence provenance/integrity,
   revision equality, cancellation, deadlines and stale results. The model may assess evidence;
   free-form model text cannot directly change Work state.
4. The evaluator has no executor-session write, execute or network authority by default. Missing,
   failed, inconclusive, stale, cancelled or unavailable input is non-PASS and is visible as its
   own state.
5. Evaluation is explicit in this iteration. The same shared operation may later be invoked by an
   automatic mode without creating a second policy or state machine.
6. The initial implementation may keep context and results ephemeral when no supported durable
   store exists. After restart, an unavailable result must not resurrect an old PASS. A future
   durable adapter may persist the same contracts without changing the authority boundary.

## Not Decided By This ADR

- the workspace revision algorithm (Git plus dirty content, a bounded artifact revision, or
  another independently validated representation);
- the durable storage adapter and retention policy;
- evaluator model selection, routing and whether a second provider is required.

These choices must preserve exact subject binding, bounded cost, cancellation and fail-closed
Delivery behavior. They require implementation evidence before being made normative.

## Consequences

Shared Runtime gains a reusable evaluation entry point and UI-neutral projection. Desktop avoids a
parallel business state machine and can display unavailable honestly. A public API/migration plan
is required if the context crosses the existing Runtime facade. Until an authoritative producer
exists, I284 remains incomplete; a fixture can test the state machine but cannot claim live
production evaluation.

## Validation Plan

- construct a real session-bound context without inventing authority identities;
- exercise pass, fail, inconclusive and stale revision transitions through `MissionGate`;
- verify cancellation/deadline and no tool admission in the evaluator;
- verify Desktop renders unavailable after restart when no durable adapter is configured;
- obtain independent security/API review before marking the public boundary accepted.

## Scope

This ADR concerns shared evaluation authority only. It does not authorize new Mission/Evaluation
schema, automatic evaluation after every turn, autonomous Mission planning, or a full artifact
editor/merge UI.
