# Architecture Decision Records

## Purpose

Record significant technical decisions that affect Soft or Assumption constraints. Not for
routine implementation choices that follow established patterns.

## Naming Convention

```
docs/decisions/
├── README.md           (this file)
├── 001-<slug>.md       (decision record)
├── 002-<slug>.md
└── ...
```

## Template

```markdown
# [Decision Title]

## Context
[Why a decision is needed]

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
| --- | --- | --- | --- |
| [constraint] | Hard / Soft / Assumption | [source] | No / Yes / Maybe |

## Reasoning
[What is the simplest approach satisfying Hard constraints?
Why deviate if we chose to?
Which Assumptions need validation?]

## Decision
[What was chosen and what was rejected]

## Reversal Trigger
[Under what conditions should this be revisited?]
```

## When to Write

| Trigger | Example |
| --- | --- |
| Choosing between approaches satisfying Hard constraints | Async runtime choice |
| Proceeding based on unvalidated Assumption | "WASM is fast enough for plugins" |
| Overriding a Soft constraint | "Using dynamic dispatch despite preferring static" |
| A Hard constraint forces an unpopular choice | "No unsafe without ADR" |

## Current Decisions

1. [001: Self-Evolution as Runtime Primitive](001-runtime-self-evolution.md) — Evolution is a first-class runtime capability (Observe → Learn → Adapt), not just a skill system feature.
