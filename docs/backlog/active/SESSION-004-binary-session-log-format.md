# SESSION-004: Binary Session Log Format Evaluation

| Field | Value |
|---|---|
| ID | SESSION-004 |
| Type | Research / Product Story |
| Priority | P1 |
| Status | Ready for Implementation — evaluation complete |
| Source | Maintainer request 2026-07-06 — JSONL session files have low information density; evaluate binary session storage and candidate crates `winnow` / `binrw` |
| Depends on | SESSION-002, MEM-002, ADR-034 |
| Blocks | Durable long-session storage density, faster resume scans, future session export/archive format |

## Problem

Talos stores durable session history as append-only JSONL files under `~/.talos/sessions/...`.
JSONL has been useful because it is human-readable, append-only, easy to repair after a crash, and
compatible with older session files. The downside is low information density:

- every entry repeats JSON field names;
- timestamps, UUIDs, role strings, metadata keys, and nested reasoning/tool payloads are verbose;
- resume and scan paths repeatedly parse text JSON;
- large tool sessions make local session files and cleanup pressure grow quickly.

Current session JSONL is not just a serialization detail. It also provides append-only persistence,
crash-tail tolerance, backward compatibility with older session files, request-history
reconstruction, durable event replay, session list previews, fork/resume/delete behavior, and
SQLite FTS index reconciliation.

Any binary replacement must preserve those behaviors or explicitly stage a compatibility migration.

## Evaluation Verdict

SESSION-004 is ready for implementation as a staged storage-format change.

Selected first implementation direction:

- Talos-owned framed append-only binary log.
- `postcard` payload encoding for first implementation.
- Manual/simple frame header read/write in `talos-session`.
- New sessions use the binary format as the preferred/default format in the first implementation.
- JSONL reader remains supported as legacy compatibility for existing sessions.
- Normal operation does not dual-write JSONL and binary.
- Human-readable transcript/export is provided through a shared session transcript service that
  both CLI export commands and future WEB-001 pages can use.

Rejected for first implementation:

- `winnow` as the primary dependency.
- `binrw` as the primary dependency.
- raw `bincode` as the selected payload format.
- native compression in the first slice.

Rationale:

- `postcard` is serde-compatible, compact, varint-oriented, and documents a stable wire format.
  That matches Talos's existing serde DTOs and long-term compatibility needs.
- `binrw` is useful for declarative binary structures, but Talos only needs a small explicit frame
  header plus serde payloads.
- `winnow` is a parser toolbox, not a storage format or serializer.

## Candidate Assessment

### `winnow`

`winnow` is a parser combinator library. Its docs position it as a flexible parser toolbox for
strings, binary data, separate lexing/parsing phases, and high-performance custom parsers.

Fit for Talos:

- Good if Talos designs a custom byte grammar and wants precise recovery scanning.
- Good for hand-written validation of magic/version/frame headers.
- Not a complete persistence format: it does not provide a serializer, schema evolution model, or
  append-log framing by itself.

Assessment:

- Do not choose `winnow` as the primary session storage dependency.
- Keep it as a possible low-level parser only if a future custom frame scanner needs more than
  simple `Read` helpers.

### `binrw`

`binrw` provides declarative binary readers and writers through `BinRead` / `BinWrite` derives and
attributes. Its docs emphasize maintainable binary data readers/writers, magic numbers, byte order,
padding/alignment, and nested binary structures.

Fit for Talos:

- Better than `winnow` for struct-shaped binary read/write.
- Good for fixed headers, magic bytes, version fields, and simple nested structs.
- It is a macro-heavy binary format toolkit, not a session-log format. It does not solve
  compatibility policy, record-level checksums, text export, or crash-tail recovery by itself.

Assessment:

- Reasonable for a small custom header/frame layer.
- Not the best primary choice for serializing Talos `SessionEntry` payloads because Talos already
  has serde DTOs and needs compatibility discipline more than declarative binary layout syntax.

### Recommended Direction

Prefer a framed append-only binary log with explicit Talos-owned wire DTOs:

```text
magic/version header
record*

record =
  kind u8
  schema_version u16
  payload_len varint/u32
  payload_crc32 optional
  payload bytes
```

Payload should be encoded with a serde-compatible compact binary format. Selected first
implementation candidate:

