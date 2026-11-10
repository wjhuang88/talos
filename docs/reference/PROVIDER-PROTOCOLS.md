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

Use a Copilot-compatible credential authorized for the selected model. The packaged provider advertises
`GITHUB_TOKEN` to connection setup; a manual `[providers.github-copilot]` entry must explicitly set
`api_key_env = "GITHUB_TOKEN"` or, for example,
`api_key_env = "COPILOT_GITHUB_TOKEN"`. Talos does not perform OAuth login, token exchange, refresh
or enterprise endpoint discovery. Account scopes, subscriptions, integration quotas and model
allowlists remain server-side decisions. No real credentials or live vendor probe are used in tests.
See [ADR-093 section 9](../decisions/093-openai-responses-protocol-boundary.md) for the pinned contract.

### Explicit Gemini reasoning controls (implementation candidate)

The Gemini adapter in PROVIDER-008-B uses an explicit AI Studio or Vertex transport profile;
its Draft #838 is not yet fully accepted. Reasoning options never activate Gemini by themselves.
For an explicitly selected Gemini provider, configure model-local options as follows:

```toml
[providers.google]
protocol = "google-gemini"
api_key_env = "GEMINI_API_KEY"

[providers.google.gemini]
transport = "ai-studio"

[providers.google.models."gemini-2.5-flash".reasoning]
budget_tokens = 1024
```

Confirmed budget models are `gemini-2.5-pro` (128–32768), `gemini-2.5-flash` (0–24576)
and `gemini-2.5-flash-lite` (0 or 512–24576). Omit the budget for the vendor default;
zero disables thinking only on the admitted Flash/Lite models. `gemini-3.8-flash` and
`gemini-4-argon` accept `effort = "low"`, `"medium"`, or `"high"` instead of a budget.
An empty Argon options block uses its documented medium level. Enabled options request thought
summaries; the existing output limit still caps total generated tokens including thinking.

Unsupported model/setting combinations, both fields together and `replay = false` fail before
generation dispatch. Gemini native tool calls may require positional signatures, so the adapter
refuses a request to disable their replay instead of silently corrupting tool history. Isolated
permission decisions use independent minimal budget/level controls and suppress summaries and
reasoning events; they do not inherit conversation settings. Explicit reasoning_supported=false
continues to suppress conversation controls, but cannot silently ignore replay=false.
These exact model mappings do not infer preview, gateway or future model support, and do not
activate catalog entries. See ADR-094 for the primary source and current acceptance boundary.


## Explicit Gemini transports (unreleased implementation candidate)

Draft #838 / I310 adds `google-gemini` to the adapters listed above. Full B acceptance and
catalog activation are pending. Selecting a model named Gemini does not enable this candidate;
configure both the wire and its authentication profile explicitly in `~/.talos/config.toml`.

AI Studio uses the configured API-key source and the official generateContent destination:

```toml
provider = "google"
model = "gemini-2.5-flash"

[providers.google]
protocol = "google-gemini"
api_key_env = "GEMINI_API_KEY"

[providers.google.gemini]
transport = "ai-studio"
```

Set `GEMINI_API_KEY` in your environment before starting Talos. Standard Vertex uses OAuth
ADC instead of an API key; it requires an explicit project and location:

```toml
provider = "vertex"
model = "gemini-2.5-flash"

[providers.vertex]
protocol = "google-gemini"

[providers.vertex.gemini]
transport = "vertex"
project = "your-project-id"
location = "global"
allow_metadata = false
```

For Vertex, set `GOOGLE_APPLICATION_CREDENTIALS` to a local `service_account` or `authorized_user`
JSON file. Without that variable, discovery uses `$HOME/.config/gcloud/application_default_credentials.json`
on Unix or `%APPDATA%/gcloud/application_default_credentials.json` on Windows. An invalid explicit
file fails; it does not fall through to another account. File discovery is delayed until the first
request. Token acquisition/refresh and cache locking share the request deadline; cancelling the
request cancels its token exchange. The local synchronous service-account signer cannot be
preempted; its input bounds and integration limits are recorded in ADR-094.

Metadata authentication is disabled by default. `allow_metadata = true` permits the fixed metadata
service only when no local ADC file exists. Environment metadata-host overrides, external-account
executables, impersonation and arbitrary token destinations are not supported. Talos does not run
`gcloud` or perform an interactive login. ADC is never used for AI Studio or a replacement gateway.
Do not set a custom provider/model `base_url` for Vertex; it is rejected before credential discovery.

The candidate supports ordered images, native tool results and positional signature replay.
Ordinary tools also support explicit `compat` and `talos-strict` modes over the same Gemini wire.
These modes omit native function declarations, validate textual calls before releasing any tool,
and retain signed model text unchanged on continuation. A signed native-tool turn cannot be
rewritten as textual-tool history; start a new conversation to change that mode. Request-local
images remain request-local in both textual modes. `auto` must be resolved before dispatch.
Browser invocation is still refused in every mode: original argument syntax alone is insufficient
to make signed continuation safe for durable history. Reasoning controls and their model
boundaries are described above. Failures never trigger an automatic retry over a different wire.
There is no live-vendor success claim or packaged Gemini catalog activation in this candidate.

`ProviderConfig` gains optional `gemini: Option<GeminiTransportConfig>`; downstream exhaustive
struct literals must supply `gemini: None` or use `..Default::default()`. The protocol and catalog
enums gain `GoogleGemini`; keep catch-all match arms. These public changes require the next
pre-1.0 minor migration, together with ADR-094, rather than a patch release.
