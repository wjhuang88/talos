# Iteration I306: Protocol Architecture And OpenAI Responses

> Document status: Planned
> Published plan date: 2026-10-22
> Planned objective: establish model-level wire-protocol resolution and deliver a complete OpenAI Responses adapter under ADR-093.
> Baseline rule: preserve this bounded child-A scope; Gemini, Azure or Bedrock use separate owners/iterations.
> MVP deliverable: one executable stateless `/v1/responses` agent round with text, images, native tools, reasoning summary, usage and protocol provenance.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | PROVIDER-008-A protocol architecture plus OpenAI Responses only; no Gemini/Azure/Bedrock implementation. |
| Claimed At | Not applicable |
| Source Issue | #724 |
| Governance Claim PR | Pending |
| Implementation PR | Not started |
| Authorization Mode | Not applicable |
| Authorization Evidence | Draft governance-only proposal; exact-head claim/activation is ineffective until merged to main. |
| Last Updated | 2026-10-22 |
| Handoff / Release Condition | Activate only with accepted ADR-093 and effective PROVIDER-008-A claim. Complete only with existing implementation commit evidence. |

## Published Baseline

Selected [PROVIDER-008-A](../backlog/active/PROVIDER-008-A-openai-responses-protocol.md), proposed
child of PROVIDER-008 / #724. ADR-093 is the governing protocol/public-API decision and must become
Accepted atomically with activation.

### Selection Inventory And Disposition

Selection main: `0caaf83241b8f94cba0cbdca66e3e166068f024d`.

- Issue #724 and all current comments re-read; parent PROVIDER-008 remains Refinement / Unclaimed.
- No open PR found for #724 / PROVIDER-008 / OpenAI Responses and no matching `724`/`provider`
  implementation branch found.
- I293 and I294 retain Search work; I300 retains structured Git scope; I302 retains Windows resource
  attribution Review; I305 is Complete/Closed. None owns provider wire-protocol expansion.
- PROVIDER-007/I297 is Complete/Closed and changes retry defaults only; it is not reopened.
- MODEL-014 is Refinement / Unclaimed and remains separate. I306 may add protocol metadata required
  by #724 but may not absorb general model capability/variant inference.
- Sibling PROVIDER-008-B/C/D are deliberately not claimed.

No different active/review/planned/blocked owner found with the same work slice. If target-branch
truth changes before merge, the merge-time CAS preflight must stop and reconcile rather than silently
steal scope.

### Scope

1. Accept ADR-093 and add `openai-responses` plus the public `#[non_exhaustive]` migration boundary.
2. Centralize protocol-to-adapter construction so future families do not multiply exhaustive factory
   matches across CLI/Desktop/discovery/model browser.
3. Add model-level protocol configuration, catalog hints and provenance using the precedence frozen
   in ADR-093.
4. Implement the dedicated stateless OpenAI Responses request and streaming adapter, including native
   tool calls/results, images, reasoning summaries/encrypted replay handling, usage and terminal/error
   semantics.
5. Expose effective protocol/source on the #724 model/debug surfaces and document the confirmed-
   support default plus explicit `openai-chat` rollback.

### Non-Goals

No Gemini, Azure, Bedrock, Responses built-in tools, Realtime, Assistants, Batch, server-side state,
MODEL-011 probing, MODEL-014 general capability inference, permission/storage redesign, unrelated
retry-policy change or new external dependency unless separately decided and claimed.

### Acceptance

Acceptance is the PROVIDER-008-A owner acceptance in full. In particular:

- confirmed official OpenAI can select Responses only when protocol is unset and support is known;
- explicit chat and unconfirmed compatible gateways stay chat;
- model-level confirmed signals outrank provider defaults and provenance is visible;
- complete `/responses` text/image/tool/reasoning/usage round works without server session state;
- unknown stream events never panic and non-terminal EOF is an error;
- protocol rejection is actionable and never triggers cross-protocol retry;
- old config behavior and serialization remain covered by tests.

### Planned Validation

Focused request/stream/config/catalog tests and proptest during local convergence; mock HTTP full-turn
coverage including tool call and image continuation; then pinned-toolchain locked workspace tests and
`./scripts/release_preflight.sh`. Exact candidate CI and independent API/provider review are required
before merge. Catalog regeneration, if needed, remains a separately reviewable stage/diff rather than
being hidden inside an unrelated code change.

### Documentation / Migration

Update public provider/config documentation, README en/zh examples, model UI/debug provenance and
release migration guidance. The public enum becomes non-exhaustive in the same pre-1.0 minor release;
downstream exhaustive match users must add a catch-all arm.

### Risks / Rollback

The visible risk is default-path change for confirmed-support models. Rollback is explicit
`protocol = "openai-chat"`; never silently retry a second wire protocol after a request begins.
Encrypted reasoning is confidential opaque data and must not enter readable logs/debug surfaces.
Revert the Responses/default-selection stage if protocol correctness cannot be demonstrated rather
than weakening terminal/error or provenance requirements.

## Activation Record

2026-10-22: governance-only draft prepared. No production code, implementation tests, generated
catalog, dependency or config behavior changed. Claim/Active state, accepted ADR and implementation
authority remain pending until the finalized governance PR reaches main.

## Completion / Residuals

No Completion Commit exists. PROVIDER-008-B/C/D and every excluded Responses feature remain residual
under the parent epic. Do not mark I306 or PROVIDER-008-A Complete without an already-existing
implementation/evidence commit SHA.
