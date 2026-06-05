# ADR-012: Exec Policy DSL Boundary

- **Status**: Accepted
- **Date**: 2026-06-05
- **Backlog**: #I010-S8

## Context

`#I010-S8` proposes rule files under `.talos/rules/*.rules` and
`~/.talos/rules/` for approving or denying command execution. This can
materially change the effective permission boundary, especially for shell
commands that include pipes, redirects, globbing, environment variables, or
path-sensitive arguments.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
| --- | --- | --- | --- |
| Write-capable tools must go through permission pipeline | Hard | AGENTS.md hard constraint #4 | No |
| Permission code changes need security review | Hard | AGENTS.md hard constraint #5 | No |
| Rule DSL should reduce repetitive approvals | Soft | #I010-S8 product direction | Yes |
| Shell parsing is easy to get wrong | Assumption | Existing `bash` escape hatch behavior | Must be bounded |

## Reasoning

A broad DSL can accidentally become a shell parser and create hidden allow
paths. Talos should not attempt to prove arbitrary shell commands safe. The DSL
must classify a narrow structured surface and fall back to `Ask` when it cannot
classify a command with confidence.

## Decision

- The DSL is permission policy input, not a replacement for the permission
  engine.
- Rule files must compile into typed `PermissionRule` values or a small
  equivalent internal representation. No dynamic code execution.
- Matching order is explicit: project rules before user-global rules only when
  the project root is trusted by policy; otherwise user-global rules win.
- Deny rules always win over allow/trusted rules when both match the same
  operation.
- Paths must be normalized against the workspace root before path-prefix or glob
  matching.
- Rules may match command name and bounded argument positions. They must not
  require full shell parsing for correctness.
- Commands containing complex shell features (`|`, `>`, `>>`, `<`, command
  substitution, unexpanded wildcards, subshells, or unknown metacharacters) must
  bypass trusted rules and return `Ask` unless a future ADR approves a parser.
- Environment-variable references in rule files may be expanded only at rule
  load time through the existing config substitution behavior. Unset variables
  must not widen a rule.
- Parse errors are fail-safe: ignore the invalid rule set and report a warning,
  or fail startup for explicitly strict modes. Never partially apply ambiguous
  rules silently.

## Reversal Trigger

Revisit this decision if Talos adopts a proven command parser or replaces the
`bash` escape hatch with fully structured native command tools for the relevant
operation classes.
