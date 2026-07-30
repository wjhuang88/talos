# Provider Terminal Outcomes

Talos treats the provider protocol's explicit terminal signal as authoritative. It does not infer a
successful response from prose shape or from the transport simply closing.

## User-visible outcomes

| Outcome | TUI | `--print` | Exit behavior |
|---|---|---|---|
| Explicit normal completion | Response completes quietly | Response completes quietly | Success |
| Output token limit (`MaxTokens`) | Partial response remains visible with a truncation notice | Partial stdout remains visible and stderr reports that the response was truncated | Success, but explicitly truncated |
| Unsupported finish/stop reason | Bounded provider failure | Partial stdout may remain visible; stderr names the bounded unsupported reason | Failure |
| Stream closes without terminal frame | Bounded provider/stream failure | Partial stdout may remain visible; stderr explains the missing explicit terminal signal | Failure |
| Invalid UTF-8 or transport read failure | Bounded provider/transport failure | stderr identifies decode or transport category | Failure |

A truncation notice means the visible partial answer is retained intentionally; it must not be
mistaken for a complete answer. Retry or continuation is a user decision—Talos does not silently
continue, fail over, or infer intent.

## Tool continuation failures

When a tool call and result completed successfully but the following provider continuation fails,
Talos preserves the completed tool exchange under SESSION-006. A half-streamed continuation
fragment is not committed as a completed assistant fact, and the provider failure is reported
separately rather than being described as a tool failure.

## Diagnostic evidence

Interactive sessions retain one bounded terminal diagnostic per provider-response boundary. The
record contains normalized outcome, source, bounded reason/category, provider/model identity,
`turn_id`, and response ordinal. It does not contain credentials, headers, raw request/response
bodies, hidden reasoning, or raw tool output.

Terminal diagnostics survive supported TLOG reopen and recent-entry compaction, but are excluded
from conversation messages, provider history, transcript hydration, copy, and export.

## Deterministic verification

No paid provider credentials are needed:

```bash
cargo build --locked -p talos-cli
scripts/verify_i168_provider_terminal.sh target/debug/talos
```

The fixture covers OpenAI-compatible and Anthropic-compatible normal completion, MaxTokens,
unsupported reasons, terminal-frame-less EOF, invalid UTF-8, transport failure, and a successful
tool result followed by continuation EOF.

The I168 completion packet used workflow run `30552762936` at validation harness commit `62ae098d`.
All focused commands, source scan, governance validation, build, and the rebuilt-binary fixture
exited 0; `cargo test --workspace --locked` recorded 2673 passed and zero failed. The governance
closeout mutation then passed in workflow run `30554255195`. Completion Commit `2eac5b05` predates
both evidence-recording steps and owns the final implementation plus deterministic fixture scripts.

## Troubleshooting

- A normal provider gateway must emit the compatible protocol's documented terminal signal.
- A gateway that closes after content without `[DONE]`, `finish_reason`, or Anthropic
  `message_delta.stop_reason` is treated as malformed rather than successful.
- For post-incident inspection, use the retained session/TLOG evidence; do not copy raw provider
  payloads or credentials into logs.
