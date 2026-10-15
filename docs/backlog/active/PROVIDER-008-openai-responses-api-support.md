# PROVIDER-008: OpenAI Responses API Support And Protocol Mode Configuration

**Status**: Refinement / Unclaimed
**Type**: Provider / Config / Model-Catalog API Story (Epic-shaped; child Stories decompose at claim time)
**Parent Epic**: None
**Source Issue**: [#724](https://github.com/wjhuang88/talos/issues/724)
**Selected Iteration**: None

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #724 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-10-15 |
| Handoff / Release Condition | Intake only: no claim, no activation, no implementation authority. Claiming follows AGENT-COLLABORATION preflight and the accepted ADR-093. |

## Identity / Goal / Value

Users of the official OpenAI provider and of catalogs that mark Responses support need Talos to
speak the OpenAI Responses API (`POST /v1/responses`) — including reasoning summaries, tool calls
and image input — and to pick the wire protocol automatically when support is confirmed, while
remaining fully configurable and backward compatible for `openai-chat` and gateways.

Confirmed maintainer decision (2026-10-15, Issue #724): **default to Responses when support is
confirmed; fall back to `openai-chat` when it is not; explicit configuration always wins.**

## Scope

Per [#724](https://github.com/wjhuang88/talos/issues/724) (authoritative detail; this owner
summarizes and will be decomposed into child Stories at claim time):

- New protocol value `openai-responses` through config, catalog and custom-model matching.
- Provider adapter: stateless Responses request mapping, typed stream-event parsing, encrypted
  reasoning replay, all `LanguageModel` paths, retry/progress parity (ADR-062).
- Catalog: consume models.dev `shape`/`npm` signals; confirmed-support default per the decision.
- Configuration: provider-level and model-level protocol with
  configured / catalog-inferred / default provenance visible to users.
- Documentation and release notes covering the default switch and the public-enum migration.

## Exclusions

See #724 non-goals: no server-side state (`previous_response_id`/`store`), no Responses built-in
tools, no Realtime/Assistants/Batch, no active probing (MODEL-011), no silent switch for
unconfirmed openai-compatible gateways.

## Dependencies / Decision Links

Requires a new ADR-093 (Responses protocol boundary, confirmed-support default, public-enum
`#[non_exhaustive]` migration, catalog signal mapping, reasoning replay policy) superseding the
ADR-034 deferral. Interacts with ADR-013/021/022/023/045/048/050/051/062/075, MODEL-011,
MODEL-014 (branch-only), PROVIDER-004; the claiming PR must re-verify overlap.

## Acceptance (summary; Given/When/Then detail in #724)

- Confirmed-support default: built-in `openai` without explicit protocol uses Responses and is
  revertible via explicit `protocol = "openai-chat"`; unconfirmed gateways stay on `openai-chat`.
- Full-turn correctness on `/responses` for text, tool calls, images and reasoning summaries.
- Old configs with explicit protocols behave identically; protocol provenance is visible.
- No-crash on unknown stream events; bounded encrypted-reasoning handling with no disclosure.

## Planned Validation / Documentation

Locked workspace tests and proptest for parsing; recorded-fixture stream tests; config precedence
matrix tests; catalog regeneration diff reviewed in its own PR; `release_preflight.sh`; README en/zh,
SDK contract, ARCHITECTURE, release notes.

## Residual Destination

Remaining protocol gaps (Responses built-in tools, server-side state, Realtime) stay out of scope
and return to the backlog.

## Required Reads

- Issue #724 (full implementation steps, risks, acceptance).
- `docs/decisions/034-reasoning-thinking-boundary.md`, `013-provider-config-schema-boundary.md`,
  `021-tool-call-protocol-architecture.md`, `062-typed-provider-retry-progress.md`.
- `crates/talos-provider/src/openai*.rs`, `crates/talos-config/src/{types,endpoint,config,model}.rs`,
  `crates/talos-config/build.rs`, `crates/talos-cli/src/provider_setup.rs`.

## Intake Checkpoint — 2026-10-15

Registered from maintainer request with #724 as the external requirement record. No claim, no
iteration, no implementation. Claiming requires: accepted ADR-093, child-Story decomposition,
preflight inventory, and an atomic claim+activation PR per AGENT-COLLABORATION.