- Use `postcard` because it has a documented stable wire format, uses varints for
  lengths/discriminants, and works with serde DTOs.
- Keep `bincode` only as a future benchmark/debug comparison if postcard results are unexpectedly
  poor. Do not block first implementation on adding bincode as a direct dependency.
- Avoid native compression in the first slice. If compression is needed later, prefer a pure-Rust
  compression crate or create an ADR for any native dependency.

## Product Direction

Use a binary-first format strategy:

1. Keep JSONL reader support indefinitely for existing sessions.
2. Introduce a `SessionStore` abstraction behind current `Session` APIs.
3. Make the binary writer the default for newly created sessions in the first implementation after
   compatibility tests pass.
4. Keep explicit export to JSONL or human-readable transcript so support/debuggability is not lost.
   The export renderer should be a reusable service, not a TUI-only path, so the future WEB-001
   dashboard can provide a session history page/export button using the same backend.
5. Do not dual-write in normal operation. If a temporary debug dual-write mode is ever added, it
   must be test-only or explicitly gated and must name one source of truth.
6. Do not auto-migrate existing JSONL files on upgrade. Migration remains an explicit future user
   action.

## Development-Ready Scope

### Slice A: Format Boundary And Store Abstraction

Deliverable:

- A `SessionStore` internal boundary that can read JSONL sessions and binary sessions behind the
  current `Session` public API.
- New session creation routes through the binary store by default once Slice B lands.

Required behavior:

- Existing `*.jsonl` files still work.
- `SessionManager` can discover both `*.jsonl` and the new binary extension.
- Newly created sessions use the binary extension by default.
- JSONL is treated as a legacy store, not the preferred write format.
- `read_entries`, `read_messages`, `read_events`, and session preview scanning route through the
  store boundary.
- Fork logic is expressed in store terms rather than assuming line-oriented JSONL copying.
- If both `.jsonl` and binary files exist for the same session ID, the manager must report an
  explicit duplicate/corrupt state or apply a documented deterministic priority. It must not
  silently merge records from both files.

### Slice B: Postcard-Framed Binary Store

Deliverable:

- New append-only binary session file format, proposed extension: `*.tlog`.
- New sessions write `*.tlog` by default.

Frame shape:

```text
file header:
  magic: b"TALS"
  format: u8 = 1
  flags: u8
  reserved: u16

record:
  magic: b"TR"
  kind: u8
  schema_version: u16
  payload_len: u32 little-endian
  payload_crc32: u32
  payload: postcard bytes
```

Record kinds:

- `1`: session entry.
- `2`: durable agent event, if event persistence remains separate from entries.
- `255`: reserved for future metadata/checkpoint records.

Wire DTO rule:

- Define `SessionEntryV1`, `SessionMetadataV1`, and any event payload wrapper separately from the
  public `SessionEntry` and provider/runtime enums.
- Convert between wire DTOs and public/runtime types explicitly.
- Do not persist unstable Rust enum discriminants or rely on derive layout as a compatibility
  contract.

Crash behavior:

- On read, validate magic, length, CRC, and postcard decode.
- If the final record is truncated or fails CRC/decode, skip it and return all prior valid records.
- If a middle record is corrupt, return an explicit corruption error unless a later repair story
  defines resynchronization rules.

### Slice C: Transcript Export Service And Web Reuse Point

Deliverable:

- A storage-format-neutral transcript/export service in `talos-session` or a narrow adjacent
  module.
- CLI/export paths and future WEB-001 pages must call this service instead of parsing JSONL
  directly.

First slice requirement:

- Provide machine-readable JSON export and plain/Markdown transcript export from both JSONL and
  binary sessions.
- The WEB-001 integration itself remains future work, but SESSION-004 must leave a stable service
  boundary for a web session-history page.

Web integration note:

- WEB-001 should expose session history and export only through loopback/dashboard security rules.
- The web page must not read raw session files directly; it calls the same transcript/export
  backend used by CLI.

## Proposed Implementation Path

1. Add format fixtures and density measurements:
   - sample realistic session fixtures: short chat, tool-heavy chat, reasoning-heavy Anthropic
     chat, long 1000-entry session, forked session;
   - compare current JSONL and postcard framed records;
   - optionally compare bincode only if postcard density or latency is not acceptable;
   - measure file size, append latency, full read latency, preview scan latency, corrupted-tail
     recovery, and index reconciliation time.
