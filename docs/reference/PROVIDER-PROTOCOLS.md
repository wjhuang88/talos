# Provider Wire Protocols

Talos selects the wire protocol separately from the provider identity and the tool protocol.
The available wire adapters are `openai-chat`, `openai-responses` and `anthropic-messages`.

## Selection And Provenance

The runtime factory, configuration queries and model-selection previews share one resolver:

| Priority | Evidence | Displayed source |
|---|---|---|
| 1 | Explicit model `protocol` | Configured |
| 2 | Explicit provider `protocol` | Configured |
| 3 | Confirmed model-level protocol in the packaged catalog | CatalogInferred |
| 4 | Confirmed provider default | Default |
| 5 | Conservative `openai-chat` fallback | Default |

Omission differs from an explicit `openai-chat`. Connecting a provider, entering a credential or
selecting a model must preserve omission. The internal absence marker is neither a configuration
value nor an option in the provider wizard. Old files containing an explicit Chat choice retain it.

The official `openai` provider uses Responses when its protocol is omitted and its endpoint is
official. An unconfirmed gateway defaults to Chat. Model catalog evidence requires a unique match;
ambiguous matches inherit nothing. No network probing is performed. The model picker, catalog
browser and `TALOS_MODEL.md` context show the effective protocol and provenance; the factory emits
the same resolution in debug tracing.

Normal builds consume the committed catalog without network access. New models.dev protocol
metadata is produced only during explicit regeneration, whose generated diff needs separate review.
Adding the mapper does not add signals to an older packaged catalog; generated data is reviewed
separately. Overrides of a built-in provider to an unconfirmed endpoint suppress its packaged
protocol evidence; explicit protocol choices retain precedence. Distinct model endpoints and
unimplemented host transports remain gated.

## Configuration

```toml
provider = "openai"
model = "o3"

[providers.openai]
api_key_env = "OPENAI_API_KEY"
# Omitted protocol selects the confirmed official default.

[providers.openai.models.o3]
protocol = "openai-responses"
```

Provider and model overrides can also be edited through the CLI:

```bash
talos config set providers.openai.protocol openai-chat
talos config set providers.openai.models.o3.protocol openai-responses
talos config get providers.openai.models.o3.protocol
```

The model override wins over the provider override. To roll that model back to Chat, set
`providers.openai.models.o3.protocol` to `openai-chat`. To return to inferred defaults, remove the
corresponding `protocol` keys from TOML; `config unset` retains its existing provider/credential-only
scope. The `/connect` custom-provider wizard offers all three adapters and defaults to Chat.

## Responses Contract

The dedicated adapter sends `POST /v1/responses` against the default OpenAI API root, or appends
`/responses` to the configured API root. Requests use `stream: true`, `store: false`, ordered
`input` items, flat function tools and `call_id`-bound function results. No server conversation or
`previous_response_id` is used. Native tools, images, reasoning summaries, bounded opaque reasoning
replay, usage and explicit terminal outcomes use the existing Talos provider boundary.

Unknown events do not imply completion. EOF without a semantic terminal event is an error. Endpoint
rejection suggests an explicit Chat override; it never triggers a cross-protocol retry. Encrypted
reasoning remains opaque, is isolated by provider/model identity and is excluded from readable
request previews. Built-in Responses tools, server-side state, Gemini, Azure and Bedrock are outside
PROVIDER-008-A.

## Public API Migration

`ProviderProtocol` and its shared `CatalogProviderProtocol` mirror gain `OpenAIResponses` and become
`#[non_exhaustive]`. Downstream matches must
include a catch-all arm. This change requires the next pre-1.0 minor release under ADR-093; it must
not be shipped as a patch release. `Config::provider_protocol()` now reports the same effective
protocol as runtime construction; `resolve_provider_protocol()` also reports its source.

Implementation and acceptance tracking: [PROVIDER-008-A](../backlog/active/PROVIDER-008-A-openai-responses-protocol.md)
and [I306](../iterations/I306-provider008a-openai-responses.md).
