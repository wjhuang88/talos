# Iterations

## Purpose

Track iteration plans, execution progress, and retrospectives.

## Naming Convention

```
docs/iterations/
├── README.md           (this file)
├── I001-<slug>.md      (iteration plan + execution record)
├── I002-<slug>.md
└── ...
```

## Lifecycle

1. **Planned** — Iteration created with scope, selected stories, and acceptance criteria.
2. **Active** — Work in progress. Update story status as work proceeds.
3. **Review** — All stories implemented. Run verification checklist.
4. **Complete** — Verification passed, retrospective written.

## Rules

- Each iteration has a unique ID (`I001`, `I002`, ...).
- Published iteration baselines must not be silently overwritten by later execution.
- Start a new iteration only after inventorying all existing active, review, planned, and blocked iterations.
- Record execution results by appending to the plan, not replacing it.

## Current Iterations

No iterations yet. See `docs/roadmap/IMPLEMENTATION-ROADMAP.md` for the plan and `docs/sop/START-ITERATION.md` for the process.
