# TUI-053: Numeric Permission Approval Shortcuts

| Field | Value |
|---|---|
| Story ID | TUI-053 |
| Type | TUI / Permission Interaction Story |
| Priority | P1 |
| Status | Complete — existing implementation reconciled; historical claim gap disclosed |
| Source Issue | #268 |
| Selected Iteration | None |

Completion Commit: 1647f9d84e039cf6e53b8bdf11315bd7f358878b

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
| Handoff / Release Condition | Acceptance complete by maintainer confirmation and automated tests below. Historical Unclaimed fields are retained as an audit gap, not current implementation work or retroactive claim authority. |

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

### Automated Acceptance Follow-up — 2026-09-20

Maintainer requested closure of #268. Added only a regression test against the existing production
input entrypoint, without changing permission behavior: sequential 1/2/3 replies use their own
response channels; 0/4/9, legacy letters and a Chinese character leave the request unanswered;
Repeat/Release events cannot resolve the next prompt. TUI approval-focused suite passed 38 tests;
CLI expired-request/rollover barrier suite passed both tests. Commands: `cargo test --locked -p
talos-tui approval --lib` and `cargo test --locked -p talos-cli approval_queue_tests`.
These close the previously missing invalid-digit/sequential-event evidence, but do not simulate
an operating-system IME. Physical IME acceptance remains pending user confirmation; no Complete
claim or Issue closure is justified until that final evidence is supplied.

### Maintainer Acceptance And Closure — 2026-09-20

In direct response to the explicit question whether keeping Chinese IME enabled allows 1/2/3
to immediately select Once/Session/Deny without Enter or switching to English, the maintainer
confirmed: “这个我已经验证过了,符合预期的”. This is the previously missing physical-IME
acceptance, not a synthetic event test. Terminal name, IME product and tested binary SHA were
not supplied; do not invent them or claim universal compatibility across IMEs.

The existing main implementation above plus this confirmation and the 38 TUI / two CLI tests
complete #268's acceptance. Numeric mappings and invalid/repeat/sequential routing are covered
by `numeric_approval_routes_once_and_rejects_invalid_or_repeated_input`; existing render, Esc,
scope-preview and expired-request tests remain green. Arrow/Enter routing and permission policy
are unchanged by the original mapping-only diff. TUI-045 owns layout and is not reopened.

This reconciliation is explicitly authorized by the maintainer's “正确闭环” and “完成一下#268的闭环吧”.
It changes no production code, permission policy or grant scope. The original implementation was
misattributed to #285 and lacked a synchronized TUI-053 claim; leaving that historical metadata
Unclaimed discloses the gap rather than manufacturing a prior claim. No new iteration or claim
cycle is necessary to repeat already delivered behavior. Source #268 closes only after this
evidence repair and regression test are merged with applicable CI and independent review.
