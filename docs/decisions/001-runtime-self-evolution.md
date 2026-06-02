# 001: Self-Evolution as Runtime Primitive

## Context

Talos aims to be a next-generation agent runtime. The initial design (inherited from Hermes) treats
self-evolution as a feature of the skill system: after complex tasks (5+ tool calls), the agent
extracts a SKILL.md file, which is loaded in future sessions.

This approach has limitations:

1. **Reactive, not proactive**. Evolution only triggers after "complex tasks". The agent does not
   learn from simple interactions, errors, user corrections, or efficiency patterns.
2. **Single dimension**. SKILL.md captures procedural knowledge ("how to do X") but ignores user
   preferences, project conventions, error patterns, and efficiency opportunities.
3. **Coarse granularity**. Skill extraction is a binary event (happens or doesn't). There is no
   continuous learning signal.
4. **Application is manual**. Skills must be explicitly loaded into context. The runtime itself
   does not change its behavior based on accumulated experience.

The user proposes that self-evolution should be a **first-class runtime capability**: a built-in
learning loop that continuously observes, learns, and adapts agent behavior across all sessions.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
|---|---|---|---|
| All crates are pure Rust, no FFI | Hard | AGENTS.md | No |
| No unsafe without ADR | Hard | AGENTS.md | No |
| No secrets in persistent storage | Hard | AGENTS.md | No |
| Tests must pass before merge | Hard | AGENTS.md | No |
| Evolution must be transparent to user | Soft | Design principle | Yes |
| Evolution should not degrade performance | Soft | Performance requirement | Yes |
| Evolution data stored locally only | Soft | Privacy expectation | Yes |
| LLM calls for learning are acceptable cost | Assumption | User tolerance for token cost | Maybe |
| Structured knowledge store (SQLite) scales sufficiently | Assumption | Based on Hermes reference | Maybe |
| Pattern extraction can be partly rule-based | Assumption | Avoiding LLM cost for all learning | Maybe |

## Reasoning

### From Hard Constraints

The evolution system must be a pure Rust crate with no unsafe code, no external dependencies
beyond SQLite (already planned for session storage). All learning data stays local.

### From Soft Constraints

**Transparency**: The user must be able to inspect what the runtime has learned and correct it.
This means the knowledge store should have a human-readable representation (not just embeddings).

**Performance**: Learning observation must be near-zero cost (no LLM call per turn). Pattern
extraction can use lightweight heuristics. LLM-assisted learning is reserved for significant
moments (error recovery, user correction, complex task completion).

**Local-only**: All evolution data stored in `~/.talos/evolution/` (SQLite). No telemetry, no
cloud sync. User owns their data.

### From Assumptions

**LLM cost for learning**: We assume users accept occasional LLM calls for learning (e.g., "why
did this task fail?"). If this proves too expensive, the system can degrade gracefully to
rule-based learning only.

**SQLite scalability**: For a single-user CLI tool, SQLite is sufficient for storing evolution
data. If multi-user or server deployment emerges, this would need revisiting.

**Rule-based + LLM hybrid**: Not all learning needs LLM. Simple patterns (tool call frequency,
error recurrence, user correction rate) can be detected with rules. LLM is used for abstract
pattern extraction (e.g., "this user prefers functional style over imperative").

### Scope Separation: Evolution vs Skills

The key design question is where evolution ends and skills begin:

| Concern | Owner | Why |
|---|---|---|
| Procedural workflows ("how to do X") | **Skills** (SKILL.md) | User-editable, shareable, explicit |
| User preferences ("this user likes X") | **Evolution** | Implicit, personal, continuous |
| Project conventions ("this project uses X") | **Evolution** + AGENTS.md | Evolution detects, AGENTS.md declares |
| Error patterns ("avoid doing X here") | **Evolution** | Accumulated from failures |
| Efficiency optimization ("use tool Y first") | **Evolution** | Learned from performance data |
| Prompt strategy adaptation | **Evolution** | Runtime behavior, not user-facing |
| Tool call pattern optimization | **Evolution** | Performance concern |

Skills remain the **explicit, user-editable** knowledge layer. Evolution is the **implicit,
continuous** learning layer that influences runtime behavior below the skill level.

### Architecture: The Learning Loop

```
Agent Turn Execution
    │
    ├── Observe (zero-cost, synchronous)
    │   ├── Tool calls made (name, args, result, duration)
    │   ├── Errors encountered (type, context, recovery)
    │   ├── User corrections ("no, do it this way")
    │   ├── Token consumption per action
    │   └── Task outcome (success, partial, failure, user-abandoned)
    │
    ├── Accumulate (batch write to SQLite)
    │   ├── Observation buffer flushed periodically
    │   ├── No blocking on agent loop
    │
    ├── Extract (deferred, async)
    │   ├── Rule-based: frequency counters, error recurrence, correction tracking
    │   ├── LLM-assisted: abstract patterns from accumulated observations
    │   ├── Triggered by: session end, N observations accumulated, explicit /learn command
    │
    └── Apply (on next session / next turn)
        ├── Inject learned context into system prompt (user prefs, project patterns)
        ├── Adjust tool routing hints (prefer certain tools for certain patterns)
        ├── Adjust compaction strategy (preserve different content based on task type)
        └── Surface insights to user ("I noticed you always X, should I make that default?")
```

### Crate: `talos-evolution`

New crate introduced in iteration I005 (replacing skill-only approach):

```
talos-evolution/
├── src/
│   ├── lib.rs              # Crate root, re-exports
│   ├── observer.rs         # Captures events, buffers observations
│   ├── store.rs            # SQLite persistence for observations and patterns
│   ├── extractor.rs        # Rule-based + optional LLM-assisted pattern extraction
│   ├── adapter.rs          # Applies learnings to runtime behavior
│   └── types.rs            # Core types
```

The internal API (trait boundaries, method signatures, type definitions) will be designed
during I005 based on real usage patterns from I001-I004. The crate structure above is
directional guidance, not a commitment to specific traits.

## Cognitive Feedback Enhancements

Inspired by cognitive memory systems (MenteDB research), the evolution engine incorporates lightweight
feedback mechanisms. **Specific implementation details (signal types, decay functions, conflict
resolution strategies) are deferred to I005 iteration design.** This section establishes the core
concepts only. Concrete reference designs (Rust code, formulas, heuristics) are preserved in
`docs/reference/REFERENCE-PROJECTS.md` §17 for I005 implementation reference.

### Principles

1. **Observations are richer than success/failure**: The system captures nuanced signals —
   errors, user corrections, retries, inefficiency, satisfaction. The exact signal taxonomy
   will be designed during I005 based on what patterns emerge in I001-I004 usage.

2. **Patterns have confidence, not certainty**: Learned patterns carry a confidence score that
   increases with supporting evidence and decreases with contradicting evidence. A simple
   evidence-ratio formula is sufficient (no probabilistic models needed).

3. **Knowledge decays without reinforcement**: Patterns not observed recently lose confidence
   over time (time decay). This prevents stale preferences from dominating. The decay rate
   will be tuned during I005 based on real usage data.

4. **Contradictions are detected, not ignored**: When new observations conflict with existing
   patterns, the system resolves the conflict rather than silently overwriting. Resolution
   strategies range from override to surfacing the conflict to the user.

5. **Extraction is signal-driven**: Pattern extraction triggers on meaningful events (high-pain
   signals, observation thresholds, session end, user request) — not just batch processing.

### What We Explicitly Do NOT Build

| Concept | Why Rejected |
|---------|-------------|
| **Belief propagation** (graph-based) | Our data is flat key-value patterns, no graph topology |
| **Bi-temporal validity** | Over-engineered for our needs; simple timestamps sufficient |
| **Knowledge graph (CSR/CSC storage)** | Session branching is a tree, not a general graph |
| **LLM-powered contradiction resolution** | Rule-based resolution is sufficient; LLM reserved for extraction only |
| **Phantom tracking** | Time decay serves the same purpose with simpler implementation |

## Decision

1. Self-evolution is a **first-class runtime capability**, not a skill system feature.
2. A new crate `talos-evolution` is introduced with a 4-phase learning loop:
   Observe → Accumulate → Extract → Apply.
3. The crate is introduced in iteration I005, alongside the skill system. Skill creation from
   experience (#I005-S4) is redefined as one output channel of the evolution engine (not the only one).
4. Skills remain the explicit, user-editable knowledge layer. Evolution is the implicit learning
   layer that influences runtime behavior.
5. All evolution data is stored locally. No telemetry.
6. Evolution is transparent: users can inspect learned patterns and preferences, correct them,
   or disable evolution entirely via config.
7. **Specific implementation details** (signal taxonomy, confidence formulas, decay rates,
   conflict resolution strategies) are **deferred to I005 iteration design**, when we have
   real usage data from I001-I004 to inform the design.

## Evolution Wiring Mechanism (2026-06-01)

Evolution integrates as a builtin `talos_plugin::HookHandler` (`EvolutionHookHandler`),
subscribed to the lifecycle points needed for the four-phase learning loop:

| Phase | Hook event(s) used |
|-------|--------------------|
| **Observe** | `OnProviderError` (objective error signal), `BeforeProviderCall` (user-correction heuristic from latest user message), `OnTextDelta` / `OnToolResultObserved` / `AfterToolCall` (tool call observation) |
| **Accumulate** | Handler-internal `Arc<Mutex<TurnObserver>>` (reset on `TurnStart`) |
| **Extract** | `PatternExtractor::extract_from_observation` invoked at flush time |
| **Apply** | `OnSystemPromptBuilt` + `HookResult::Modify` returns augmented prompt with learned context |
| **Ingest** | Flush accumulated state in `TurnComplete` (override `fn timeout()` from default 500ms to ~5s for SQLite write) |

**Crate dependency**: `talos-evolution` depends on `talos-plugin` (added 2026-06-01).
No cycle: `talos-plugin` does not depend on `talos-evolution`. The plugin remains a thin
primitive; evolution is a consumer.

**Known trade-off**: prompt injection via `HookResult::Modify(HookEvent<'static>)` requires
`Box::leak` for the augmented `&str`. One small permanent allocation per turn.
Mitigation tracked separately as a `HookResult::ModifyOwned` variant (out of ADR-001 scope;
see ADR-005 → "Hook-Driven Evolution" for the full re-scope rationale).

## Reversal Trigger

Revisit this decision if:
- Observation overhead measurably impacts turn latency (> 5ms per turn).
- LLM-assisted extraction costs exceed 10% of total token budget.
- Users report that evolved behavior is unpredictable or unwanted despite transparency measures.
- SQLite proves insufficient for evolution data volume (unlikely for single-user CLI).
- The scope separation between evolution and skills proves unclear in practice, causing confusion
  about where to put different types of knowledge.
