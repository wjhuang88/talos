# Iteration I134: Model-Private Snapshot-Anchored File Edits

> Document status: Complete
> Published plan date: 2026-07-16
> Planned objective: Deliver compact stale-read-resistant file edits while keeping snapshot
> mechanics out of user-facing and durable history.
> Baseline rule: preserve this target; changed targets use a new iteration ID.
> MVP deliverable: the normal Talos file-tool composition reads model-only `line:hh` anchors and
> executes permission-gated atomic anchored edits with sanitized TUI/TLOG projections.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| TOOL-022 | EXT-002 | Refinement promoted by explicit maintainer implementation request | ADR-039, ADR-042, TOOL-014/015/018 | Runnable snapshot read-to-edit flow with no visible/durable snapshot leakage |

### Scope

- Additive default model/display/persistence projections on `AgentTool`.
- Runtime-memory bounded snapshot registry with full SHA-256 revision and line digests.
- Exactly two model-visible lowercase hexadecimal digits per line.
- Existing `read`/`edit` names in normal CLI/TUI/inline/RPC composition, with legacy edit input
  retained and anchored input added.
- Replace, replace-range, insert-before, insert-after, and delete batch operations.
- Strict full-file revision validation, same-path Talos serialization, staged sync/rename, and
  snapshot invalidation after write/edit/delete.
- Current permission evaluation at edit time using the original path/operation input.
- Negative leakage coverage through events, approvals, returned history, durable messages, and
  TLOG bytes.

### Non-Goals

- No safe reapply or automatic line relocation.
- No persistent/cross-Runtime snapshots.
- No TLOG format, permission default, sandbox, provider protocol, release, or tag change.
- No full external Hashline library/CLI/MCP adoption.
- No promise of linearizable CAS against non-cooperating external writers.

### Acceptance

- Given a normal snapshot-aware `read`
  When the model receives its result
  Then every selected line is `line:hh|content`, `hh` is two hex digits, and no long digest appears.

- Given the same execution
  When UI events, approval data, returned history, durable history, or TLOG are inspected
  Then no snapshot handle or line hash appears.

- Given an unchanged snapshot and allowed anchored operations
  When `edit` executes
  Then the batch commits atomically, preserves untouched line endings, and invalidates stale
  snapshots.

- Given revision drift, wrong hash/path/handle/range, Deny, cancellation, or write failure
  When `edit` executes
  Then the file is unchanged and success is not reported.

