The test-only RSA key was generated locally with OpenSSL genpkey (2048-bit RSA) for
mock authentication tests. It has no associated account, permissions, or deployment.
Never use it for real authentication. OpenSSL is not a runtime or test dependency.


## Sourced SSE: trailing empty STOP

`go_genai_trailing_empty.sse` is extracted from Google's Go Gen AI SDK
`TestChatsStreamTrailingEmptyChunk`, `chats_test.go` lines 532–533 at commit
`087281541b864840a1ff51a14d16eea23f0d8d83` (retrieved 2026-11-05):
https://github.com/googleapis/go-genai/blob/087281541b864840a1ff51a14d16eea23f0d8d83/chats_test.go#L532

Upstream source Git blob: `cd9c1b9ca6b91e14ee36b3566f787534d166f9fd`.
Fixture SHA-256: `430ba30d37eb656932ba34ba2ad1765528ddf696ec945b7e3be5984ca6eb206c`.
Copyright 2025 Google LLC; Apache-2.0, full upstream license in `LICENSE-go-genai`.
Transformation: take the first response's two JSON literals, render their upstream
`data: %s\n\n` framing. JSON values are unchanged.
The signature decodes to the upstream synthetic `sig-1`; this is a public SDK
regression fixture, not a captured production response or live-vendor validation.

The Talos regression feeds the sample at byte widths 1, 7 and whole-buffer,
requires tools to remain buffered through EOF, and checks that the signed call,
arguments, positional binding and ToolUse terminal survive the empty STOP chunk.
