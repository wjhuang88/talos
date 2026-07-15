# Iteration I128: Embedded Durable Runtime Sessions

> Document status: Complete
> Published plan date: 2026-07-15
> Planned objective: let an embedded Talos Runtime bind safely to a host-selected durable TLOG session
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a host supplies a directory and external ID, then rebuilds a Runtime with continuous model context and atomic successful-turn persistence.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| DS128-1 | Embedded durable sessions | Ready | I127 Complete | Safe external-ID to UUID binding in a host directory |
| DS128-2 | Embedded durable sessions | Ready | DS128-1 | Atomic, idempotent TLOG turn commit with redaction |
| DS128-3 | Embedded durable sessions | Ready | DS128-2 | `RuntimeBuilder::durable_session` restores context and reports durable IDs |

### Scope

- host-selected directory, UUID TLOG names, external-ID binding, transcript/lifecycle APIs, and durable Runtime integration;
- atomic successful-turn persistence and sensitive-data filtering;
- Rustdoc, ADR, host example, and locked validation evidence.

### Non-Goals

- migration or dual writing of host JSONL; approval audit/artifact/UI state persistence; any change to approval decisions or streaming ordering; durable scheduler work.

### Acceptance

- Given a host directory and `task:<uuid>` external ID, when it opens a durable session twice, then it receives the same UUID-backed TLOG without writing to `~/.talos`.
- Given a successful Runtime turn, when the Runtime is rebuilt from that session, then model context and the normalized transcript continue; failed/cancelled/denied turns leave no partial entries.
- Given secret-like content, raw provider payload, or HTTP authentication material, when persistence runs, then none is present on disk or in the transcript API.

### Planned Validation

- focused `talos-session` and `talos-runtime` tests for binding, atomicity, recovery, transcript, deletion, and redaction;
- `cargo fmt --all -- --check`, `cargo check --workspace --locked`, `cargo clippy --workspace --locked -- -D warnings`, and `cargo test --workspace --locked`;
- embedded runtime integration test using a temporary host directory.

### Documentation To Update

- `README.md`, session/runtime rustdoc, ADR index, iteration index, Board, and this owner doc.

### Risks And Rollback

- Risk: a failed filesystem commit is surfaced after streaming output.
- Rollback: durable binding is optional; unconfigured `RuntimeBuilder` remains the existing in-memory path.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-15 | Activation | I124-I127 are Complete; unrelated PLUGIN-001/CMD-002 remain separately In Progress and are not touched. Existing `SessionManager::with_dir`, TLOG, `initial_history`, and actor persistence were audited. I128 is active for this independent embedded-runtime objective. |

## Verification Evidence

- 2026-07-15 — DS128-1: `SessionManager::with_dir` creates only the host-selected root;
  external IDs are SQLite logical keys and UUID TLOG filenames are used instead. Focused tests
  prove colon-bearing IDs, repeated open, concurrent open, and traversal rejection.
- 2026-07-15 — DS128-2: `DurableSession::commit_turn` writes a full filtered turn through a
  synced temporary sibling and atomic rename. Tests prove retry idempotency, abort-with-no-entry,
  deletion/index cleanup, tool transcript projection, policy output omission, and structured write
  failure. Credential cases cover Authorization/Bearer, API keys, Cookie, and token-like values.
- 2026-07-15 — DS128-3: `RuntimeBuilder::durable_session` restores history automatically and
  emits `SessionEvent::EntriesCommitted` only after commit. The runtime integration test proves
  committed entry IDs and context recovery after rebuild; unbound builder behavior remains covered
  by the existing workspace suite.
- 2026-07-15 — locked validation passed: `cargo fmt --all -- --check`, `cargo check --workspace
  --locked`, `cargo clippy --workspace --locked -- -D warnings`, and `cargo test --workspace
  --locked`. A pre-existing parallel-test collision in `workspace_trust` was repaired by giving
  its tests isolated temporary roots; the subsequent full workspace run passed.

## Variance And Residuals

- Existing generic session append remains legacy behavior; I128 durable Runtime uses its new atomic turn path only.
- `begin_turn` is intentionally non-durable and `abort_turn` is a no-write acknowledgement:
  user messages are committed only with a successful completed turn. This makes crash, denial,
  interruption, and provider failure recovery unambiguous: no partial model transcript exists.
- The binding index is colocated SQLite metadata, not a second transcript format; TLOG remains the
  sole new model-history format. No host JSONL migration or dual write is provided.

## Retrospective

- Closeout: the smallest host contract is a stable external ID plus a host-selected directory.
  Approval audit, artifacts, provider conversation IDs, and transient UI state remain host-owned.
  No permission, approval bridge, tool authorization, or streaming-progress semantics were changed.
