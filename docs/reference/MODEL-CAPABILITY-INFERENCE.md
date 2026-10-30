# Custom model capability inference (I311, unreleased)

Implementation status and outstanding delivery gates are owned by
[I311](../iterations/I311-model014-capability-inference.md). This describes the
source candidate; it does not claim availability in an existing release.

## Resolution and overrides

Custom models use one unique catalog match for context, output, image, reasoning
and variants. Exact IDs take priority; otherwise one provider prefix may be
removed for a unique match. Ambiguous exact IDs remain unknown and do not fall
through to a normalized match. No fuzzy matching or endpoint probing occurs.
Each field independently uses explicit configuration, then matched metadata,
then unknown. An explicit context override does not disable output inference.

```toml
[providers.my-gateway.models."proxy/gpt-4o"]
image_input = false
reasoning_supported = false
output_limit = 4096
```

These false values override positive catalog evidence. Omitting a field permits
inference; derived values are not written into saved configuration. Unknown
image capability refuses attachment. Inferred image support still requires the
normal path permission, supported protocol and bounded image validation.

Reasoning false suppresses request effort, thinking budget and history replay,
while retaining raw history and tool call/result pairing. When capability is
unknown, legacy explicitly configured reasoning options remain usable; this is
not endpoint-support evidence. A selected supported variant overrides effort,
while preserving explicit budget and replay options unless reasoning is disabled.
An incompatible GLM-5.3 Auto review requiring thinking is refused before dispatch
when reasoning is disabled, rather than overriding the configured authority.

Chat sends `reasoning_effort` and `max_completion_tokens`; Responses sends
`reasoning.effort` and `max_output_tokens`; Messages sends a thinking budget and
`max_tokens`. Protocol-inapplicable effort/budget settings produce a runtime
materialization diagnostic. Explicit output ceilings override inferred ceilings;
existing protocol fallbacks remain when neither exists. Catalog limits are not
verified endpoint hard limits. Context and output limits remain independent.

The custom-model picker labels fields configured, catalog inferred or unknown,
and identifies the matched catalog record as endpoint unverified. No inference
selects a different wire protocol, performs paid requests or probes an endpoint.

## Rust configuration migration

`talos_config::ModelConfig` adds `reasoning_supported: Option<bool>`. Existing
TOML loads unchanged. Exhaustive Rust struct literals must add
`reasoning_supported: None`, or use `..Default::default()` when appropriate.
`None` permits inference; `Some(false)` explicitly disables reasoning.

This public literal change requires the next pre-1.0 minor release under
[ADR-095](../decisions/095-model014-catalog-capability-authority.md); it must not
ship as a patch release. No version/tag is created by this implementation.
Consumers constructing providers should materialize the selected variant through
`talos_config::variant::materialize_runtime_model_config`, surface its diagnostic,
and use `build_configured_provider` for effective capability and output policy.
Direct adapter constructors do not have access to a Talos configuration catalog.
