# EVOL-001: Evolution Cognitive Rigor (MenteDB Phase 2)

## Outcome

`talos-evolution` exercises the full MenteDB cognitive-feedback design (6 SignalKinds,
Bayesian confidence, time decay, cross-session provenance, outcome tracking) on top of the
MenteDB-aligned data structure delivered by I021.

## Status

Planned. Will be selected into I022 (or a successor cognitive-rigor slice) after I021
lands and stabilizes.

## Priority

P3. Lower than the I021 root-cause fix because these items are independent enhancements that
do not solve the 5MB bloat, `400 Bad Request`, or signal-loss problems I021 addresses.

## Why these are NOT pre-requisites for I021

The five stories below are **cognitive rigor** improvements to the evolution engine. They
add quality, not correctness. Specifically:

- They do **not** bound storage size (I021 does that via `Signal.context` window capture).
- They do **not** prevent the 400 Bad Request failure mode (I021 does that by separating the
  system_prompt prefix from the user signal).
- They do **not** recover signals lost to head-truncation (I021 does that by capturing
  around the marker).

Each story below assumes I021's data structure is in place. Adding them before I021 would
re-introduce the same field-semantic risk on the new signal types (Retry, TokenWaste).

## Stories

Each story is small enough to be selected independently into a future iteration. They are
grouped under one backlog entry because they share the same architectural foundation (I021
schema) and the same verification surface.

### #EVOL-002: Add `Retry` and `TokenWaste` to `SignalKind`

- `Retry`: detected when the same tool name is called twice within one turn (or N turns).
  `intensity` = `0.5` baseline. Suggests the first tool choice was insufficient.
- `TokenWaste`: detected when `usage.input_tokens + output_tokens` exceeds a configurable
  threshold per turn (e.g., 16K). `intensity` = `(tokens - threshold) / threshold`, clamped
  to `[0, 1]`. Suggests the prompt assembly or tool execution was inefficient.
- Both new signals must use the I021 `capture_window` capture logic. They cannot reuse the
  pre-I021 `truncate_context` because the same head-vs-tail bug would reappear.
- Pattern category for `Retry` is `tool_choice_correction`; for `TokenWaste` is `efficiency`.

### #EVOL-003: Outcome-driven `decayed_confidence` integration

- `BehaviorAdapter::get_evolution_context` currently sorts by raw `confidence`. Switch to
  `decayed_confidence(half_life_days)`.
- Compute once at read time (cached in the query, not persisted) so the `last_reinforced`
  timestamp drives decay without explicit rebalancing.
- Add a config flag `enable_time_decay: bool` (default `true`) so users can disable decay
  during debugging.

### #EVOL-004: Bayesian confidence with `contradicting_count`

- Replace `pattern.confidence = (pattern.confidence + avg_intensity) / 2.0`
  (`PatternExtractor::merge_evidence`) with the MenteDB formula:
  `confidence = supporting_evidence / (supporting_evidence + contradicting_evidence)`.
- Detecting a contradiction is not implemented in I008. This story adds a minimal detector:
  when a new observation's signal type is `Correction` and it contradicts an existing
  `preference` pattern (e.g., the user says "no, use sed" after a pattern said "prefer
  awk"), increment the existing pattern's `contradicting_count` instead of inserting a new
  one.
- Add a `ConflictResolution` strategy enum (`Override`, `KeepBoth`, `IncreaseUncertainty`)
  with the resolution heuristic from MenteDB §17. `AskUser` is out of scope for now (no
  TUI surface to surface conflicts).

### #EVOL-005: Cross-session learning via `source_sessions`

- The I021-S3 schema already includes `source_sessions: Vec<Uuid>`. This story wires it up:
  every `Pattern` insert/update appends the current `session_id` to
  `source_sessions` (dedup).
- `KnowledgeStore::get_active_patterns` no longer filters by `session_id`. Patterns persist
  across sessions and accumulate evidence from any session that triggers them.
- Add a `--learned --since-session <uuid>` CLI flag to limit display to patterns that include
  the specified session (optional UX surface; can be deferred to TUI).

### #EVOL-006: `TurnOutcome` and `ToolUsage` event capture

- `TurnObserver` records `outcome: TurnOutcome` at `OnTurnEnd` based on
  `AgentEvent::TurnEnd { stop_reason }`: `EndTurn` → `Success`, `ToolUse` →
  `PartialSuccess`, `MaxTokens` → `Failed`. `UserAbandoned` is detected from the
  `CancellationToken` path (the `run_inner` already has a `cancelled` branch).
- `tools_used: Vec<ToolUsage>` is recorded from `OnToolResultObserved` events. Each
  `ToolUsage` carries `{ name: String, duration_ms: u64, is_error: bool }`.
- These are stored on `TurnObservation` (parent) and are not turned into `Signal`s unless a
  separate signal type is added (e.g., a future `ToolError` signal). They are metadata for
  observability, not learning inputs (yet).

## Acceptance Criteria (whole backlog entry)

- [ ] All five stories pass their per-story acceptance criteria
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace -- -D warnings` is clean
- [ ] `BehaviorAdapter` output reflects `decayed_confidence` and Bayesian formula
- [ ] `~/.talos/evolution/knowledge.db` remains < 1MB during a 50-turn stress test
- [ ] New SignalKinds (`Retry`, `TokenWaste`) appear in `--learned` output when triggered
- [ ] `TurnOutcome` recorded for every turn (verified via `cargo run --mock -p "..." --show-outcome` debug flag or equivalent)

## Required Reads (for implementers)

- `docs/iterations/I021-evolution-mentedb-realignment.md` — pre-requisite data structure
- `docs/reference/REFERENCE-PROJECTS.md` §17 — MenteDB cognitive feedback design
- `docs/decisions/001-runtime-self-evolution.md` — evolution loop intent
- `EVOLUTION.md` lesson #19 — the bloat incident that motivated I021 and this backlog

## Residual Work Destination

- If I021 changes schema in a way that this backlog becomes inconsistent, re-derive these
  stories from the new schema before selection. Do not blindly port.
- Conflict resolution `AskUser` strategy needs a TUI surface; defer until I014 TUI
  completion lands.
