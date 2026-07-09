# ADR-037: Compact Text Session Log Format and Archival Architecture

## Context

Talos stores durable session history as append-only JSONL files under `~/.talos/sessions/`. JSONL
has low information density: every entry repeats JSON field names, timestamps are verbose ISO strings,
UUIDs are 36-character strings, and structural characters (`{}`, `""`, `:`) add overhead on every
record. For long coding sessions with heavy tool output, session files grow quickly and resume/scan
paths repeatedly parse text JSON.

The original SESSION-004 evaluation selected a binary format (postcard-framed `*.tlog`). After design
review, a compact text format was chosen instead because it preserves human readability, Unix tool
compatibility (`less`, `grep`), simpler crash recovery, and lower migration risk while still achieving
40–65% size reduction over JSONL.

Additionally, two distinct mechanisms must be separated:

1. **Tool Output Compression** (per-request, permanent): every tool execution may compress its output
   before injecting it into the conversation. The session record stores both the model-facing
   (compressed) and raw (full) versions.
2. **Session Compaction with Archival** (episodic, context-window-driven): when the conversation
   exceeds context limits, old turns are summarized, the current segment is frozen and compressed at
   the file level, and a new active segment is created.

These mechanisms are orthogonal and must not be conflated.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
| --- | --- | --- | --- |
| Session log is append-only | Hard | SESSION-002 integrity | No |
| Existing JSONL sessions must still resume/search/fork/export | Hard | Backward compatibility | No |
| No `unsafe` without ADR | Hard | AGENTS.md #2 | No |
| No arbitrary C/C++ bindings without ADR | Hard | AGENTS.md #1 | Yes — ADR-036 covers zstd |
| Human-readable / debuggable session files | Soft | Operations, support, governance audit | Yes |
| Format should be more compact than JSONL | Soft | SESSION-004 motivation | Yes |
| Content may differ between model view and display view | Assumption | Future tool output compression | Yes — design accounts for it |

## Reasoning

### Why compact text instead of binary

| Dimension | Binary (postcard) | Compact Text (TSV + length-prefix) |
| --- | --- | --- |
| Density vs JSONL | -60–70% | -40–65% |
| Human readable | No (needs tool) | Yes (`less`, `grep`) |
| Crash recovery | CRC + frame scan | Line/length scan, natural |
| Unix tool friendly | No | Yes |
| Migration risk | High (JSON→binary) | Low (text→text) |
| Dependencies | postcard crate | None new |
| Governance audit | Needs decoder | Direct read |

The 10–20 percentage point density gap does not justify losing human readability for a session log
use case where debuggability, crash safety, and auditability are important.

### Why length-prefix instead of escaping

Traditional TSV/CSV breaks on content containing the delimiter (`\t`) or record separator (`\n`).
Escaping (`\\t`, `\\n`) adds overhead and complexity. Instead, the content field uses a
length-prefix (`<len>:<raw_bytes>`) so the reader reads exactly N bytes — tabs, newlines, and any
other bytes pass through without escaping.

This is a hybrid format: header fields are simple TSV (short values, no tabs expected), and the
content field is binary-safe via length-delimiting. It is still fully text for typical content.

### Why segment-chain archival

Single-file session logs grow unboundedly. When context compaction summarizes old turns, the
summarized version should not destroy the original — it should produce a new file and archive the
old one. This is a Log-Structured Merge (LSM) pattern applied to session logs:

- **Active segment** (`head.tlog`): append target, stores current model-view content
- **Archived segments** (`s001.tlog.zst`, ...): frozen, compressed, immutable
- **Chain metadata** (`chain.tlog`): segment list, lineage, compaction history

Benefits: append-only preserved, natural compression of cold data, granular access (LLM reads only
head segment), crash safety (failed compaction leaves old segment intact).

### Why fork uses snapshot references

When forking, the new session references the parent by `(parent_session_uuid, parent_segment_id,
parent_record_offset)` — a snapshot of the parent state at fork time. The parent may later compact
(archiving its head segment), but archived segments are immutable and non-deletable while referenced.
This provides copy-on-write semantics without copying segment files.

## Decision

### 1. Compact Text Record Format (`.tlog`)

Each record is a single line with TSV header fields followed by a length-prefixed content field:

```text
<kind>\t<field1>\t<field2>\t...\t<content_len>:<content_bytes>\n
```

Reader logic: split header by `\t`, parse `content_len` from the `<decimal>:` prefix, `read_exact(N)`
for content, expect `\n` terminator.

Record kinds:

```text
M  Message       M\t<role:0-2>\t<ts_ms>\t<len>:<content>\n
C  Tool Call     C\t<ts_ms>\t<tool_name>\t<call_id>\t<len>:<input_bytes>\n
R  Tool Result   R\t<ts_ms>\t<call_id>\t<status:0-2>\t<raw_flag:0-2>\t<model_len>:<model_bytes>[\t<raw_payload>]\n
E  Event         E\t<ts_ms>\t<event_type>\t<len>:<event_bytes>\n
P  Compaction    P\t<ts_ms>\t<source_segment>\t<src_records>\t<dst_records>\t<len>:<rules_json>\n
```