2. Define explicit wire structs separate from public `SessionEntry`:
   - numeric role/kind tags;
   - required schema version;
   - explicit optional metadata fields;
   - no direct persistence of unstable internal enum layout without versioning.
3. Implement a `SessionStore` trait or equivalent internal module boundary:
   - `append_entry`;
   - `read_entries`;
   - `read_messages`;
   - `read_events`;
   - `scan_preview`;
   - `copy_prefix_for_fork` or safe fork materialization.
4. Add binary store behind a setting or internal opt-in path.
5. Flip new-session creation to binary by default in the same implementation once compatibility
   tests pass.
6. Add transcript/export service before merging the default format flip.

## Acceptance Criteria

- [ ] A benchmark/evidence report compares JSONL and postcard-framed binary on realistic fixtures;
      bincode is included only if postcard is rejected or materially underperforms.
- [ ] The selected format has a magic header, version field, record boundary, and corrupted-tail
      recovery story.
- [ ] Existing JSONL sessions still resume, list, search, fork, export, and cleanup correctly.
- [ ] Newly created sessions use the binary format by default.
- [ ] Binary sessions can append and resume through the current `Session` public API.
- [ ] Normal operation does not dual-write JSONL and binary session records.
- [ ] Duplicate `.jsonl`/binary files for the same session ID have an explicit error or documented
      deterministic priority; no silent merge occurs.
- [ ] `read_messages()` preserves tool-call/tool-result pairing and ADR-034 reasoning metadata.
- [ ] `read_events()` preserves durable non-transient events while still excluding transient
      thinking deltas.
- [ ] `scan_file()` or its replacement can list sessions without fully hydrating every message.
- [ ] Forking a binary session does not corrupt the source session and preserves branch ancestry.
- [ ] SQLite index reconciliation works for both JSONL and binary sessions during the transition.
- [ ] Corrupt/truncated final binary record is skipped or repaired without losing earlier records.
- [ ] Human-readable export remains available through a storage-format-neutral service.
- [ ] The export service has a documented integration point for future WEB-001 session history and
      export pages.
- [ ] No native dependency or unsafe code is introduced without an ADR.

## Non-Goals

- Do not remove JSONL compatibility in the first implementation.
- Do not make SQLite the primary durable conversation log in this story.
- Do not compress with a native library in the first slice.
- Do not persist provider request-history semantics by relying on unstable Rust enum discriminants.
- Do not change session topology, workspace scoping, or cleanup policy semantics.
- Do not auto-migrate existing JSONL sessions on startup.

## Initial Recommendation

Do not choose `winnow` or `binrw` as the main answer.

Approved first implementation direction is:

- Talos-owned framed append-only binary log;
- `postcard` payload encoding;
- bincode only as an optional benchmark fallback;
- manual/simple frame header parsing;
- `binrw` only if a later review proves the frame/header code becomes clearer with derives;
- no `winnow` unless custom recovery parsing becomes complex enough to justify it.

## Implementation Gate

This story can enter development without another research-only pass if the implementation follows
the selected direction above.

Before changing the default new-session format, the implementation must provide:

- JSONL compatibility tests;
- binary append/read/resume tests;
- corrupted-tail tests;
- transcript/export tests;
- SQLite index reconciliation tests;
- duplicate-format conflict handling tests.

The first implementation is expected to include the default new-session binary write path. If that
cannot be delivered safely, keep the story open rather than landing a hidden opt-in-only binary
format.

## Validation Plan

Required targeted checks for the eventual implementation:

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

- `crates/talos-session/src/jsonl.rs`
- `crates/talos-session/src/types.rs`
- `crates/talos-session/src/manager.rs`
- `crates/talos-session/src/sqlite.rs`
- `crates/talos-cli/src/session_setup.rs`
- `crates/talos-cli/src/event_loop.rs`
- `docs/decisions/034-reasoning-thinking-boundary.md`
- `docs/backlog/active/SESSION-002-session-integrity-lifecycle-hardening.md`
- `docs/backlog/active/MEM-002-conversation-context-continuity.md`
