# SOP: Evolution Feedback

## Purpose

Define when and how Agents update `EVOLUTION.md`. Lessons are for reusable corrections,
surprising failures, and project traps; they are not a changelog.

## When to Record a Lesson

At session close, or after a failed validation/user correction, check whether the work exposed a
reusable lesson:

- corrected omission in rules, docs, tests, commands, or generated artifacts;
- repeated failure, surprising failure, or user correction;
- newly discovered project trap;
- workaround that should be reused;
- process drift that caused an Agent to miss an existing rule.

If none apply, do not write a lesson.

## Entry Shape

Use this compact shape:

```markdown
## YYYY-MM-DD - short lesson title

- Trigger:
- Symptom:
- Root cause:
- Fix:
- Prevention:
- Promoted to rule/check:
```

`Promoted to rule/check` must name `AGENTS.md`, a SOP, validator, test, or `none`. If the lesson is
mandatory for future execution, promote it instead of leaving it only in `EVOLUTION.md`.

## Routing Rule

Before writing `EVOLUTION.md`, route these cases here:

- diagnosis after a user correction;
- session close when non-obvious problems occurred;
- failed governance validation;
- repeated mistakes;
- process drift that caused stale or missing owner documents.

When multiple lessons exist, keep the `EVOLUTION.md` lesson index current.

