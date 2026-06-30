# Self-Bootstrap Session Evidence Template

Created: 2026-06-30 (T11 of the four-month self-bootstrap plan)

This template records evidence for Talos-on-Talos development rehearsal sessions (plan items T38,
T52, T61). Copy this template into a new file under `docs/tasks/` for each rehearsal session.

## Purpose

REL-002 requires that Talos can perform 100% self-bootstrap development as the primary runtime
before `v1.0.0` is claimed. Rehearsal sessions expose gaps between the current product and that
goal. They need not fully satisfy REL-002, but they must record what worked, what required external
assistance, and what gaps remain.

## Template

```markdown
# YYYY-MM-DD Self-Bootstrap Rehearsal: {Title}

**Rehearsal number**: {1|2|3}
**Plan item**: {T38|T52|T61}
**Session date**: YYYY-MM-DD
**Runtime**: Talos {version} on {platform}
**Change type**: {documentation-only | small code change | architecture-sensitive slice}
**External assistance**: {none | labeled below}

## Objective

{What this rehearsal attempted to accomplish using Talos as the primary development runtime.}

## Scope

- **In scope**: {specific files/modules changed}
- **Out of scope**: {what was explicitly not attempted}

## Environment

- Talos version: {output of `talos --version`}
- Provider/model used: {provider and model name}
- Workspace: {repo path}
- Starting commit: {git SHA}

## Execution Record

| Step | Tool(s) used | Outcome | Notes |
|---|---|---|---|
| 1 | {read/grep/edit/bash/...} | {success/failure} | {observation} |

## External Assistance

Label every action that required a human or external agent (non-Talos) to complete:

| Step | What was needed | Who provided it | Why Talos could not |
|---|---|---|---|
| {N} | {action} | {human/external-agent} | {gap description} |

If no external assistance was needed, state: "No external assistance required."

## Validation Evidence

- `cargo check --workspace`: {pass/fail + output summary}
- `cargo test --workspace`: {pass/fail + counts}
- `scripts/validate_project_governance.sh .`: {0 warnings/errors}
- Other: {any runtime or manual verification}

## Commit

- Commit SHA: {SHA}
- Commit message: {message}
- Files changed: {count + list summary}

## Gaps Exposed

| Gap | Severity | Blocking REL-002? | Recommended fix |
|---|---|---|---|
| {description} | {high/medium/low} | {yes/no} | {action item} |

## Assessment

- **Self-bootstrap coverage**: {percentage estimate of work done by Talos vs external}
- **Would this rehearsal satisfy REL-002?**: {yes/no + why}
- **Ready for the next rehearsal level?**: {yes/no + conditions}

## Recovery

- To resume: `git checkout {SHA}` and read this evidence record.
- Next rehearsal should attempt: {suggested next complexity level}.
```

## Usage Notes

- Fill every field. "None" or "N/A" is acceptable; blanks are not.
- External assistance must be labeled explicitly per the plan's Team Handoff Prompt rule 7.
- The gaps table feeds directly into the REL-002 readiness report (T63) and the Month-3 closeout
  gap report (T54).
- Do not delete rehearsal evidence records after creation; they are cumulative evidence.
