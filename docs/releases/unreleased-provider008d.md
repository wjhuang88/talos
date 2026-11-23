# Unreleased — Amazon Bedrock ConverseStream

Existing explicit provider protocol settings retain precedence. To use the
regional Bedrock Runtime transport, select `amazon-bedrock`, configure the
model ID, and supply the AWS region and environment credentials described in
the [protocol reference](../reference/PROVIDER-PROTOCOLS.md#amazon-bedrock-conversestream).
Remove custom endpoint overrides when opting into this transport.

SDK callers using the configured factory can supply an empty API-key value.
The adapter reads AWS credentials at dispatch. Use the Agent replay path for
signed assistant reasoning. Keep the live session when continuing a signed
conversation. See the [SDK contract](../reference/RUNTIME-SDK-CONTRACT.md).

With a configured `[providers.amazon-bedrock]` table and no model or provider
endpoint override, three exact catalog IDs now default to ConverseStream:
`amazon.nova-micro-v1:0`, `amazon.nova-lite-v1:0` and `amazon.nova-pro-v1:0`.
An explicit `openai-chat` setting takes precedence when using a compatible
chat endpoint. Exhaustive SDK matches should add a fallback. This public
protocol addition targets a pre-1.0 minor release.
