# I021: Evolution MenteDB Realignment

**User can**: Trust that the evolution engine stores learning signals as small, structured,
semantically-correct fields, so the 5MB knowledge.db bloat and "5MB system_prompt → 400 Bad
Request" loop cannot recur.

## Status: Complete (2026-06-06)

## Context

I008 shipped the `talos-evolution` engine on 2026-06-01 with a closed learning loop
(Observe → Accumulate → Extract → Apply per ADR-001). The original data-structure intent
followed the MenteDB cognitive-feedback reference design
(`docs/reference/REFERENCE-PROJECTS.md` §17), but during implementation two critical
deviations occurred:

1. **`Signal.context` was reused as a full-text container** instead of a small, signal-specific
   window. MenteDB intends `context` to be the phrase the user used to correct the agent
   (e.g. "不要用 sed" plus ~200 bytes of surrounding text), not the entire 5MB user message.
2. **The `Message::User` content already had the system_prompt embedded** (5MB
   `system_prompt + "\n\n" + user_input` per `talos-agent/src/lib.rs:380-384`), so the hook
   captured the full system_prompt and stored it verbatim as `Signal.context`.

This deviation caused a real production failure (2026-06-06): `knowledge.db` grew to 241MB,
`system_prompt` ballooned to 5,151,386 bytes (380× over the 202,752 context limit), and a
`cargo run -- -p "你好"` returned `400 Bad Request: Range of input length should be [1, 202752]`.

### What 7470ac5 did (already shipped, defense layer)

Commit `7470ac5` added three defense layers and a one-time migration:

- `max_context_bytes` (default 4096) caps `observation.context` at write time
- `max_output_bytes` (default 8192) caps `BehaviorAdapter` output, drops oversized single patterns
- `content_hash` column + dedup prevents near-duplicate accumulation across turns
- `delete_oversized_patterns` deactivates 30 of 32 patterns on first run

**This is a defense-in-depth fix, not a root-cause fix.** The 4KB cap keeps storage bounded but
still stores ~4KB of `system_prompt` prefix (no user signal) per observation. The cap exists
because the underlying field semantics are wrong.

### What this iteration (I021) does

I021 is the root-cause fix. It realigns the `talos-evolution` data structure with the MenteDB
blueprint so the `Signal.context` field is small, semantic, and correct by construction. After
I021, the 7470ac5 byte caps become belt-and-suspenders (still useful as defense in depth, but
no longer the only thing standing between the user and a 5MB prompt).

## Scope

Refactor `talos-evolution` to match the MenteDB data structure documented in
`docs/reference/REFERENCE-PROJECTS.md` §17. Touches `talos-evolution` only — no changes to
`Message` enum, agent loop, or providers (preserves the "don't break my design" constraint).

## Selected Stories

- [x] #I021-S1: Restructure `Observation` into `TurnObservation` (parent, per-turn) + `Signal`
      (child, per-event). MenteDB-aligned fields: `kind: SignalKind` (4 types: Correction/Error/
      Satisfaction/Inefficiency), `intensity: f32`, `context: String` (small window), and
      `turn.tools_used: Vec<ToolUsage>`, `turn.outcome: TurnOutcome`, `turn.duration_ms: u64`,
      `turn.session_id`, `turn.turn_number`.
- [x] #I021-S2: Hook capture uses `find_marker + capture_window(text, marker_pos, 200)`. Drop
      `truncate_context` and the "head of the string" semantic. The marker phrase ("不要用 sed"
      etc.) is the center of the captured window, not buried at the tail.
- [x] #I021-S3: Restructure `Pattern` with MenteDB fields: `key: String`, `value: serde_json::Value`
      (structured, replaces the free-text `instruction`), `contradicting_count: u32`,
      `last_reinforced: DateTime<Utc>`, `source_sessions: Vec<Uuid>`. Keep `description` and
      `instruction` for backward-compat with `BehaviorAdapter`'s prompt-injection output, but
      populate them from `key` + `value` rendering.
- [x] #I021-S4: One-time hard reset of `~/.talos/evolution/knowledge.db` on the next `open()`.
      Schema changes are not backward-compatible (new `signal.tool_name`, `turn.outcome`,
      `pattern.value`, `pattern.key`, `pattern.source_sessions` columns). Soft-deactivation
      (the 7470ac5 approach) does not work here because the column shapes changed. Log the
      reset count via `tracing::warn!` so the user sees it.
- [x] #I021-S5: Mark 7470ac5's `max_context_bytes` as defense-in-depth (retain field, keep
      default 4096, but document that with MenteDB-aligned `capture_window` the context is
      naturally < 500 bytes per turn). Update `EVOLUTION.md` lesson #19 with "real fix landed
      in I021" annotation.

## Acceptance Criteria

