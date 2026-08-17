# TUI-054: Native Key-Repeat Routing

| Field | Value |
|---|---|
| Story ID | TUI-054 |
| Type | TUI / Input Routing Story |
| Priority | P1 |
| Status | Intake / Unclaimed |
| Source Issue | #269 |
| Selected Iteration | None |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #269 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-17 |
| Handoff / Release Condition | Inventory repeat-safe versus one-shot actions and terminal event support before selecting an implementation iteration. |

## Identity / Goal / Value

Honor terminal-native repeat events for editing and navigation while preventing repeated submit,
permission, confirmation, mutation or modal-transition actions.

## Scope

- Inventory repeat event handling across composer, menus, pickers and viewport navigation.
- Define one shared repeat-safe/one-shot classification and focus-transition boundary.
- Preserve Unicode editing, IME composition and existing wrap/clamp semantics.

## Exclusions

No application-owned repeat timer, OS repeat-setting change, shortcut redesign or implementation
authority.

## Acceptance For Intake

- [ ] Crossterm press/repeat/release facts and supported-terminal behavior are recorded.
- [ ] Repeat-safe and one-shot action matrices cover focus/modal transitions and Issue #268.
- [ ] A runnable iteration, real-terminal matrix and effective claim exist before implementation.
