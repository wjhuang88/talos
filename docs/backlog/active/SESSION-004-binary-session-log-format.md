# SESSION-004: Compact Text Session Log Format and Archival Architecture

| Field | Value |
|---|---|
| ID | SESSION-004 |
| Type | Product Story |
| Priority | P1 |
| Status | Complete — implemented via ADR-037; reconciled 2026-07-12 (I116/LT010) |
| Source | Maintainer request 2026-07-06 (original); design revision 2026-07-09 (compact text + archival) |
| Depends on | SESSION-002, MEM-002, ADR-034, ADR-036 (zstd), ADR-037 (format decision) |
| Blocks | Durable long-session storage density, faster resume scans, compaction archival, future session export/archive format |

## Problem

Talos stores durable session history as append-only JSONL files. JSONL has low information density:
repeated field names, verbose timestamps/UUIDs, and structural character overhead. For long coding
sessions with heavy tool output, files grow quickly and resume/scan paths repeatedly parse text JSON.

Additionally, two independent requirements must be served:

1. **Tool Output Compression** (Mechanism A): tool outputs can be large; a per-request compression
   mechanism stores both a model-facing summary and the full raw output.
2. **Session Compaction with Archival** (Mechanism B): when conversations exceed context limits, old
   turns are summarized, the current segment is frozen and compressed, and a new active segment
   begins. This is an LSM-style archival model.

These mechanisms are orthogonal. See ADR-037 for the full design rationale.

## Selected Direction (revised 2026-07-09)

- **Format**: compact text (TSV header + length-prefixed content), file extension `*.tlog`.
- **Not binary**: the original postcard-framed binary direction is superseded. See ADR-037 for the
  text-vs-binary tradeoff analysis.
- **Archival**: segment-chain model with zstd-compressed archived segments (ADR-036).
- **JSONL compatibility**: existing `*.jsonl` files are read as "legacy segments." No auto-migration.
- **No dual-write**: new sessions write `.tlog` by default; JSONL is read-only legacy.
- **Compression trait**: `SegmentCompressor` trait abstracts compression for future pure Rust swap
  (COMP-001).

## Record Format

```text
# File header (first line of every .tlog segment)
TALOS\tv1\t<session_uuid>\t<segment_id>\t<prev_segment_id|none>\t<created_ts>\t<compacted_ts|none>\n

# Message
M\t<role:0-2>\t<ts_ms>\t<content_len>:<content_bytes>\n
#   role: 0=user 1=assistant 2=system

# Tool Call
C\t<ts_ms>\t<tool_name>\t<call_id>\t<input_len>:<input_bytes>\n

# Tool Result (Mechanism A: per-request tool output compression)
R\t<ts_ms>\t<call_id>\t<status:0-2>\t<raw_flag:0-2>\t<model_len>:<model_bytes>[\t<raw_payload>]\n
#   status: 0=ok 1=error 2=denied
#   raw_flag:
#     0 = raw == model (no separate raw content)
#     1 = raw inline: <raw_payload> = <raw_len>:<raw_bytes>
#     2 = raw external: <raw_payload> = <ref_type>:<ref_id> (e.g. sqlite:blob:<hash>)

# Event
E\t<ts_ms>\t<event_type>\t<content_len>:<content_bytes>\n

# Compaction Marker (first record in a compacted head segment)
P\t<ts_ms>\t<source_segment>\t<src_records>\t<dst_records>\t<rules_len>:<rules_json>\n
```

**Reader logic**: split header fields by `\t`; for content field, parse `<decimal>:` to get length N,
then `read_exact(N)` for content bytes, then expect `\n` terminator. Content may contain `\t`, `\n`,
or any byte — the length prefix makes it binary-safe without escaping.

## Segment Chain and Archival (Mechanism B)

### Session Directory Layout

```text
~/.talos/sessions/<workspace_hash>/<session_uuid>/
├── head.tlog          ← active segment (append target)
├── s001.tlog.zst      ← archived segment 1 (frozen, zstd-compressed)
├── s002.tlog.zst      ← archived segment 2
└── chain.tlog         ← segment chain metadata
```

### chain.tlog Format

```text
S\t<segment_id>\t<status>\t<prev_segment_id|none>\t<record_count>\t<orig_bytes>\t<archived_bytes|->\t<created_ts>\t<archived_ts|->\t<archive_format|->\t<ref_count>\n

# status: active / archived / compressed
# archive_format: zstd / lz4 / none
```

### Compaction Flow

1. Freeze `head.tlog` → rename to `s00N.tlog`.
2. Read all records from `s00N.tlog`.
3. Apply compaction rules (old tool results → summaries, redundant turns → collapse, thinking → remove per ADR-034).
4. Write compacted records to new `head.tlog` (first record is `P` compaction marker).
5. Compress `s00N.tlog` → `s00N.tlog.zst` (per ADR-036). Delete uncompressed original.
6. Update `chain.tlog`.

