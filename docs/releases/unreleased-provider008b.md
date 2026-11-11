# Unreleased — PROVIDER-008-B Gemini candidate

Draft PR #838 / I310 adds a stateless native Gemini wire with AI Studio API-key and standard
Vertex OAuth profiles, lazy user/service-account ADC, ordered images, native/textual tools,
usage, reasoning controls and identity-bound signed continuation. No cross-wire error fallback.

Browser tools work in Native, Compat and TalosStrict through the existing permission path.
Signed originals remain in live Actor custody; persisted projections cannot restore them after
reopen/fork. Custody limits are 32 blocks / 16 MiB. With browser tools registered, ordinary signed
textual tool frames also require that live custody. Auto must resolve before dispatch.

Only exact google and google-vertex gemini-2.5-flash catalog pairs with matching explicit
transport profiles and unchanged endpoints infer Gemini. Explicit model/provider protocols win.
Other catalog models and custom gateways retain existing routing. Model selection admits lazy
Vertex ADC without reading credentials and displays effective protocol/source.

Public API changes require the next pre-1.0 minor migration, including non-exhaustive
ProviderInvocationEvent and stream_with_replay_context. See ADR-094, PROVIDER-PROTOCOLS and
RUNTIME-SDK-CONTRACT. This candidate is not released or finally accepted. Mock HTTP and sourced
Google SDK SSE fixtures do not establish live Google service success. Azure, Bedrock and parent
#724 remain separately unfinished; #132 authentication scope is not closed by this work.
