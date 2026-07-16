# TOOL-022: Model-Private Snapshot-Anchored File Edits

**Type**: Product/API/State Story
**Status**: Complete — I134 (2026-07-16)
**Priority**: P2
**Parent research**: EXT-002
**Source**: User request 2026-07-16; Kilo Code issue #11492
**Iteration**: I134

## Identity / Goal / Value

**Recipient**: Coding-model users and embedded Runtime hosts.

Let a model read compact two-hex-digit line anchors and submit precise file edits against a bounded
Runtime-memory snapshot, while users see normal read/edit summaries and diffs rather than internal
snapshot mechanics. This should reduce fragile first-string replacement without inflating context
with long hashes or weakening file permissions.

## Scope

- A bounded, Runtime-local, non-persistent snapshot registry.
- Exact file revision plus internal full per-line digests.
- Exactly two lowercase hexadecimal digits per model-visible line anchor.
- Model-private snapshot handle and hash annotations.
- Separate model, display, and persistence projections for tool input/results.
- Typed replace, replace-range, insert-before, insert-after, and delete operations.
- Strict Phase 1 full-file revision rejection; no automatic reapply.
- Normal workspace confinement and write approval at execution time.
- Same-directory staged atomic write with explicit failure reporting.
- Compatibility-preserving, optional tool/backend delivery.
- Comparative token/reliability evidence before any default change.

## Exclusions

- No automatic relocation using a two-digit hash.
- No persistent snapshots or cross-Runtime handle recovery.
- No permission grant encoded in a snapshot handle.
- No unified-diff grammar requirement for models.
- No replacement/removal of the current `read` or `edit` contract in the first slice.
- No arbitrary script hook, global message bus, multi-agent coordination, or TLOG format change.
- No adoption of a full external Hashline CLI/MCP stack in the first slice.
- No claim of atomic compare-and-swap against non-cooperating external processes.

## Dependencies

- TOOL-014 conditional tool presentation, if the capability uses an on-demand backend.
- ADR-039 single-flow event and session-writer boundary.
- ADR-042 durable Runtime session boundary; requires an accepted transient model-only projection
  amendment or successor ADR before implementation.
- TOOL-015/TOOL-018 bounded write/edit diff presentation.
- Current workspace confinement and permission pipeline.

## Decision Links And Constraints

- Two hex digits are a compact check code, not identity or authority.
- Full file revision and full line digests remain in bounded Runtime memory.
- Renderer-only suppression is insufficient. Projection must occur before event broadcast, raw
  capture, and Session/TLOG commit.
- The session actor remains the only durable writer; no secondary persistence sink is allowed.
- All existing unconfigured Runtime and file-tool behavior must remain compatible.
- Adding fields to public input structs or changing public event/message types requires semver and
  migration review under AGENTS.md Hard Constraint #6.

## Uncertainty And Validation Path

The open architecture question is how to carry a model-only tool payload while preserving current
public APIs and ADR-042 replay semantics. Resolve it with an ADR and a narrow prototype/fixture that
proves the handle appears in provider context but nowhere in TUI/history/TLOG. Registry limits and
the optional tool/backend shape must be selected from benchmark evidence, not guessed into a
production default.

## State / Status Owners

- This owner story controls readiness and implementation status.
- `docs/proposals/model-private-snapshot-anchored-file-edits.md` controls the current design and risk
  inventory.
- A future ADR controls model/display/persistence projection semantics.
- A future iteration file controls activation and validation evidence.
- Any residual safe-reapply work receives a new story and may not be folded into Phase 1.

## User-Facing Documentation

- File-tool documentation must explain stale snapshot recovery without exposing internal handles.
- Runtime SDK documentation must explain that handles are memory-only and invalid after rebuild.
- Security documentation must state that snapshot validity never bypasses write approval.
- Release notes must say existing file-tool callers require no change while the feature is optional.

## Required Reads

- `docs/proposals/model-private-snapshot-anchored-file-edits.md`
- `docs/backlog/active/EXT-002-oh-my-pi-feature-analysis.md`
- `docs/decisions/039-runtime-event-semantic-single-flow.md`
- `docs/decisions/042-embedded-durable-runtime-session-boundary.md`
- `docs/backlog/active/TOOL-014-conditional-tool-backends.md`
- `docs/backlog/active/TOOL-015-write-edit-result-visibility.md`
- `docs/backlog/active/TOOL-018-diff-output-and-rendering.md`
- `crates/talos-core/src/tool.rs`
- `crates/talos-core/src/message.rs`
- `crates/talos-agent/src/tool_execution.rs`
- `crates/talos-agent/src/session/turn.rs`
- `crates/talos-session/src/durable.rs`
- `crates/talos-tools/src/file_tools/`
- `crates/talos-tui/src/tool_display.rs`

## Acceptance For Behavior

- Given a snapshot-aware read of a supported text file
  When the provider receives the tool result
  Then each selected line has a `line:hh|content` anchor and `hh` is exactly two lowercase hex
  digits, while no long digest is provider-visible.

- Given the same read event
  When any TUI, approval, export, transcript, dashboard, RPC, TLOG, raw metadata, log, error, or hook
  diagnostic surface is inspected
  Then neither the handle nor any line hash is present.

- Given a valid handle, unchanged full file revision, and valid non-overlapping operations
  When the model requests an anchored edit and current permission allows it
  Then the complete mutation is written atomically, a bounded user-visible diff is emitted, and all
  snapshots for the path are invalidated.

- Given any file revision change, wrong/expired handle, hash mismatch, invalid range, permission
  denial, cancellation, or write failure
  When an anchored edit is requested
  Then no mutation or false success is produced and the model receives a structured recovery code.

- Given the capability is disabled or absent
  When an existing caller uses `read`, `write`, `edit`, or `delete`
  Then its schema, permission semantics, output, and persistence behavior remain unchanged.

## Acceptance For Technical / Governance Work

- [x] ADR-045 defines transient model-only tool context and reconciles ADR-042.
- [x] Public API additions have rustdoc and a migration/compatibility statement.
- [x] Dependency review selects workspace-resolved pure-Rust `sha2`; no new package and no
      `talos-runtime`/`talos-session` cycle is introduced.
- [x] Deterministic reference/collision fixtures cover parser rejection and prove a two-digit
      collision cannot bypass the full revision. A new property-test dependency was not needed.
- [x] Cross-platform-oriented fixtures cover LF, CRLF, mixed terminators, trailing/missing newline,
      path traversal, symlink escape, bounded failure paths, and ASCII-safe IDs/temp suffixes.
- [x] End-to-end fixtures prove model-visible/user-hidden/Hook-hidden projection, an allowed real
      read-to-edit flow, and permission denial with no mutation.
- [x] Comparative deterministic evidence reports context bytes, protocol success/rejection,
      stale/wrong-target behavior, bounded test latency, and registry cardinality/accounted-payload
      caps. It makes no unsupported provider-tokenizer or real-model accuracy claim.
- [x] `cargo fmt --all -- --check`, locked workspace check/clippy/test, socket-capable release
      preflight, governance validation, and `git diff --check` pass.
- [x] Owner story, iteration, Board, backlog, README/user docs, ADR/proposal indexes are synchronized.
- [x] Safe reapply remains disabled and outside I134.

## Readiness Gate

The maintainer explicitly authorized full implementation on 2026-07-16. I134 is complete with real
Runtime composition, permission-denial, Hook/event/history/TLOG leakage negatives, collision and
concurrency fixtures, and the full validation ladder. Normal composition uses the capability;
legacy constructors remain the compatibility and rollback path.