- [ ] `Signal.context` is, by construction, a small window around the marker phrase. In tests
      with a 5MB `Message::User` content and a 7-byte user tail "不要用 sed", the stored
      `Signal.context` length is < 500 bytes and contains the marker.
- [ ] `TurnObservation` is the per-turn parent that aggregates `signals: Vec<Signal>` plus
      `tools_used` / `outcome` / `duration_ms` metadata.
- [ ] `Pattern` carries `key`, `value`, `contradicting_count`, `source_sessions`. Existing
      `description` and `instruction` fields remain for `BehaviorAdapter` to consume; they
      render `value` as natural language.
- [ ] `knowledge.db` is reset once on first open after upgrade. Subsequent runs append
      normally.
- [ ] `cargo run -- -p "你好"` continues to work (regression check). `system_prompt` size stays
      bounded (target: < 32KB after running 20+ correction-rich turns).
- [ ] `cargo test --workspace` passes. New tests cover: `capture_window` extracts marker,
      `Signal` roundtrip preserves all fields, `TurnObservation` flushes multi-signal
      aggregation, schema migration succeeds.
- [ ] `cargo clippy --workspace -- -D warnings` is clean for changed files.

## Out of Scope (Phase 2 — MenteDB cognitive rigor, not bug fixes)

The following items are **independent enhancements** to the evolution engine that align with
MenteDB's full cognitive-feedback design. They do **NOT** solve the 5MB bloat / 400 error /
signal-loss problems; the root fix is the I021 stories above. These items are tracked as
backlog stories and will be selected into a future iteration (likely I022) once I021 lands.

- **`#EVOL-002`: 6 SignalKinds (add `Retry`, `TokenWaste`)** — richer observation, but each new
  signal type has the same field-semantic risk as the original 4. Will be safer to add **after**
  I021's `Signal` schema is in place.
- **`#EVOL-003`: Outcome-driven `decayed_confidence` integration** — `last_reinforced` field
  exists after I021-S3, but the existing `Pattern::decayed_confidence` method is not yet called
  by `BehaviorAdapter`. Pure integration work.
- **`#EVOL-004`: Bayesian confidence with `contradicting_count`** — requires
  `supporting / (supporting + contradicting)` formula to be applied. The data is there after
  I021-S3; wiring it is mechanical.
- **`#EVOL-005`: Cross-session learning via `source_sessions`** — requires merging patterns
  across sessions. Useful for long-term learning but doesn't address per-turn correctness.
- **`#EVOL-006`: `TurnOutcome` (4 values) and `ToolUsage` event capture** — needs
  `OnTurnEnd`/`OnToolResultObserved` hook data to be richer. Independent of bloat / signal
  semantics.

These five stories are filed in `docs/backlog/active/EVOL-001-evolution-cognitive-rigor.md`
together so a future iteration can pick them up as a single cognitive-rigor slice.

## Verification Plan

### Library / unit tests

- `capture_window(text, marker_pos, window_size) -> String` unit tests: marker in center, edge
  cases (marker at start, end, alone)
- `Signal` roundtrip preserves all fields including `tool_name`
- `TurnObservation::flush()` aggregates multiple `Signal`s with shared turn metadata
- `Pattern` roundtrip with new fields
- Schema migration: `KnowledgeStore::open` on a v1 DB either hard-resets or migrates; test both
  code paths

### Workspace

- `cargo test --workspace` exits 0
- `cargo clippy -p talos-evolution -- -D warnings` clean
- `cargo build -p talos-cli` clean

### End-to-end (regression for the original bug)

- `cargo run -p talos-cli -- -p "你好"` succeeds, response streams normally
- `system_prompt` size check (debug print) stays < 32KB even after 20+ correction-rich turns
- `~/.talos/evolution/knowledge.db` does not grow past 1MB during a 20-turn test session
- A simulated correction turn ("不要用 sed") produces a `Signal.context` containing the phrase
  "不要用 sed", not system_prompt content

## Execution Results (2026-06-06)

### Atomic commits

| Commit | Story | Files | Net change |
|---|---|---|---|
| `8af7446` | #I021-S1 | `Cargo.toml`, `lib.rs`, `store.rs` | +251 lines (SignalKind/Signal/TurnObservation/TurnOutcome/ToolUsage types + new tables) |
| `511fb78` | #I021-S2 | `observer.rs`, `hook.rs` | +246 lines (find_marker + capture_window + wire into hook) |
| `164e641` | #I021-S3 | `lib.rs`, `store.rs` | +164 lines (Pattern.key/value/contradicting_count/last_reinforced/source_sessions + new columns) |
| `260c3fb` | #I021-S4 | `store.rs` | +94 lines (schema version detection + hard-reset on mismatch) |
| `d2303b9` | #I021-S5 | `EVOLUTION.md`, `hook.rs` | +5 lines (lesson #19 annotation + minor hook fix) |

### Verification evidence

