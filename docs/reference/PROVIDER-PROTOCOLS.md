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

Tool protocol remains a separate selection: explicit `compat` and `talos-strict` use text tool
instructions/results and the shared text parser over the Responses wire, while `native` sends
function schemas. Provider-level `tool_protocol` is retained when merging user configuration.
Tool-generated images (`read_image` or browser captures) use request-local `input_image` parts only
in the immediate next request. The shared image resource limits and earliest expiry apply; a
request containing these images is dispatched once and is not automatically retried. Durable user
attachments retain their existing history behavior.

## Model-Specific Endpoints

`providers.<provider>.models.<model>.base_url` sets an explicit model API root. It wins over the
provider connection and is independent from `protocol`. For an unconfirmed endpoint, select the
wire explicitly; a new model URL is not evidence that the official OpenAI Responses default applies.

```toml
[providers.gateway]
base_url = "https://gateway.example/v1"
api_key_env = "GATEWAY_API_KEY"

[providers.gateway.models.example-model]
protocol = "openai-responses"
base_url = "https://gateway.example/openai/v1"
```

The CLI supports `config set/get providers.gateway.models.example-model.base_url`; setting an empty
value removes the override. Model endpoints allow HTTPS or HTTP loopback, and `${ENV_VAR}` roots
must resolve before adapter construction. Unsupported transports fail before a request; Realtime
WebSocket support is not implemented by the Chat/Responses HTTP adapters.

Packaged model endpoint metadata is a reviewed connection default, paired with its protocol.
It applies only to the exact built-in provider/model, while the configured provider root still
matches that provider's catalog root and the effective protocol matches the model's catalog wire.
Selecting an explicitly different wire (including Chat rollback) suppresses that wire's endpoint.
Replacing the provider root also suppresses catalog routing. A custom provider may still inherit
protocol by a unique model match, but never inherits the catalog vendor's URL or credential destination.
A separately reviewed data projection is required to activate new packaged endpoint records; adding
routing code does not change the committed catalog. Cross-origin packaged defaults require the
same source/credential-destination review as provider defaults; no live models.dev input changes
connections during a conversation.

`Config::resolve_provider_endpoint()` returns the effective API root and connection source;
provider construction records the source without logging the endpoint or credentials.
`Config::base_url()` remains the provider-level compatibility accessor.

## Public API Migration

`ProviderProtocol` and its shared `CatalogProviderProtocol` mirror gain `OpenAIResponses` and become
`#[non_exhaustive]`. Downstream matches must
include a catch-all arm. This change requires the next pre-1.0 minor release under ADR-093; it must
not be shipped as a patch release. `Config::provider_protocol()` now reports the same effective
protocol as runtime construction; `resolve_provider_protocol()` also reports its source.

Implementation and acceptance tracking: [PROVIDER-008-A](../backlog/active/PROVIDER-008-A-openai-responses-protocol.md)
and [I306](../iterations/I306-provider008a-openai-responses.md).

`ModelConfig` gains optional `base_url`; downstream explicit struct literals should use
`base_url: None` or `..Default::default()` in the same pre-1.0 minor migration.


### Public GitHub Copilot static credentials

The shared factory recognizes only the reviewed public `https://api.githubcopilot.com` HTTPS
origin. Its existing Chat, Responses and Messages adapters use Bearer authentication there, with
truthful Talos attribution and no redirects. Messages uses `/v1/messages` and retains the Anthropic
version header; it never sends the GitHub credential as `x-api-key`. Other gateways retain their
own authentication. Wire overrides remain explicit and never cause a fallback retry.

Use a Copilot-compatible credential authorized for the selected model. The packaged provider retains
`GITHUB_TOKEN` lookup; an explicit `[providers.github-copilot]` entry may instead set
`api_key_env = "COPILOT_GITHUB_TOKEN"`. Talos does not perform OAuth login, token exchange, refresh
or enterprise endpoint discovery. Account scopes, subscriptions, integration quotas and model
allowlists remain server-side decisions. No real credentials or live vendor probe are used in tests.
See [ADR-093 section 9](../decisions/093-openai-responses-protocol-boundary.md) for the pinned contract.
