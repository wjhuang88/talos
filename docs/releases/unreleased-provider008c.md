# Unreleased — PROVIDER-008-C Azure implementation candidate

I323 / #914 adds explicit Azure OpenAI transport profiles over the existing Chat and Responses
message builders and streaming parsers. V1 supports both bodies; explicitly versioned deployment
paths support Chat. Model-local body selection overrides the profile default, with conservative
Chat fallback. Deployment aliases do not replace logical model identity.

API-key and supplied Entra-token profiles use mutually exclusive headers. Tokens are read from the
specified environment variable only at dispatch; missing or invalid values fail without API-key
fallback. Login, token minting, managed identity and refresh remain #132. Redirects and model
endpoint overrides cannot relocate credentials. Azure error details are redacted and opaque
Responses reasoning continuation is bound to its resource/deployment identity.

Public config additions require the next pre-1.0 minor version and downstream literal/enum review;
see RUNTIME-SDK-CONTRACT. No version or tag is published by this change. The candidate still needs
full local preflight, exact-head CI and final independent review. Controlled HTTP tests are not live
Azure validation. Bedrock and parent #724 completion remain separate pending work.
