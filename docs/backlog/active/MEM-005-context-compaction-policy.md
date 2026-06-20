# MEM-005: Context Compaction Trigger And Runtime Policy

| Field | Value |
|-------|-------|
| Story ID | MEM-005 |
| Priority | P2 |
| Status | Planned |
| Depends On | MEM-002; MEM-003 for full LLM layers |
| Origin | User feedback 2026-06-18 — context compaction needs explicit trigger and runtime logic |

## Problem

Talos has context compaction mechanisms, but the product-level policy is not
explicit enough:

- when compaction should trigger
- whether compaction runs before a model call, after a turn, or manually
- how automatic compaction interacts with user-visible session history
- how failures degrade without losing useful context
- how the user can see that compaction happened and what it changed

Without a first-class policy, compaction risks becoming an implicit
implementation detail that is hard to reason about during long sessions.

## Scope

Define and implement a deterministic compaction policy for active sessions.

Required policy decisions:

- Trigger thresholds based on provider context window, estimated token usage,
  tool-result pressure, and reserved output budget.
- Model metadata source precedence for context/output limits, including explicit
  user config, refreshed catalog cache, built-in model data, and conservative
  fallback.
- Reserved reasoning/thinking budget when the active model uses hidden or
  interleaved reasoning tokens.
- Pre-turn compaction behavior before sending messages to the provider.
- Post-turn maintenance behavior after large tool outputs or long assistant
  responses.
- Manual compaction command behavior, including whether `/compact` is allowed
  during an active turn.
- Register `/compact` as a CMD-001 BuiltinCommand that delegates a typed operation to the session
  compaction owner rather than mutating conversation display state.
- User-visible status: compact notification, before/after token estimate, and
  failure reason when compaction is skipped.
- Persistence semantics: compacted context sent to the provider may differ
  from raw session history, but raw session records must remain recoverable.
- Interaction with hidden tool output: compaction may summarize hidden content
  for model context, but must not reveal hidden content into TUI scrollback.

## Relationship To MEM-003

MEM-003 wires LLM-based compaction layers 4-5 and proves long-session bounded
context behavior. MEM-005 defines the runtime policy around compaction:
triggers, ordering, manual controls, observability, and degradation behavior.

Implementation may land in either order if scoped carefully:

- MEM-005 can first formalize policy around existing layers 1-3.
- MEM-003 is needed before the policy can use full LLM summarization.

## Relationship To MODEL-001

MODEL-001 supplies the model metadata needed by this policy: context window,
output limit, pricing, and reasoning/thinking capability. MEM-005 should not
hardcode a single default context limit when catalog or user-configured limits
are available.

## Acceptance Criteria

- [ ] Compaction trigger thresholds are documented and configurable through
      provider/model limits where available.
- [ ] Trigger calculation reserves output budget and reasoning/thinking budget
      when the active model requires it.
- [ ] Limit source precedence is documented: user config, refreshed catalog,
      built-in model dataset, then fallback.
- [ ] Pre-turn compaction runs before provider calls when context pressure
      exceeds the threshold.
- [ ] Post-turn maintenance can mark the session as needing compaction after
      large tool outputs or long responses.
- [ ] Manual compaction is exposed through a user command or equivalent UI
      action.
- [ ] The TUI displays a compact, non-persistent status line when compaction
      runs, succeeds, is skipped, or fails.
- [ ] Raw session history remains recoverable; compacted provider context does
      not replace the durable source of truth unless an explicit future ADR
      changes that boundary.
- [ ] Hidden tool output is never printed into history as a side effect of
      compaction.
- [ ] Compaction failures do not abort the user turn; the session continues
      with a safe fallback or a clear refusal if the context cannot fit.
- [ ] Tests cover threshold decisions, pre-turn ordering, manual compaction,
      skipped compaction, and failure fallback.

## Smart Compaction Strategy (Research 2026-06-20)

The current `should_compact()` uses a purely mechanical 80% token threshold.
This can fire mid-task, losing critical intermediate tool results. A smarter
scheduler should detect natural conversation boundaries first.

### Available Signals (already in the codebase)

| Signal | Source | How to use |
|---|---|---|
| `StopReason::EndTurn` vs `ToolUse` | AgentEvent per turn | `EndTurn` after tool-call sequence = natural boundary; `ToolUse` = mid-task, don't compact |
| Tool call count per turn | `turn_tool_calls.len()` | 0-call turns (pure text) are safer compaction targets than 5+ call turns |
| Tool call names/patterns | `call.name` per ToolCall | Read-only burst → write/edit burst = phase transition boundary |
| Token usage delta | `Usage` per turn | Large input spike (read_files) followed by output = task completed |

### Phased Implementation

**Phase 1 — Boundary-Aware Trigger (low complexity)**:
- Before compacting, check if current turn is `EndTurn` with 0 pending tool calls
  → if yes, this is a natural boundary, safe to compact.
- If `ToolUse` → defer compaction by 1 turn unless hard overflow.
- Track tool-call density: turns with 0 tool calls can be compacted more aggressively
  than turns with 3+ tool calls (current task context).

**Phase 2 — Tool-Call Counter (low complexity)**:
- Track cumulative tool calls per session, signal at 50/75/100:
  - 50: "Consider /compact if transitioning phases"
  - 75: "Good boundary for /compact"
  - 100: "Strongly recommend /compact before next task"
- Phase transition detection: exploration (read/grep/glob) → implementation
  (write/edit/bash) shift signals a natural compaction boundary.

**Phase 3 — Agent Self-Triggered (medium complexity)**:
- Expose `compact_conversation` as an agent-callable tool with an eligibility gate
  (only available when context > 50% of threshold).
- Model decides when prior context is no longer needed, based on system prompt
  guidance about "good times to compact."
- The tool is NOT granted by default; user opt-in via config.

**Phase 4 — Task Completion Checkpoint (medium complexity, future)**:
- `mark_task_done` tool: model explicitly declares task complete.
- Writes structured record: intent, files modified, decisions, errors, next steps.
- Subsumes compaction at the boundary — checkpoint IS the compaction artifact.

### What NOT to do

- Embedding-based semantic detection: overkill for a coding agent where
  tool patterns + stop reasons are more reliable signals.
- Full 5-tier Claude Code cascade: Talos already has 5 layers; the gap is
  *when* to trigger, not *how* to compact.
- ML-based boundary classification: requires training data, adds complexity
  disproportionate to benefit.

- `docs/backlog/active/MEM-002-conversation-context-continuity.md`
- `docs/backlog/active/MEM-003-llm-compaction.md`
- `docs/backlog/active/MODEL-001-model-catalog-and-reasoning.md`
- `docs/backlog/active/TUI-009-input-and-session-exit-polish.md`
- `docs/backlog/active/CMD-001-interactive-command-runtime-contract.md`
- `docs/decisions/016-layered-memory-architecture.md`
- `crates/talos-agent/src/compaction.rs`
- `crates/talos-agent/src/lib.rs`
- `crates/talos-session/src/lib.rs`
- `crates/talos-tui/src/state.rs`
