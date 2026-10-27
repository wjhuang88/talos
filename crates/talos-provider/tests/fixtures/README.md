# Responses contract replay fixtures

`responses_parallel.synthetic.sse` is authored synthetic test data, not a live API capture or
provider response recording. It serializes the current Responses event contract checked on
2026-10-27 against https://developers.openai.com/api/docs/guides/function-calling and
https://developers.openai.com/api/reference/resources/responses/streaming-events. No upstream
fixture/code is copied. IDs, names, arguments and token counts are invented.

The fixed wire bytes exercise two distinct item_id/call_id pairs, interleaved Unicode argument
deltas, reverse completion order, unknown events, ToolUse terminal and usage. This adds the
missing parallel replay regression; it does not itself establish captured-upstream provenance.

## Upstream SDK persisted streams

`vercel-ai/` contains unmodified Vercel AI SDK fixtures pinned at
`e59b6e017aece6cc45c4ee7d94af88829621ed8d` from https://github.com/vercel/ai .
Original directory: `packages/openai/src/responses/__fixtures__/`.
The upstream `openai-responses-language-model.test.ts` consumes these through
`prepareChunksFixtureResponse`: each nonempty JSON line is framed as SSE `data`, matching our
replay harness. The encrypted-content fixture is used by upstream's real-fixture reasoning test
and its streaming encrypted-content test (lines 2816 and 11544 at this revision); the error fixture
is used by the API streaming-error test (10704). This is upstream persisted SDK evidence, not a
new live call by Talos; original acquisition was not independently repeated.

- `openai-reasoning-encrypted-content.1.chunks.txt`, SHA256
  `a1e8b0e752046299cafe44af4c5fcdcda5ceb9a9ee5c117304b39ef527a58edc`.
  It concatenates four Responses turns, replayed with a fresh decoder after each completed event.
  Covers reasoning summary and opaque state, native calculator calls, final text and usage.
- `openai-error.1.chunks.txt`, SHA256
  `9046a4b9521a02eec098359b70bf8cc89ef1e5338375f91618c73cb650132b25`.
  Covers upstream top-level error followed by response.failed, without manufactured success.

Copyright 2023 Vercel, Inc.; Apache-2.0. The exact upstream root license notice is retained at
`vercel-ai/LICENSE`; Talos's root LICENSE supplies the full Apache-2.0 license text. No root NOTICE
file exists in the pinned upstream tree. No SDK runtime code or dependency is imported. Fixture
bytes and encrypted content are never printed by passing tests or added to request previews.
The separate synthetic parallel fixture above deliberately stresses interleaving/reverse completion;
it is not relabeled as upstream data. Newline-to-SSE framing occurs only in the test harness.
