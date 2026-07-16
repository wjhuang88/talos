# Model-Private Snapshot-Anchored File Edits

**Status**: Implemented by I134 (2026-07-16)
**Date**: 2026-07-16
**Owner candidate**: TOOL-022
**Sources**:

- User design discussion, 2026-07-16
- [Kilo Code issue #11492](https://github.com/Kilo-Org/kilocode/issues/11492)
- [Hashline reference named by that issue](https://github.com/izzzzzi/opencode-hashline/blob/088c22e06da12418fb053031e5c3765e91c8da3e/README.en.md)
- `docs/backlog/active/EXT-002-oh-my-pi-feature-analysis.md`

## Outcome

Give the model a compact, stale-read-resistant way to select exact file lines while keeping
snapshot handles and hash annotations out of TUI history, approval text, exports, transcripts, and
TLOG. The first implementation must remain optional and must not change the current `read`, `edit`,
permission, or persistence behavior when the capability is disabled.

This proposal deliberately separates three concerns:

1. **Model addressing** uses a line number plus a two-hex-digit check code to limit context growth.
2. **Correctness** uses a model-private snapshot handle backed by a full file revision and full
   per-line digests held in Runtime memory. Two hex digits are never the correctness boundary.
3. **Presentation and persistence** use a sanitized projection that contains neither the snapshot
   handle nor line hashes.

## User Decisions And Constraint Classification

| Constraint | Class | Consequence |
|---|---|---|
| Hash annotations exposed to the model use exactly two hexadecimal digits. | Hard product choice | Do not replace them with long hashes in model-visible text. |
| Snapshot handles and hashes are internal mechanics and must not appear in user-facing presentation. | Hard product choice | TUI, approvals, exports, dashboard, transcript, and TLOG require sanitized projections. |
| A `read` result necessarily passes through model context before the model selects an edit. | Known architecture fact | Every displayed hash consumes context; keep the model protocol compact. |
| Full revision and full per-line digests may be retained in bounded Runtime memory. | Soft design choice | Avoids placing correctness material in model context or durable storage. |
| Phase 1 rejects any full-file revision mismatch instead of relocating an anchor. | Soft safety choice | Prevents two-digit collisions from causing reapplication mistakes. Revisit only with evidence. |
| Snapshot handles are not credentials or permission grants. | Hard security boundary | Every edit still passes through normal workspace confinement and write approval. |

## Pre-I134 Talos Capability Inventory

The proposal is not already implemented:

- `ReadTool` returns ordinary content with optional line numbers. It has no snapshot or hash mode.
- `EditTool` finds `old_string` and replaces the first occurrence. It neither rejects duplicate
  matches nor verifies that the file still matches the version read by the model.
- `ToolResult` has one `content` field. The Agent currently derives separate full UI and compressed
  model results in a few paths, but a tool cannot provide an explicit model/display/persistence
  projection.
- The TUI already summarizes successful `read` results in scrollback. This is presentation-only:
  it does not prove that hidden content is absent from session history or exports.
- `Message::Assistant.tool_calls` retains tool inputs. A future `snapshot_id` argument would be
  persisted unless it is explicitly removed from the durable and legacy session projections.
- Durable persistence redacts known credential shapes but otherwise retains model-visible tool
  results by default. Legacy persistence can additionally retain raw tool output metadata.
- The session actor is the authoritative successful-turn writer under ADR-039 and ADR-042. Any
  model-private projection must be applied there; renderers must not become persistence filters.

## Terminology

- **Snapshot handle**: a short opaque Runtime-local identifier such as `s7`. It indexes internal
  snapshot state. It is neither a content hash nor an authorization token.
- **File revision**: a full deterministic digest over the exact file bytes read.
- **Full line digest**: a full deterministic digest over one logical line's exact content bytes,
  excluding its line terminator. It remains internal.
- **Check code**: the first eight bits of the full line digest, rendered as two lowercase hex
  digits. It is model-visible and diagnostic only.
- **Model projection**: content sent to the provider as tool context.
- **Display projection**: safe summary/diff shown by TUI, CLI, dashboard, RPC, or approval UI.
- **Persistence projection**: canonical content eligible for Session/TLOG/transcript/export.

## Proposed Model Protocol

### Read

Phase 1 should use an optional tool or optional capability mode rather than silently changing every
existing `read` response. An illustrative result is:

```text
[snapshot:s7]
10:3a|pub fn open() {
11:91|    initialize();
12:0f|}
```

The short header and `:<hh>|` line suffix are visible only to the model. The implemented display
and persistence projection removes the header/check codes and retains ordinary numbered content;
the existing TUI may summarize it using its normal read threshold, for example:

```text
read src/lib.rs lines 10-12
```

The model must copy only the short handle, line number, and check code into an edit call. Long
digests never enter provider context.

### Edit

Illustrative JSON input:

```json
{
  "path": "src/lib.rs",
  "snapshot_id": "s7",
  "operations": [
    {
      "op": "replace_range",
      "start": "10:3a",
      "end": "12:0f",
      "content": "pub fn open() {\n    initialize_checked()?;\n    Ok(())\n}"
    }
  ]
}
```

Phase 1 operations:

- `replace`
- `replace_range`
- `insert_before`
- `insert_after`
- `delete`

All operations in one call are validated against the same original snapshot and applied as one
mutation. Overlapping or reverse ranges are rejected before permission-approved execution writes
anything.

### Structured failure contract

Errors must be typed internally and rendered without echoing the snapshot handle:

| Code | Meaning | Recovery |
|---|---|---|
| `SNAPSHOT_NOT_FOUND` | Runtime restarted, handle expired, was evicted, or belongs to another Runtime. | Re-read the file. |
| `SNAPSHOT_PATH_MISMATCH` | Handle was used with a different path. | Reject; re-read the intended path. |
| `FILE_REV_MISMATCH` | Exact current bytes differ from the read snapshot. | Re-read; no automatic Phase 1 reapply. |
| `HASH_MISMATCH` | The submitted two-digit code does not match the snapshot line. | Correct the reference or re-read. |
| `AMBIGUOUS_OPERATION` | Ranges overlap or operations conflict. | Submit a non-overlapping batch. |
| `INVALID_REF` / `INVALID_RANGE` | Reference cannot be parsed or is out of bounds. | Correct the call. |
| `SNAPSHOT_LIMIT` | File or registry exceeds the bounded snapshot policy. | Fall back to current read/edit behavior. |
| `ATOMIC_WRITE_FAILED` | Staged write, sync, or rename failed. | Report failure; do not report the edit as successful. |

## Internal Snapshot Registry

The registry belongs to a Runtime-scoped component shared by the snapshot read and anchored edit
tools. It must not be global and must not be placed in `talos-session`.

Each record contains:

```text
runtime-local handle
canonical workspace-confined path
full file revision
full digest per logical line
two-digit check code per logical line (or derivable from the full digest)
line byte boundaries and original terminators
creation/last-use time
```

Required lifecycle rules:

- Handles are unique for the lifetime of one Runtime and are never reused during that lifetime.
- A handle is bound to exactly one canonical path and workspace root.
- Runtime restart invalidates every handle; snapshots are never restored from TLOG.
- A successful Talos write/edit/delete invalidates every snapshot for the affected path.
- TTL, maximum snapshot count, maximum eligible file size/line count, and total metadata budget are
  mandatory. Exact defaults must be measured in the implementation iteration and documented; an
  over-limit file remains readable without snapshot support.
- Eviction is recoverable and produces `SNAPSHOT_NOT_FOUND`, never a panic.
- Cancellation or permission denial must not consume the snapshot or mutate the file.

The registry may use a short monotonic handle because the handle is not a security capability.
Permission checks, canonical path confinement, and revision validation remain mandatory even when a
handle resolves successfully.

## Hash And Text Semantics

The two-digit check code is reasonable only because line number and strict full-file revision are
also verified. With only 256 possible values, collisions are normal in non-trivial files and must
never drive relocation or identity.

Recommended digest policy:

- Compute the full file revision over exact bytes.
- Compute a deterministic full digest for each logical line, then expose its first byte as two hex
  digits. `sha2`, already used elsewhere in the workspace, is the lowest-surprise candidate; adding
  it to `talos-tools` still requires dependency review and a lockfile change.
- Exclude the line terminator from the line digest, but record the terminator separately.
- Preserve LF, CRLF, mixed terminators, trailing empty lines, and final-newline presence exactly for
  untouched content. Do not implement this using `str::lines()`, which drops information needed for
  byte-faithful reconstruction.
- Reject non-UTF-8/binary inputs through the existing file boundary. Do not silently transcode.

## Three-Projection Data Flow

```text
tool execution
  ├─ model projection: snapshot header + 2-hex hash lines  → provider context
  ├─ display projection: sanitized numbered content/diff   → EQ → TUI/RPC
  └─ persistence projection: sanitized content/diff         → session actor → TLOG
```

The projection must be created before the result is broadcast or captured as raw output. A renderer
cannot be the security boundary.

The cleanest compatibility direction is an additive, defaulted `AgentTool` projection hook rather
than changing every existing `ToolResult` constructor. Existing tools default to identical model,
display, and persistence content. Snapshot-aware tools override the projection and declare
model-private input fields such as `snapshot_id`.

The Agent/session path then carries a turn-local sidecar keyed by `tool_use_id`:

- provider messages receive model tool content and the functional tool-call input;
- `AgentEvent::ToolCall` and approval presentation receive a display-safe input with
  `snapshot_id` removed;
- `AgentEvent::ToolResult` receives display content;
- the session actor commits persistence-safe tool calls/results and never captures model-private
  content into `raw_content`;
- resumed history contains the sanitized result and therefore forces a fresh read before another
  anchored edit.

This is an explicit exception to ADR-042's current statement that durable tool results use the
model-visible representation. A new ADR or an ADR-042 amendment is required before implementation.
ADR-039 remains intact: the session actor still owns persistence and all live projections remain on
the single ordered EQ.

## Presentation Boundary

The following surfaces must never show or export snapshot handles, check codes, full revisions, or
full line digests:

- live and hydrated TUI scrollback;
- approval overlay and permission preview;
- CLI/print/inline modes;
- dashboard and RPC projections;
- copy and export commands;
- generic and durable transcript queries;
- TLOG content and metadata, including `raw_content`;
- ordinary logs, tracing fields, panic messages, and structured errors.

The model provider necessarily sees the model projection. This is intentional and must be stated
plainly: “model-private” means hidden from user-facing/history surfaces, not hidden from the model
provider. Snapshot data contains no credentials and grants no authority.

User-facing edit approval must still show material information: canonical relative path, operation
kind, target line/range, replacement preview, and write nature. Hiding `snapshot_id` must not hide
what will be modified.

## Write, Concurrency, And Crash Semantics

Phase 1 algorithm:

1. Resolve and confine the requested path using the existing workspace policy.
2. Resolve the Runtime-local snapshot and verify its bound canonical path.
3. Read exact current bytes and compare the full file revision.
4. Validate every submitted line number and two-digit check code against the stored snapshot.
5. Validate the entire non-overlapping operation batch and build new bytes in memory.
6. Pass the normal write permission pipeline using the real target path and operation preview.
7. Serialize Talos writes to the same canonical path with a Runtime-local per-path lock.
8. Re-read/revalidate after acquiring that lock.
9. Write a sibling temporary file, preserve required permissions, sync it, atomically rename it,
   and sync the parent directory where supported.
10. Emit success only after the write completes; invalidate all snapshots for that path.

This provides atomic visibility and prevents races among cooperating Talos operations. It does not
provide an absolute compare-and-swap against arbitrary external writers: portable filesystems do
not offer a universal “rename only if destination revision is X” primitive. An external process can
still write between final validation and rename. Advisory locking only helps when the other writer
cooperates. The implementation and documentation must describe this as a residual TOCTOU risk,
not claim impossible cross-process linearizability.

Symlink replacement is another race. The implementation must re-resolve confinement after locking
and place the temporary file in the validated parent directory. Any path identity change fails
closed.

## Permission And Security Boundary

- Snapshot read remains `Read`.
- Anchored edit remains `Write`/`Ask` unless existing explicit policy says otherwise.
- Snapshot creation does not pre-authorize a later edit.
- A valid handle does not bypass Deny, workspace confinement, trusted-workspace rules, or the
  permission bridge.
- Permission is evaluated at edit execution time against current policy.
- Denial emits no fabricated tool result and writes nothing.
- `snapshot_id` is omitted from observable input, but the permission engine receives the actual
  path and complete operation impact.
- Hooks that can observe functional tool calls are a potential leak path. Before implementation,
  define whether observational hooks receive the display projection while the execution hook alone
  receives model-private input. No hook may log model-private fields by default.

## Compatibility And Tool Surface

The safest first slice is optional and additive:

- keep current `read`, `write`, `edit`, and `delete` schemas and behavior unchanged;
- add snapshot-aware file tools or a conditional file backend under the existing TOOL-014
  presentation policy;
- present the snapshot edit schema only when the capability is enabled;
- do not make the model learn unified diff syntax;
- do not remove `old_string` edit until real comparative evidence supports a migration.

An alternative is adding optional fields to public `ReadInput`/`EditInput`. That is compact at the
tool-schema level but can break external Rust callers that construct the public structs directly.
It therefore requires an explicit semver/migration decision and is not the default recommendation.

## Dependency Assessment

No full Hashline library should be adopted in the first slice:

- `hashline` is close in concept but brings a broader library/CLI/MCP surface and its own protocol
  choices.
- `oxi-hashline` includes a wider patch language and optional merge behavior beyond this scope.
- `rho-hashline` uses two-digit hashes as a more central matching mechanism; Talos must not rely on
  that collision space for correctness.

The required mechanics are small: exact-byte revisioning, line-boundary parsing, typed operations,
and projection routing. A narrow digest dependency such as `sha2` is easier to audit than adopting a
complete external edit protocol. `similar`, already in `talos-tools`, remains suitable for bounded
user-facing diffs after a successful edit; it is not the mutation engine.

## Potential Problems And Required Mitigations

| Problem | Severity | Mitigation / decision |
|---|---|---|
| Two-digit collisions are common. | High if treated as identity | Treat as a check code only; strict full revision and internal full digest are authoritative. |
| Current single `ToolResult.content` can leak model-private data into history/raw output. | High | Deliver the three-projection architecture before enabling snapshot reads. |
| Tool-call input can expose `snapshot_id` in approval, replay, export, and TLOG. | High | Add model-private input projection and test every surface. |
| ADR-042 currently persists model-visible tool results. | High governance | Accept a new ADR/amendment defining transient model-only tool context. |
| Restarted sessions cannot reuse handles. | Expected | Persist only sanitized ordinary content and require the model to re-read before editing after resume. |
| Strict file revision rejects edits after unrelated external changes. | Medium UX | Phase 1 favors safety. Measure rejection rate before considering target-line reapply. |
| Automatic relocation using two hex digits can target the wrong line. | Critical | Forbidden in Phase 1. A future mode must use stored full digests and unique contextual matches. |
| Full per-line digest storage can consume memory on huge files. | Medium | Bound eligibility, registry memory, count, TTL, and use LRU eviction. |
| `str::lines()` loses CRLF/trailing-newline information. | High correctness | Use byte-boundary parsing that preserves terminators and final empty lines. |
| Validate-then-rename races with arbitrary external writers. | Medium residual | Per-path Talos lock, final revalidation, atomic replace, honest residual documentation. |
| Symlink/path identity can change during the operation. | High security | Canonical confinement and identity revalidation immediately before write. |
| Hidden fields may leak through hooks, tracing, errors, or raw metadata. | High privacy | Central projection policy plus negative tests; never rely on renderer suppression. |
| Added schemas increase model prompt size even if per-line hashes are short. | Medium | Conditional presentation; benchmark total read+edit tokens against current edit and diff modes. |
| Sanitized durable replay is not byte-for-byte equivalent to live model context. | Medium architecture | Record an explicit ADR-039/042-compatible transient-context exception and require re-read. |
| Snapshot handle mistaken for authorization. | High security | Bind path/runtime, keep short-lived, and always re-run normal permission checks. |

## Phased Delivery

### Slice A — Projection boundary

- Define model/display/persistence result projections with backward-compatible defaults.
- Define model-private tool-input fields.
- Route live events, approval, legacy Session, durable Session, transcript, copy/export, dashboard,
  RPC, hooks, and tracing through the correct projection.
- Add leakage fixtures before any snapshot token exists in production.

### Slice B — Snapshot read

- Add bounded Runtime-local registry and exact byte/line parsing.
- Return compact `snapshot_id` plus `line:hh|content` only to the model.
- Display/persist only sanitized ordinary read content; normal TUI summarization remains available.
- Prove eviction, expiry, restart invalidation, path binding, and zero-history leakage.

### Slice C — Anchored atomic edit

- Add five typed operations, batch validation, strict file revision check, normal permission gating,
  staged atomic write, bounded diff, and snapshot invalidation.
- Keep safe reapply disabled.

### Slice D — Evidence and default decision

- Benchmark context tokens, first-pass success, wrong-target rate, stale-read rejection, latency,
  and memory on representative Rust/Markdown/JSON files.
- Compare current `edit`, model-generated patch, and snapshot-anchored edit.
- Decide whether to keep optional, make default for coding models, or reject/defer.

## Acceptance Evidence For A Future Implementation

At minimum, tests must prove:

1. Two-digit codes are deterministic and long hashes never enter provider-facing text.
2. Duplicate lines and two-digit collisions cannot cause a wrong-line write.
3. A changed file produces `FILE_REV_MISMATCH` with zero mutation.
4. An expired, evicted, foreign-runtime, or wrong-path handle fails safely.
5. All five operations preserve LF/CRLF/mixed endings and final-newline state for untouched text.
6. Overlapping/reverse/out-of-range batches are all-or-nothing.
7. Permission Deny and cancellation write nothing and do not fabricate success.
8. Same-path Talos concurrency is serialized and stale snapshots lose.
9. Write/sync/rename failures are observable and do not report success.
10. Snapshot handles and hashes are absent from live/hydrated TUI, approvals, exports, transcripts,
    TLOG bytes/metadata, dashboard, RPC, logs, errors, and hook diagnostics.
11. The model receives the handle and can complete a read-to-edit fixture without long hashes.
12. Runtime restart requires re-read and restored model context remains coherent.
13. Existing unconfigured `read`/`edit` callers and their tests remain unchanged.
14. Benchmarks report total input/output tokens rather than only edit-call output size.

## Resolved Decisions

- ADR-045 selects additive defaulted `AgentTool` projections; existing implementers remain
  source-compatible.
- Provider/tool hooks see sanitized projections. An unchanged Hook return restores private data
  only on the provider/execution branch; a modification safely replaces it.
- Runtime-local limits are 15 minutes, 64 snapshots, 2 MiB/50,000 lines per file, and 16 MiB total
  accounted metadata.
- Normal Talos composition uses one shared registry across the existing four file tools. Legacy
  constructors remain the opt-out and rollback path.
- Safe reapply remains disabled. Increased retries/context cost require a new measured story rather
  than an implicit protocol expansion.

Maintainer authorization on 2026-07-16 activated I134. ADR-045 resolves the transient projection
boundary. I134 closed with Hook/log negative evidence, real Runtime read-to-edit and denial flows,
bounded collision/concurrency/cross-platform-oriented fixtures, synchronized documentation, and
the full locked validation ladder.