**Crash safety**: if steps 2–4 crash, `s00N.tlog` is intact (uncompressed = not yet archived); retry.
If step 5 crashes, both files exist; retry compression. `head.tlog` is not authoritative until
`chain.tlog` is updated.

### Compaction Rules (initial set)

| Rule | Applies To | Behavior |
|---|---|---|
| Tool result summarization | R records older than N turns | Replace `model_bytes` with one-line summary; preserve `raw_bytes` or external ref |
| Thinking removal | Reasoning blocks in old turns | Remove per ADR-034 transient boundary |
| Redundant turn collapse | Sequential user corrections | Collapse "no, use X" → "use X" pairs into final intent |
| Large content truncation | Any content > threshold | Truncate `model_bytes`; keep full in `raw_bytes` |

Rules are recorded in the `P` marker for auditability.

## Fork Semantics

Fork creates a snapshot reference, not a copy:

```text
# Forked session head.tlog header
TALOS\tv1\t<fork_uuid>\t<head_seg>\t<parent:<parent_uuid>:<parent_seg_id>:<fork_offset>>\t<forked_ts>\n
```

- Parent's referenced segment is immutable and non-deletable (`ref_count > 0` in chain.tlog).
- Parent may later compact — archived segments remain accessible by `segment_id`.
- Session switching does not affect parent references — each session's chain is independent.

**Parent deletion policy**:
- Default: reject if `ref_count > 0` for any segment.
- `--force-orphan`: copy referenced segments to fork, break COW.
- `--force-dangling`: accept dangling ref; fork conversation works, history truncated at fork point.

## Implementation Slices

### Slice A: SessionStore Abstraction and Segment Chain

**Deliverable**: `SessionStore` internal boundary supporting both JSONL legacy and `.tlog` segment
formats behind the current `Session` public API.

**Scope**:
- `SessionStore` trait: `append_entry`, `read_entries`, `read_messages`, `read_events`,
  `scan_preview`, `copy_prefix_for_fork`.
- Segment chain data structures: `SegmentId`, `SegmentStatus`, `ChainMetadata`.
- `chain.tlog` reader/writer.
- Legacy JSONL reader: existing `*.jsonl` files loaded as single "legacy segment" with
  `prev_segment_id=none`.
- New session creation routes through `.tlog` writer.
- Duplicate `*.jsonl`/`.tlog` for same session ID: explicit error, no silent merge.

**Not in scope**: compaction engine, compression, tool output compression.

### Slice B: Compact Text Writer/Reader

**Deliverable**: `.tlog` format writer and reader implementing all record kinds (M, C, R, E).

**Scope**:
- Compact text encoder: serialize `SessionEntry` → TSV header + length-prefixed content.
- Compact text decoder: parse TSV header, read length-prefixed content, reconstruct `SessionEntry`.
- Wire DTO types: `SessionEntryV1`, `SessionMetadataV1` separate from public runtime types.
- Corruption tolerance: truncated/corrupt final record is skipped; earlier records returned.
- Density benchmark: compare JSONL vs `.tlog` on realistic fixtures (short chat, tool-heavy,
  reasoning-heavy, 1000-entry session, forked session).
- `read_messages()` preserves tool-call/tool-result pairing and ADR-034 reasoning metadata.
- `read_events()` preserves durable non-transient events, excludes transient thinking deltas.
- `scan_file()` lists sessions without fully hydrating every message.

**Not in scope**: tool output compression (`raw_flag` always 0 in this slice), compaction archival.

### Slice C: Transcript/Export Service

**Deliverable**: Storage-format-neutral transcript/export service in `talos-session`.

**Scope**:
- Machine-readable JSON export from both JSONL and `.tlog` sessions.
- Plain/Markdown transcript export from both formats.
- CLI `/export` and `talos storage export` call this service.
- Documented integration point for future WEB-001 session history page.
- Web page must not read raw session files; it calls this service.

**Not in scope**: WEB-001 implementation, compaction archival.

### Slice D: Session Compaction and Archival Engine

**Deliverable**: Compaction engine with segment freezing, rule application, archival compression.

**Scope**:
- `SegmentCompressor` trait + zstd implementation (ADR-036).
- Compaction trigger: configurable thresholds (size, turn count, context pressure, explicit command).
- Compaction rule engine: tool result summarization, thinking removal, redundant turn collapse,
  large content truncation.
- Segment archival: freeze → compact → compress → update chain.tlog.
- `P` (compaction marker) record writing.
- Crash-safe archival: old segment not deleted until new head + chain.tlog committed.
- SQLite FTS5 index reconciliation across segments.

**Not in scope**: tool output compression (Mechanism A), fork COW.

