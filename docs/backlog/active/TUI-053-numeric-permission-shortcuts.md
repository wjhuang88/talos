# TUI-053: Numeric Permission Approval Shortcuts

| Field | Value |
|---|---|
| Story ID | TUI-053 |
| Type | TUI / Permission Interaction Story |
| Priority | P1 |
| Status | Intake / Unclaimed |
| Source Issue | #268 |
| Selected Iteration | None |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #268 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-17 |
| Handoff / Release Condition | Resolve overlap with TUI-045/#125 and establish a permission-surface claim before implementation. |

## Identity / Goal / Value

Use visible `1 / 2 / 3` one-shot permission choices so users with a non-Latin IME do not need to
switch input methods to approve once, approve with the existing reusable scope, or deny.

## Scope

- Numeric presentation and direct-key routing for the existing three interactive decisions.
- Preserve arrow/Enter and Esc behavior where currently supported.
- Prove invalid or repeated numeric input cannot authorize another request.

## Exclusions

No permission-policy, grant-scope, request-identity, timeout, headless or layout redesign. No
implementation authority is created by this intake.

## Acceptance For Intake

- [ ] Existing decision semantics and the TUI-045/#125 boundary are mapped.
- [ ] IME and one-shot repeat-safety evidence is runnable and testable.
- [ ] A selected iteration and effective protected-surface claim exist before implementation.