Tool Result `raw_flag`:
- `0`: raw == model (no separate storage; common case for small outputs)
- `1`: raw inline (`<raw_len>:<raw_bytes>` follows model content)
- `2`: raw external (`<ref_type>:<ref_id>` follows model content, e.g. `sqlite:blob:<hash>`)

### 2. Segment File Header

```text
TALOS\tv1\t<session_uuid>\t<segment_id>\t<prev_segment_id|none>\t<created_ts>\t<compacted_ts|none>\n
```

`prev_segment_id` forms the chain. `none` marks the original (pre-compaction) segment.

### 3. Segment Chain Metadata (`chain.tlog`)

```text
S\t<segment_id>\t<status>\t<prev_segment_id|none>\t<record_count>\t<orig_bytes>\t<archived_bytes|->\t<created_ts>\t<archived_ts|->\t<archive_format|->\t<ref_count>\n
```

`status`: `active` / `archived` / `compressed`. `ref_count` prevents deletion of segments referenced
by forks.

### 4. Session Directory Layout

```text
~/.talos/sessions/<workspace_hash>/<session_uuid>/
├── head.tlog          ← active segment (append target)
├── s001.tlog.zst      ← archived segment 1 (compressed, immutable)
├── s002.tlog.zst      ← archived segment 2
└── chain.tlog         ← segment chain metadata
```

Legacy single-file JSONL sessions are read as a single "legacy segment" with `prev_segment_id=none`.

### 5. Two Independent Mechanisms

**Tool Output Compression (Mechanism A)**:
- Triggered per-request when a tool produces output exceeding a threshold.
- The R record stores both `model_bytes` (what the LLM sees) and `raw_bytes` (full output).
- `raw_flag=0` when no compression needed (small output); `raw_flag=1` for inline raw;
  `raw_flag=2` for external blob storage.
- Immutable once written — does not change with session lifecycle.

**Session Compaction with Archival (Mechanism B)**:
- Triggered when context window pressure exceeds a threshold, turn count limit reached, or explicit
  `/compact` command.
- Freezes current `head.tlog` → renames to `s00N.tlog`.
- Reads all records, applies compaction rules (tool result summarization, redundant turn collapse,
  thinking removal per ADR-034).
- Writes compacted records to new `head.tlog`.
- Compresses `s00N.tlog` → `s00N.tlog.zst` (per ADR-036), deletes uncompressed original.
- Updates `chain.tlog`.
- Archived segments are immutable and non-deletable while `ref_count > 0`.

### 6. Fork Snapshot Reference

```text
# Forked session head.tlog header
TALOS\tv1\t<fork_uuid>\t<head_seg>\t<parent_ref>\t<forked_ts>\n

# parent_ref format:
parent:<parent_session_uuid>:<parent_segment_id>:<fork_record_offset>
```

Parent deletion policy:
- Default: reject deletion if any fork references the session (check `ref_count` in chain.tlog).
- `--force-orphan`: copy referenced segments to fork directory, break COW.
- `--force-dangling`: accept dangling reference; fork's new conversation works but history is
  truncated at the fork point.

### 7. Compression (per ADR-036)

Archived segments compressed with zstd (C binding, ADR-036). Compression behind `SegmentCompressor`
trait for future pure Rust migration (COMP-001).

### Rejected Alternatives

- **Binary format (postcard)**: loses human readability for marginal density gain.
- **Optimized JSONL (short field names)**: still carries JSON parsing overhead and structural
  characters.
- **RON / other text formats**: append semantics don't fit line-oriented log pattern; ecosystem
  support limited.
- **Dual-write (JSONL + compact text)**: violates single-source-of-truth and adds I/O cost.
- **SQLite as primary conversation log**: changes crash-recovery model; SQLite remains the index
  layer, not the durable conversation store.

## Reversal Trigger

Revisit this decision when:

1. The compact text format proves insufficient for very high-frequency session writing (benchmarks
   show write bottleneck).
2. A future requirement needs binary-only metadata (e.g., embedded images, signed reasoning blocks)
   that text framing cannot efficiently represent.
3. The segment-chain archival model proves too complex for the actual usage patterns (most sessions
   are short and never compact).
4. A new industry-standard agent session format emerges that Talos should adopt for interoperability.

## Related Documents

- `docs/backlog/active/SESSION-004-binary-session-log-format.md` — implementation story (revised)
- `docs/decisions/036-zstd-compression-dependency.md` — zstd C binding
- `docs/decisions/002-local-storage-architecture.md` — progressive storage strategy
- `docs/decisions/034-reasoning-thinking-boundary.md` — thinking/reasoning persistence policy
- `docs/backlog/active/COMP-001-pure-rust-compression-migration.md` — compression migration watch
