# Provider Terminal Outcomes

Talos treats the provider protocol's explicit terminal signal as authoritative. It does not infer a
successful response from prose shape or from the transport simply closing.

## User-visible outcomes

| Outcome | TUI | `--print` | Exit behavior |
|---|---|---|---|
| Explicit normal completion | Response completes quietly | Response completes quietly | Success |
| Output token limit (`MaxTokens`) | Partial response remains visible with a truncation notice | Partial stdout remains visible and stderr reports that the response was truncated | Success, but explicitly truncated |
| Known filtered/refused/paused/deprecated reason | Explicit bounded policy result | Partial stdout may remain visible; stderr names the known policy outcome | Failure unless the known reason is normal (`stop_sequence`) |
| Truly unknown finish/stop reason | Bounded provider failure | Partial stdout may remain visible; stderr names the bounded unsupported reason | Failure |
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
known filtered/refused/paused/deprecated reasons, a synthetic unknown reason, terminal-frame-less
EOF, invalid UTF-8, transport failure, and a successful tool result followed by continuation EOF.
OpenAI `content_filter` is a known filtered failure and deprecated `function_call` is rejected with
migration guidance. Anthropic `stop_sequence` is normal completion; `pause_turn` and `refusal` are
explicit bounded non-success outcomes because Talos does not automatically resume server-tool
pauses.

The strengthened assertions require normal completion to keep stderr empty, MaxTokens to emit
exactly one truncation notice, known policy outcomes to avoid the unknown classification, and every
error path to exclude a stale truncation warning. A merged stdout/stderr run additionally proves the
partial answer is newline-terminated before the warning.

The initial I168 completion packet used workflow `30552762936`. Maintainer review then exposed
known-reason policy and merged-terminal-output gaps. Correction revalidation workflow `30558757429`
(job `90925992796`) passed all focused tests, workspace fmt/check/Clippy/tests, governance, build,
and the strengthened rebuilt-binary fixture. Completion Commit `86262d02` owns the clean corrected
implementation and contains no workflow logs, cache files, patch scripts, or CI permission changes.

## Troubleshooting

- A normal provider gateway must emit the compatible protocol's documented terminal signal.
- A gateway that closes after content without `[DONE]`, `finish_reason`, or Anthropic
  `message_delta.stop_reason` is treated as malformed rather than successful.
- For post-incident inspection, use the retained session/TLOG evidence; do not copy raw provider
  payloads or credentials into logs.
