# TUI-053: Numeric Permission Approval Shortcuts

| Field | Value |
|---|---|
| Story ID | TUI-053 |
| Type | TUI / Permission Interaction Story |
| Priority | P1 |
| Status | Partial / Unclaimed — implementation merged; acceptance reconciliation pending |
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
| Implementation PR | None — existing commit 1647f9d84e039cf6e53b8bdf11315bd7f358878b is on main; no associated PR returned by API |
| Last Updated | 2026-09-20 |
| Handoff / Release Condition | Reconcile existing implementation evidence and complete remaining acceptance below; do not invent retroactive claim authority or repeat the implementation. New permission-surface code changes require an effective claim. |

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

## Implementation Reconciliation — 2026-09-20

The intake above was not synchronized when numeric shortcuts shipped. Existing main ancestor
`1647f9d84e039cf6e53b8bdf11315bd7f358878b` changes CLI/TUI approval labels and routing from
letters to `1/2/3`, with rendering and queued-request test updates. Its message references #285,
but that prompt-authority umbrella does not own this requirement: #268 / TUI-053 does. Preserve
the historical commit; this correction establishes the requirement link, not retroactive claim
or independent-review evidence. No numeric-shortcut reimplementation is requested by this repair.

| Acceptance | Current evidence / remaining work |
|---|---|
| Visible numeric labels | `widgets.rs`, `panel_state.rs`, CLI `approval.rs`; existing rendering assertions check numeric labels. |
| Direct 1/2/3 actions | TUI `app/input.rs` maps digits directly to Once/Session/Deny; CLI retains its existing line-confirmation contract. |
| Arrow/Enter/Esc | Existing TUI branches retained in the implementation diff. |
| Permission semantics | Mapping-only diff; no permission-engine or grant-scope change in this commit. |
| Repeat and request identity | Input filters non-Press events; CLI queued-request identity/barrier tests updated. Final digit-specific sequential acceptance still needs explicit verification. |
| Invalid numeric keys | Source wildcard ignores unrelated characters; dedicated negative runtime test evidence still needs verification. |
| Chinese/non-Latin IME | No attributable physical-IME acceptance record found. Must test with an actual IME; synthetic CJK render tests are not equivalent. |

Do not mark Complete or close #268 based on code inspection alone. Remaining evidence belongs to
this owner and the existing Issue, not new subtask Issues. Manual acceptance: with Chinese IME
enabled and no active composition, use three separate harmless permission requests; confirm 1
approves once, 2 grants only the displayed reusable scope, 3 denies. On another prompt verify 0/4/9
do not approve, then arrow/Enter and Esc. Confirm a queued second request is not approved by the
first key's repeat. Record terminal/IME/build SHA and observations, including failures.