### Slice E: Tool Output Compression and Fork COW

**Deliverable**: Per-request tool output compression (Mechanism A) and fork copy-on-write semantics.

**Scope**:
- `raw_flag` support in R records: `0` (same), `1` (inline raw), `2` (external blob).
- Tool output compression engine: threshold-based, stores model summary + raw.
- External blob storage in SQLite for large raw outputs.
- Fork snapshot reference: `(parent_uuid, segment_id, offset)` in forked session header.
- `ref_count` tracking in `chain.tlog`.
- Parent deletion policy: reject/orphan/dangling.
- Fork prefix copy via segment chain traversal (no file copy for archived segments).

## Acceptance Criteria

### Format and Compatibility
- [ ] A benchmark/evidence report compares JSONL and `.tlog` on realistic fixtures; density,
      append latency, full read latency, preview scan latency, and corrupted-tail recovery are
      measured.
- [ ] Existing JSONL sessions still resume, list, search, fork, export, and cleanup correctly.
- [ ] Newly created sessions use `.tlog` by default.
- [ ] `.tlog` sessions can append and resume through the current `Session` public API.
- [ ] Normal operation does not dual-write JSONL and `.tlog`.
- [ ] Duplicate `*.jsonl`/`.tlog` for the same session ID has an explicit error; no silent merge.
- [ ] `read_messages()` preserves tool-call/tool-result pairing and ADR-034 reasoning metadata.
- [ ] `read_events()` preserves durable non-transient events, excludes transient thinking deltas.
- [ ] `scan_file()` can list sessions without fully hydrating every message.
- [ ] Forking a `.tlog` session does not corrupt the source and preserves branch ancestry.

### Tool Output Compression (Mechanism A)
- [ ] R records with `raw_flag=0` work identically to uncompressed tool results.
- [ ] R records with `raw_flag=1` store both model and raw content; both are retrievable.
- [ ] R records with `raw_flag=2` store model content inline + raw content in SQLite blob.
- [ ] Tool output exceeding threshold is automatically compressed at write time.
- [ ] UI display can access raw content when the user expands a tool result.

### Session Compaction (Mechanism B)
- [ ] Compaction freezes the current segment and creates a new head segment.
- [ ] Archived segments are compressed (zstd) and immutable.
- [ ] Compaction marker (`P` record) records applied rules for auditability.
- [ ] SQLite index reconciliation works across active and archived segments.
- [ ] Corrupt/truncated archived segment does not prevent active segment access.
- [ ] LLM context building reads only the active (head) segment by default.
- [ ] Full history display can traverse the segment chain, decompressing as needed.

### Fork
- [ ] Fork creates a snapshot reference without copying segment files.
- [ ] Parent session can compact after fork without breaking fork's history access.
- [ ] Parent deletion is rejected when forks reference it (`ref_count > 0`).
- [ ] `--force-orphan` copies referenced segments to fork directory.
- [ ] Session switching between forked sessions does not lose parent references.

### Compression
- [ ] zstd compression is behind `SegmentCompressor` trait.
- [ ] Compression failure falls back to uncompressed archival.
- [ ] Decompression failure reports corrupt segment without crashing.
- [ ] No native dependency beyond ADR-036's approved zstd C binding.

## Non-Goals

- Do not remove JSONL compatibility.
- Do not make SQLite the primary durable conversation log.
- Do not auto-migrate existing JSONL sessions on startup.
- Do not change session topology, workspace scoping, or cleanup policy semantics.
- Do not persist provider request-history by relying on unstable Rust enum discriminants.
- Do not conflate tool output compression (Mechanism A) with session compaction (Mechanism B).

## Validation Plan

```sh
cargo test -p talos-session
cargo test -p talos-cli session
cargo test -p talos-cli model_lifecycle
cargo test -p talos-agent session
cargo check --workspace
cargo clippy --workspace -- -D warnings
cargo test --workspace
scripts/validate_project_governance.sh .
git diff --check
```

## Required Reads

- `docs/decisions/037-compact-text-session-log-format.md` — format and archival design decision
- `docs/decisions/036-zstd-compression-dependency.md` — zstd C binding approval
- `docs/decisions/002-local-storage-architecture.md` — progressive storage strategy
- `docs/decisions/034-reasoning-thinking-boundary.md` — reasoning persistence policy
- `docs/backlog/active/SESSION-002-session-integrity-lifecycle-hardening.md` — session integrity
- `docs/backlog/active/COMP-001-pure-rust-compression-migration.md` — compression migration watch
- `crates/talos-session/src/jsonl.rs`
- `crates/talos-session/src/types.rs`
- `crates/talos-session/src/manager.rs`
- `crates/talos-session/src/sqlite.rs`
- `crates/talos-cli/src/session_setup.rs`
- `crates/talos-cli/src/event_loop.rs`
