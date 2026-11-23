# ADR-107: Bedrock ConverseStream transport and credentials

**Status**: Accepted and implemented in #1009 for #985 / I332

Use the regional Bedrock Runtime ConverseStream endpoint. Encode modelId as a URI path parameter. Sign the final method, path, headers and payload for the bedrock service and selected region. Read an explicitly selected AWS environment credential pair at dispatch, including the session token for temporary credentials. Disable redirects. Bind credential use to the selected regional service endpoint. Use isolated injected transport for fixtures. General profile, instance-role discovery and credential refresh remain with #132. Prefer maintained Rust signing and event-stream primitives subject to dependency review.

Consume AWS binary event-stream messages with bounded frame lengths and verified checksums before exposing events. Correlate tool blocks by index and toolUseId. Require explicit stop, account for trailing metadata usage and surface service exceptions. Preserve signed reasoning only under a verified compatible conversation binding. Establish exact replay requirements from official documentation before implementation. Audit streaming and Converse support per model. Explicit protocol choices take priority. Unsupported native-only models require an explicit documented route or refusal.

Primary references: https://docs.aws.amazon.com/bedrock/latest/APIReference/API_runtime_ConverseStream.html and https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_sigv-create-signed-request.html .
