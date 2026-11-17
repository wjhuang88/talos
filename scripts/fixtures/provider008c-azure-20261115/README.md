# Azure source admission audit

Frozen https://github.com/anomalyco/models.dev commit
888c250133b38839c2018d66fe687cb03c03a8f2, acquired via Git on 2026-11-15 and recovered at the
same revision on 2026-11-16. HTTP api.json returned 403; this is source evidence, not generated API
output. `audit.json` contains both provider records and all 99 Azure model TOMLs as exact source,
SHA256 and parsed API metadata. MIT LICENSE is preserved. Fixture SHA256 is
9220ffddda1a970bab16eb3b546fc73cffc65d572d3083f0850abc6239b8fb80.

`translate.ts.txt` preserves packages/core/src/translate.ts (SHA256
c53a1b314008e47b098c5eb800c5b99505036a9bd5571d64730deea96dce5b7e).
Its toV1Api selects the first authored API; toV1Shape recognizes Azure's Responses default.
27 records have Responses first with no Responses SDK/base_url override; 22 intersect the packaged
catalog without routing overrides. The runtime admits only those exact azure/model pairs with a
valid explicit v1 resource profile and no provider/model endpoint replacement. Explicit model and
provider protocol/shape win. No endpoint, deployment or credential is inferred.

The other source records comprise 33 Chat-only, 16 Messages, 10 without direct API tables,
8 embeddings and 5 images. The legacy azure-cognitive-services entry is a v1 alias, not another
current model set; alias/custom providers, legacy deployment mode and special /models endpoints
are not admitted for automatic inference. No packaged catalog is regenerated.

Run python3 scripts/audit_provider008c_azure.py offline. It verifies source hashes, translator,
runtime allowlist and packaged intersection. Rust external-crate tests verify runtime precedence
and exclusions; controlled HTTP tests verify factory dispatch. Neither audit nor fixtures prove
live vendor acceptance. Shared auth #132, Bedrock and release are separate work.

Wire/auth references checked 2026-11-14:
- https://learn.microsoft.com/en-us/azure/ai-foundry/openai/api-version-lifecycle?tabs=key
- https://learn.microsoft.com/en-us/rest/api/microsoft-foundry/azureopenai/responses?preserve-view=true&view=rest-microsoft-foundry-v1
- https://ai-sdk.dev/providers/ai-sdk-providers/azure

Current exact packaged admission (azure provider only):

`gpt-4.1`, `gpt-4.1-mini`, `gpt-4.1-nano`, `gpt-4o`, `gpt-4o-mini`, `gpt-5`, `gpt-5-mini`, `gpt-5-nano`, `gpt-5.1`, `gpt-5.2`, `gpt-5.4`, `gpt-5.4-mini`, `gpt-5.4-nano`, `gpt-5.5`, `gpt-5.6-luna`, `gpt-5.6-sol`, `gpt-5.6-terra`, `gpt-chat-latest`, `o1`, `o3`, `o3-mini`, `o4-mini`.
