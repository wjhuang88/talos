# MEM-007 Active Context Compression — Spike Notes

Created: 2026-06-30 (T20 of the four-month plan)
Status: Spike (research prototype); no production enablement without cache-stability proof

## Objective

Evaluate deterministic pre-entry compression strategies for `read`, `grep`, `git_diff`, and `bash`
tool outputs. The goal is to measure token savings and cache-stability risk without enabling
automatic compression by default.

Source backlog: `docs/backlog/active/MEM-007-active-context-compression.md`.

## Hard Constraint Recap

Prompt cache stability is a **gate, not a guideline**. Any compression strategy that invalidates
the stable prefix (`build_stable_prefix()`) is rejected. Compression applies only to new tool
results entering the dynamic suffix, is deterministic (same input → same compressed bytes), and
is never retroactive.

## Per-Tool Compression Strategies (deterministic)

### `read` — File content

| Trigger | Strategy | Estimated savings |
|---|---|---|
| Agent used `offset`/`limit` | Preserve as-is (already scoped) | 0% |
| Triggered by `grep`/`find_symbol` continuation | Extract ±N lines around matched region (default N=5) | 70-90% |
| Standalone read of large file (>200 lines) | Structure summary: function/class/struct headers + line counts + offer offset reads | 80-95% |
| Small file (≤200 lines) | No compression (full content is useful) | 0% |

Risk: if the agent re-reads the same file later, the compressed form changes the cache key for
that message. Mitigation: compressed form is canonical from insertion — never re-compressed.

### `grep` — Search results

| Trigger | Strategy | Estimated savings |
|---|---|---|
| ≤20 matches | Preserve as-is | 0% |
| >20 matches | Top-20 matches + total count + file distribution + "use offset for more" | 50-80% |

Risk: TUI-014 already summarizes grep output in scrollback. This compression is for the
**model-facing** context, not the display. The two are independent: display summary is visual,
context compression is semantic.

### `git_diff` — Unified diff

| Trigger | Strategy | Estimated savings |
|---|---|---|
| ≤10 hunks | Preserve as-is | 0% |
| >10 hunks | Hunk headers + statistics + top-5 largest hunks; full diff in raw log | 60-80% |

### `bash` — Command output

| Trigger | Strategy | Estimated savings |
|---|---|---|
| ≤30 lines stdout | Preserve as-is | 0% |
| >30 lines stdout | Last 30 lines + truncation marker; full output in raw log | 50-90% |

Note: TUI-015 already applies head+tail truncation for **display**. This compression is for
**model context** — the model sees the compressed form, not the full output.

## Cross-Turn Dedup (deterministic)

| Pattern | Strategy | Estimated savings |
|---|---|---|
| Same file `read` multiple times | Keep latest full content; earlier reads become `"see turn N for {filename}"` | 30-60% (depends on repetition) |
| Same `grep` query repeated | Merge results, dedup by `file:line` | 10-30% |

Risk: dedup changes historical messages. This MUST NOT happen inside the cached prefix boundary.
Rule: dedup applies only to messages in the dynamic suffix, never to cached-prefix messages.

## Raw Output Preservation

Compressed tool results are the **model-facing** representation. The full uncompressed output is
preserved in:
- Session JSONL log (durable source of truth — already exists).
- `/export` transcript (writes raw content, not compressed form).
- Debug/diagnostics.

The in-memory `Message::Tool` carries the compressed form. The raw form is retrievable from the
JSONL log by turn index.

## Cache-Stability Proof Requirements

Before any compression strategy is enabled (even behind a feature flag):

1. **Stable prefix hash test**: compute the stable prefix hash with compression disabled and
   enabled. The hashes MUST be identical. This is a P0 regression test.
2. **Determinism test**: run the same tool output through the compressor twice. The compressed
   bytes MUST be identical. No timestamps, no random ordering, no session-dependent state.
3. **No-retroactive-compression test**: after a message is inserted, re-running the compressor
   on the same history MUST NOT change any existing message bytes.

## Spike Deliverables (this task)

This spike produces:
- This design note with per-tool strategies and risk assessment.
- A decision on which strategies are safe to prototype in T26 (minimal compression packet).
- No production code. No feature flag. No automatic enablement.

## Recommendation for T26

T26 ("Implement MEM-007 minimal compression packet for one low-risk tool family, default off")
should target `bash` output as the first family:
- Highest token savings (command outputs are often very large).
- Simplest strategy (last-N-lines + truncation marker — already proven by TUI-015 display logic).
- Lowest cache risk (bash output enters the dynamic suffix and is rarely in the cached prefix).
- Raw output already preserved in JSONL log.

`read` and `grep` compression should wait until the bash packet is proven and the cache-stability
test infrastructure exists.

## Interaction with Existing Systems

| System | Interaction |
|---|---|
| MEM-005 (Compaction Policy) | Compression delays compaction triggers; MEM-005's boundary-aware policy still decides WHEN to compact |
| TUI-014/TUI-015 | Display summarization is independent; context compression is semantic. Both can coexist |
| ARCH-006 (Prompt Cache Stability) | Hard constraint source; compression must not touch stable prefix |
| TOOL-002 (Tool Execution) | Compression hook lives after tool returns, before result enters history |
