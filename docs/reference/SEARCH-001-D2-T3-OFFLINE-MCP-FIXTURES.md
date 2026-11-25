# I337 synthetic MCP observation fixtures (partial)

Research/test only. The Python companion creates a local IPv4 loopback HTTP server on an ephemeral port; the Rust tests use the real `talos-mcp` transport, dispatcher and facade against another bounded IPv4 loopback fixture. Neither reads credentials, accepts arbitrary target URLs, nor contacts Exa/Parallel. All schema and result examples are synthetic, not captured vendor wire. No runtime dependency or production protocol implementation is added.

Run from repository root:

```sh
python3 scripts/search_mcp_offline_observation.py --provider exa --case json --talos-commit "$(git rev-parse HEAD)"
python3 scripts/search_mcp_offline_observation.py --provider parallel --case sse --talos-commit "$(git rev-parse HEAD)"
python3 -m unittest discover -s scripts -p test_search_mcp_offline_observation.py -v
cargo test --locked -p talos-mcp --lib client::transport::offline_search_tests
cargo test --locked -p talos-mcp --lib
```

The commit is operator-supplied metadata checked for 40 hexadecimal characters, not independently verified by the Python tool. Replay is by fixed provider/case identifier; observations include UTC time, synthetic/LOCAL-ONLY labels, query digest, request count, outcome and admissible result count. Tests verify the metadata shape and ensure query text, result titles/snippets/URLs, provider error payloads, keys and cookies are not emitted. The Python fixture has a 64 KiB response bound, at most four requests, no semantic retries, and no followed redirects.

The Rust tests construct a private Streamable HTTP state with `ClientBuilder::no_proxy()`, redirects disabled, empty headers and a 500 ms test timeout; their URLs are generated only from an IPv4 loopback listener. They exercise `initialize` → `notifications/initialized` → `tools/list` → pinned `tools/call`, JSON/SSE, numeric IDs, malformed/mismatched/truncated/oversized replies, 401/403/429/302/500, a local-only redirect sink, in-flight timeout/cancellation, and TLS-style failure against a local plain-HTTP sink. The fixture request body/header limits and finite response cases bound the test server.

The exact-one-pinned-tool/schema and result-URL checks in Rust are **test-caller guards only**. The production facade remains lenient and the generic transport still buffers `response.text()` and includes non-success response bodies in errors; these tests characterize those existing properties, not production safety. There is no runtime MCP search allowlist, response-size cap, request budget or result URL filter added here. The Python proxy/TLS labels are injected test scenarios; only the Rust local TLS-style handshake failure is a transport error. No vendor behavior, provider rights, geography, or live network behavior is inferred. Never follow result URLs or use these fixtures for live acquisition. #848/#624 remain open.

Verification checkpoint: the Rust fixture filter passed 6 tests, all 19 `talos-mcp` library tests passed, and the Python suite passed 4 test methods covering 50 provider/case combinations plus metadata and privacy assertions. `cargo clippy --locked -p talos-mcp --all-targets -- -D warnings` passed. Full package tests and `release_preflight.sh` were attempted but did not pass: package integration tests in `git_prepared` reported inherited Git override rejection (and, with those variables removed for diagnosis, Git maintenance executable-binding errors); workspace tests reported 763 passed and 6 unrelated `talos-agent` failures. The changed MCP library tests remain green; no unrelated Git behavior was changed. Independent Network/API/security review and exact-head native CI remain required before source closeout.
