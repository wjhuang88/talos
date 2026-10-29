# Unreleased: protocol architecture and OpenAI Responses

**Status**: Unreleased source-development notes; no tag or release authorization.
**Scope**: PROVIDER-008-A / I306 / #748, child of #724.

## Behavior

- Add the dedicated stateless OpenAI Responses adapter, including text, ordered images, native
  function calls/results, reasoning summaries, encrypted reasoning replay, usage and terminal errors.
- Resolve model/provider explicit protocol before reviewed catalog signals and conservative Chat
  fallback. Display protocol provenance; never retry across wire families.
- Allow explicit model endpoint configuration and reviewed paired catalog endpoint/protocol metadata.
  The frozen catalog has 6,474 models, 4,062 protocol hints and 52 reviewed HTTP endpoint pairs.
- Support the reviewed public Copilot static Bearer profile through existing Chat, Responses and
  Messages adapters; seven catalog models select Messages. OAuth, token exchange/refresh and
  enterprise endpoints are not implemented. Vendor account/model authorization remains server-side.

## Required migration

This must ship in a pre-1.0 **minor** release, not a patch. `ProviderProtocol` and
`CatalogProviderProtocol` add `OpenAIResponses` and become `#[non_exhaustive]`; downstream exhaustive
matches need a catch-all arm. `ModelConfig` adds optional `base_url`; struct literals need
`base_url: None` or `..Default::default()`.

Old explicit protocols retain their behavior. Unset protocols on confirmed-support models can now
select Responses or their reviewed model wire. Set `protocol = "openai-chat"` at the relevant
model/provider scope to opt out; a model override takes precedence. Changed gateway roots suppress
catalog inference and credential relocation. The Copilot saved `/v1` provider root retains the
existing exact-root inference policy; use the packaged bare origin or an explicit protocol.

See [configuration and rollback examples](../reference/PROVIDER-PROTOCOLS.md) and
[ADR-093](../decisions/093-openai-responses-protocol-boundary.md).

## Validation and remaining scope

Implementation PRs #749, #775, #792, #793, #799 and #803 passed their recorded exact-head native CI
and independent Agent technical reviews. Tests include mock HTTP Agent tool/image rounds,
configuration/CLI routing, property tests and provenance-documented upstream SDK recordings.
No live credential/vendor test is claimed; local environmental failures remain recorded in I306.
Gemini, Azure and Bedrock remain unimplemented sibling tasks under open #724. Built-in provider
tools, server-side conversations and Realtime/Assistants/Batch remain the original exclusions.
