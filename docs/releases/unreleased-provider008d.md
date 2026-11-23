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
