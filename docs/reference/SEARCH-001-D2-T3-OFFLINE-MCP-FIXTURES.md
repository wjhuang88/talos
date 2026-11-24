# I337 synthetic MCP observation companion (partial)

Research/test only. The tool creates a local IPv4 loopback HTTP server on an ephemeral port; it does not read credentials, accept a target URL or contact Exa/Parallel. All schema and result examples are invented fixtures, not captured vendor wire. It uses the standard-library developer-script convention of the I294 observation tool, with no new Talos runtime dependency or production protocol implementation.

Run from repository root:

```sh
python3 scripts/search_mcp_offline_observation.py --provider exa --case json --talos-commit "$(git rev-parse HEAD)"
python3 scripts/search_mcp_offline_observation.py --provider parallel --case sse --talos-commit "$(git rev-parse HEAD)"
python3 -m unittest discover -s scripts -p test_search_mcp_offline_observation.py -v
```

The commit is operator-supplied metadata checked for 40 hexadecimal characters, not independently verified by the tool. Replay is by fixed provider/case identifier; observations include UTC time, synthetic/LOCAL-ONLY labels, query digest, request count, outcome and admissible result count. No query text, snippets, result URLs, provider error payload, key or cookie is persisted. Response bound 64 KiB, request budget at most four, no semantic retries or followed redirects. The fixture's result allowlist is deliberately synthetic, not production URL/SSRF policy.

Cases include JSON/SSE, extra/duplicate/missing tools, schema drift, malformed payload, wrong ID, RPC/tool errors, truncated/oversized responses, 401/403/429/500, redirect, invalid result URL and injected TLS/proxy faults. Pre-call cancellation/deadline/budget scenarios verify stopping before the fourth call; they are not live TLS/proxy or mid-stream cancellation tests. The helper only exercises the controlled local server, not the Rust MCP transport or real provider protocol.

Checkpoint verification: 4 local unittest methods pass, covering 50 provider/case combinations plus additional assertions. Actual Rust integration and native CI, security review, and source/owner closeout remain required under I337. Do not use this prototype for live acquisition. #848/#624 remain open.