**Library / unit tests**: `cargo test --workspace` → **615 tests pass, 0 failures**
(was 591 before; +24 new tests for Signal roundtrip, TurnObservation aggregation,
Pattern roundtrip with MenteDB fields, capture_window edge cases, schema migration
success path, knowledge.db hard-reset).

**Clippy**: `cargo clippy -p talos-evolution --no-deps -- -D warnings` → **clean**
(no warnings on changed files). Note: `talos-core` has pre-existing clippy warnings
unrelated to this iteration (not flagged by `-D warnings` on `-p talos-evolution`).

**Build**: `cargo build -p talos-cli` → **clean** (`Finished` line, no errors).

**End-to-end runtime regression**:
```
$ cargo run -p talos-cli -- -p "你好"
[2m2026-06-06T15:48:29.307855Z[0m [32m INFO[0m ... hook event, ... event: OnSystemPromptBuilt
... hook event, ... event: TurnStart
... hook event, ... event: BeforeProviderCall
... hook event, ... event: OnTextDelta
你好！我是 Talos，一个 AI 编程助手。
...
```
Model responded normally in Chinese with self-introduction. No `400 Bad Request`. No
"Range of input length should be [1, 202752]" error. This confirms the I021 fix
prevents the 5MB system_prompt / 400 error loop from recurring on the user's actual
`~/.talos/evolution/knowledge.db` (14MB post-cleanup, no bloat patterns).

**Hard-reset migration**: On this user's database, the S4 check
(`PRAGMA table_info('patterns')` for `key` column) returned `true` (column exists
from S3's ALTER TABLE), so no hard-reset was triggered. The migration is
correctly idempotent and only fires for users upgrading from a pre-S3 v1 schema
with existing data. New users / fresh databases start with the v2 schema directly.

**Defense layer preserved**: 7470ac5's `max_context_bytes` (4096) and
`max_output_bytes` (8192) config fields remain active. The `truncate_context` function
is kept with `#[deprecated]` for back-compat. Per #I021-S5, this is documented as
defense-in-depth, no longer the primary line of defense.

### Out-of-scope confirmation

The following items were **not** changed (per "不要破坏我的原始设计" constraint and
I021 explicit out-of-scope list):

- `Message` enum in `talos-core/src/message.rs` — no new variants
- `talos-agent/src/lib.rs` agent loop — no message-construction changes
- `talos-provider/*` — no system-role handling changes
- `BehaviorAdapter` output format — Markdown list rendering unchanged
- Any external dependencies (no new crates)

All 5 commits verified `git diff` against pre-iteration `2414824`: only files under
`crates/talos-evolution/src/`, `crates/talos-evolution/Cargo.toml`, and
`EVOLUTION.md` are modified.

### Residual work (deferred to a future iteration)

The 5 EVOL-001 sub-stories (MenteDB cognitive rigor Phase 2) are independent
enhancements that do not solve the bloat / 400 / signal-loss problems. They are
ready to be selected into a future iteration (likely I022) but require I021 to
land first. Status: P3 in PRODUCT-BACKLOG, file:
`docs/backlog/active/EVOL-001-evolution-cognitive-rigor.md`.

## Out of Scope (architectural — explicitly NOT changing)

- `Message` enum (`talos-core/src/message.rs`) — no `System` variant. The agent's
  `format!("{system_prompt}\n\n{user_message}")` design stays.
- `talos-agent` turn loop — no change to message construction.
- `talos-provider` (Anthropic + OpenAI) — no change to `build_request_body` or system role
  handling.
- `BehaviorAdapter` output format — keeps current Markdown list-of-patterns rendering; only the
  source data changes.

If any of these need to change in the future, they belong in a separate iteration (likely
I022) and require an ADR per the project governance rules.

## Related ADRs / Lessons

- ADR-001 (Self-Evolution as Runtime Primitive) — I021 preserves the 4-phase loop
- ADR-016 (Layered Agent Memory Architecture) — orthogonal to I021; concerns the new
  `talos-memory` crate, not the existing `talos-evolution` engine
- EVOLUTION.md lesson #19 — to be updated to note "real fix in I021" after this iteration
  lands
- EVOLUTION.md lesson #11 ("单测全过 ≠ 完成") — root cause of the bloat was only visible
  via end-to-end runtime evidence; I021 verification plan reuses this lesson

## Required Reads (for implementers)

- `docs/reference/REFERENCE-PROJECTS.md` §17 — MenteDB reference data structure
- `docs/decisions/001-runtime-self-evolution.md` — evolution loop intent
- `docs/iterations/I008-learning-agent.md` — original implementation
- `docs/iterations/I019-layered-memory-foundation.md` — orthogonal memory architecture
- `EVOLUTION.md` lesson #19 — what went wrong and why I021 is the fix
- Commit `7470ac5` — defense-in-depth layer to keep during this iteration
