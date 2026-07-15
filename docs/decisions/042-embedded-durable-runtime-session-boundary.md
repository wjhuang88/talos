# ADR-042: Embedded Durable Runtime Session Boundary

> Status: Accepted
> Date: 2026-07-15

## Context

Embedded hosts need Talos-owned, TLOG-only model transcript persistence under a host-selected
directory. The existing session store supports custom directories and TLOG, but runtime wiring is
manual, writes messages one by one, and can retain raw tool output metadata.

## Decision

`talos-session` owns durable session files, the external-ID binding index, turn atomicity,
redaction, lifecycle queries, and normalized transcript projection. A binding stores an opaque
external ID only as a SQLite value; it always maps to a randomly generated UUID TLOG filename.

A durable turn is not persisted at begin. On success, all model-visible messages for that turn are
filtered, encoded with stable entry IDs, written with the existing log plus the full new turn to a
temporary sibling TLOG, synced, and atomically renamed. A repeated `turn_id` returns the already
committed IDs. Failed, interrupted, denied, and uncommitted turns leave no durable messages.

`talos-runtime` may depend directly on `talos-session`, preserving the existing direction
`runtime -> agent -> session -> core`; no shared trait is needed and no cycle is introduced.
`RuntimeBuilder::durable_session` is optional and leaves the unbound runtime path unchanged. Its
successful completion carries committed IDs only after the session store confirms the rename.

The durable projection excludes provider raw responses, headers, credentials, and `raw_content`.
Reasoning is opt-in; tool results use the model-visible, redacted representation, not a separate
raw-output channel. Host approval audit, artifacts, provider conversation IDs, and temporary UI
state remain outside Talos transcript storage.

## Consequences

- Host applications persist only their stable external ID and choose the root directory; Talos owns
  the UUID and binding index.
- Atomic turn replacement trades write amplification for simple crash recovery and no partial
  successful turn records. Future segment-aware commits may replace this mechanism behind the same
  API.
- Existing generic Session APIs and JSONL compatibility remain available but are not used for new
  embedded durable sessions.

## Reversal Trigger

Revisit when transcript sizes make full-file turn replacement materially expensive under measured
desktop workloads, or when multi-process host access requires an explicit cross-process lock.
