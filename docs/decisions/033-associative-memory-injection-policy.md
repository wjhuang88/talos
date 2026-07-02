# 033: Associative Memory Injection Policy

## Status

Accepted

## Context

MEM-008 introduced a SQLite-backed weighted associative memory graph for explicit, bounded
multi-hop recall. T31 previously decided not to enable automatic associative memory prompt
injection by default because retrieval quality, trigger policy, token budget, and cache impact were
not yet measured.

T50/T43 then delivered the explicit `MemoryStore::graph_recall()` API with hop, edge-weight, and
fanout bounds. T51 added compression/retrieval metrics primitives, but there is still no benchmark
corpus comparing automatic associative injection against explicit recall on representative Talos
workflows.

Talos already has a separate direct memory prompt path:

- `[memory_prompt]` is disabled by default.
- `format_memory_prompt()` retrieves direct semantic memories, not graph associations.
- `SystemPromptBuilder::with_memory_section()` inserts memory as dynamic prompt context, outside the
  stable prefix.

The remaining T131 question is whether associative graph recall should now be injected
automatically on each model turn.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
| --- | --- | --- | --- |
| Current session, user instructions, source files, tests, and ADRs outrank memory. | Hard | ADR-016 / AGENTS.md accuracy discipline | No |
| Prompt injection must be bounded and disabled by default unless evidence justifies otherwise. | Hard | MEM-008 / T31 / I051 | No |
| Stable prompt-prefix behavior must not churn because of per-turn memory retrieval. | Hard | ARCH-006 / prompt cache design | No |
| Associative recall can surface weakly related but non-authoritative context. | Assumption | MEM-008 value hypothesis | Needs validation |
| T51 metrics are sufficient to evaluate token overhead and retrieval counts. | Assumption | T51 implementation | Partially; usefulness and false-positive rates still need a benchmark |
| A future opt-in experiment may be useful for self-bootstrap continuity. | Soft | Replan T131 | Yes |

## Reasoning

The implemented graph recall API is explicit and bounded, but automatic injection has a different
risk profile than explicit recall:

- It changes the model-facing prompt on every turn, even when the user did not ask for memory.
- Weak associations are easy to overread as evidence unless the section is carefully labelled.
- It consumes prompt budget during long sessions, competing with session todos, active context, and
  tool output summaries.
- T51 provides counters, not a measured usefulness or false-positive benchmark.
- The existing direct memory prompt path already proves the safe insertion seam: dynamic suffix,
  default disabled, bounded output, and hidden-output filtering.

The simplest safe policy is therefore to keep associative recall explicit for now. This preserves
the useful graph work without adding automatic context pollution or unmeasured cost.

## Decision

1. **Do not enable automatic associative memory injection by default.**
   - Default-on associative injection is rejected for v1 readiness.
   - No existing config default changes in T131.

2. **Do not add a new config-gated automatic associative injection implementation in T131.**
   - T131 is a decision gate, not an implementation gate.
   - The current direct `[memory_prompt]` path remains unchanged and disabled by default.

3. **Keep associative graph recall explicit.**
   - Consumers may call `MemoryStore::graph_recall()` directly with bounded parameters.
   - Any model-facing use must label output as advisory association, not evidence or truth.

4. **Future automatic associative injection, if pursued, must be a separate experimental slice.**
   - It must use a distinct config namespace from direct semantic memory, for example
     `[associative_memory_prompt]`.
   - It must default to disabled.
   - It must live in the dynamic prompt suffix and prove stable-prefix hashes are unchanged when
     enabled, disabled, and empty.
   - It must have independent budgets for max items, max characters/tokens, max hops, min score,
     and fanout.
   - It must expose why each association was injected: seed, path, score, and evidence reference.
   - It must include hidden-output filtering at least as strict as `format_memory_prompt()`.

5. **Required evidence before revisiting default-on behavior:**
   - a benchmark corpus with at least three representative Talos workflows;
   - before/after comparison against explicit recall only;
   - token overhead report;
   - false-positive/context-pollution review;
   - stable-prefix regression tests;
   - user-visible disable/inspection path.

## Reversal Trigger

Revisit this decision only after an opt-in experiment shows that associative injection improves
task continuity on representative workflows without material false-positive prompt pollution, and
without stable-prefix churn. Even then, changing the default from disabled to enabled requires a
new ADR because it changes model-facing runtime behavior.
