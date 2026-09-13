# ADR-074: Prompt Authority and Provenance Boundary

Status: Proposed

## Context

Talos assembles model-visible input from runtime invariants, scoped `AGENTS.md` files, tools,
skills, memory, Evolution, user preferences, and caller-provided prompt text. The existing
builder already separates a cacheable prefix from a dynamic suffix, but the authority and
provenance of each contributor are implicit.

## Decision

Every prompt contribution must be classified by source, scope, authority, provenance, and cache
class before any future semantic rewrite. Runtime/core invariants and explicit current user intent
outrank scoped project instructions, preferences, advisory memory, learned Evolution patterns,
tool output, and extension text. A nearer scoped instruction may refine an ancestor only within its
scope; it cannot weaken runtime invariants or silently erase a nearer user instruction. Memory and
Evolution remain advisory and must never be rendered as equivalent system instructions.

`custom_prompt` and hook contributions are caller/extension input, not replacements for runtime
invariants. Future changes must preserve the current stable-prefix/dynamic-suffix partition unless
an explicit migration proves cache and compatibility effects. Truncation must preserve invariant
and authority boundaries, never cutting a rule into an ambiguous fragment.

## Compatibility and migration

This ADR is a design contract only. It changes no current prompt output, public API, persistence,
or provider protocol. Implementations require separate child iterations, focused tests, and an
independent review for authority-sensitive changes.

## Consequences

The next slices can add typed provenance and conflict diagnostics without conflating advisory
context with authority. Existing memory safety boundaries and SDK compatibility remain in force.