- Given a caller uses the legacy tool constructors or legacy edit input
  When it executes
  Then behavior remains compatible.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo check --workspace --locked`
- `cargo clippy --workspace --locked -- -D warnings`
- `cargo test --workspace --locked`
- `./scripts/release_preflight.sh`
- `scripts/validate_project_governance.sh .`
- Real composition fixture through Agent/Runtime and a binary smoke for unchanged CLI startup.

### Documentation To Update

- TOOL-022 owner and proposal
- ADR-045 and decision index
- README/README.zh-CN file-tool behavior
- Board and iteration index

### Risks And Rollback

- Risk: a projection omission leaks model-private data. Rollback: disable shared snapshot registry
  construction; legacy constructors and shared projection defaults retain previous behavior.
- Risk: two-digit collision is mistaken for identity. Mitigation: strict full revision, explicit
  line number, no relocation.
- Risk: atomic replace varies across filesystems. Mitigation: sibling create-new temp, sync,
  permission preservation, rename, structured failure, cross-platform CI fixtures.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-16 | Inventory | No authoritative Active/Review/Planned/Blocked iteration was bypassed. I048-I055 stale Planned headers and I056 stale Review header are historical document drift; the iteration index marks all Complete and the 2026-07-13 inventory explicitly says not to reactivate them. I129-I133 and P100-P150 are Complete. |
| 2026-07-16 | Activation | Maintainer explicitly requested full implementation after reviewing the proposal. TOOL-022 promoted to In Progress and ADR-045 accepted. No release/tag/publish or permission-policy authority inferred. |
| 2026-07-16 | Start Gate | Rust 1.97.0 pinned; locked metadata and governance validation passed. Release preflight reached workspace tests; two dashboard loopback-bind tests failed only because the managed sandbox denied sockets (`Operation not permitted`). Final validation must repeat preflight outside that restriction. |
| 2026-07-16 | Implementation | Added bounded Runtime-memory SHA-256 snapshots, two-hex line anchors, five anchored operations, full-revision validation, same-path serialization, sibling sync/rename replacement, and shared read/write/edit/delete invalidation. Normal CLI/TUI/inline/RPC composition uses the shared registry; legacy constructors remain unchanged. |
| 2026-07-16 | Boundary review | Initial review found that raw tool payloads could reach Hook events. Provider/tool hooks were changed to receive persistence-safe projections; unchanged Hook returns restore private data only for provider/execution. Negative Hook coverage now includes provider messages, proposed/batched calls, permission events, tool results, and completed batches. |
| 2026-07-16 | Closeout | Real Runtime read-to-edit and denied-edit fixtures passed. Full locked fmt/check/clippy/test, socket-capable release preflight, governance validation, binary smoke, and diff validation passed. |

## Verification Evidence

- `snapshot_read_model_projection_is_compact_and_display_projection_is_private`: every model line
  uses exactly `line:hh|content`; handle characters are filename-safe; display/persistence omit all
  anchors while retaining ordinary numbered file content for existing TUI summarization.
- `real_snapshot_read_to_edit_is_atomic_permission_gated_and_never_durable`: a model consumes the
  real read result, constructs an anchored edit, changes the file through an explicit Write allow,
  and proves the handle absent from every SessionEvent, returned message, durable message, and TLOG
  byte stream.
- `denied_real_snapshot_edit_leaves_the_file_unchanged`: the same real flow with headless default
  denial leaves the file byte-identical.
- `model_private_projection_reaches_model_but_not_events_or_returned_history`: model visibility is
  positive while event, returned history, and all provider/tool Hook payloads are negative.
- File fixtures cover five operations, batch overlap, wrong hash/path/registry, stale revision,
  same-snapshot concurrency (exactly one winner), LF/CRLF/mixed terminators, missing/final newline,
  path traversal, Unix symlink escape, bounded registry eviction, oversized fallback, schema
  validity, and two-digit collision with full-revision rejection.
- Context evidence: for 200 numbered lines, two-hex mode adds at most `4N + 32` bytes over the
  existing numbered read and is more than 4x smaller than a 32-hex-per-line alternative in the
  deterministic fixture. This is protocol-size evidence, not a claim about a particular model's
  tokenizer or accuracy.
- Registry hard bounds: 15-minute TTL, 64 snapshots, 2 MiB and 50,000 lines per file, and 16 MiB
  total accounted metadata. Oversized files remain readable without anchors.
- Validation: `cargo fmt --all -- --check`, `cargo check --workspace --locked`,
  `cargo clippy --workspace --locked -- -D warnings`, and `cargo test --workspace --locked` passed;
  socket-capable `./scripts/release_preflight.sh` passed; binary mock smoke passed; governance and
  `git diff --check` passed at closeout.

## Variance And Residuals

- No safe reapply. External non-cooperating writers retain a documented final validate/rename
  TOCTOU window.
- No provider-specific accuracy benchmark was run. The maintainer explicitly selected two hex
  digits and requested implementation; closeout reports deterministic protocol and correctness
  evidence without fabricating model-success claims.
- Native Windows CI is not created by this local uncommitted implementation session. Identifiers
  and sibling temp suffixes are ASCII-safe, and replacement uses Rust 1.97 `std::fs::rename`; the
  repository's existing Windows CI remains the platform execution gate after submission.

## Retrospective

- Model-private state requires projection before every observer, not only before rendering or
  persistence. Treat Hook events as observation surfaces and preserve original payloads only on the
  provider/execution branch.
- A two-digit line code is safe only as a compact check. Full file revision, explicit line number,
  workspace confinement, current permission, and atomic replacement remain the correctness chain.
