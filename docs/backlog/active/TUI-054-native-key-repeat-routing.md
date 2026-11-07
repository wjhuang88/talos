# TUI-054: Native Key-Repeat Routing

| Field | Value |
|---|---|
| Story ID | TUI-054 |
| Type | TUI / Input Routing Story |
| Priority | P1 |
| Status | Review — implementation PR #872; physical acceptance deferred |
| Source Issue | #269 |
| Selected Iteration | I318 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | I318 / TUI-054 native editing/navigation repeat routing and one-shot/focus isolation |
| Claimed At | 2026-11-07 |
| Source Issue | #269 |
| Governance Claim PR | #869 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer explicitly requested implementation while #38 CI runs; independent input/approval-boundary review and exact-head CI required |
| Implementation PR | #872 |
| Last Updated | 2026-11-07 |
| Handoff / Release Condition | Effective only after atomic claim merge; complete implementation and validation, retain Review for unmet real-terminal acceptance |

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

## Implementation selection — 2026-11-07

[I318](../../iterations/I318-native-key-repeat.md) implements the complete Issue #269 input contract.
The intake exclusions describe the previous unselected state; this atomic claim explicitly authorizes
implementation only after merge. No timer, repeat-rate setting or permission-policy change.

Current facts: TUI input drops every non-Press event; terminal setup already requests Crossterm
REPORT_EVENT_TYPES. Numeric approval regressions reject Repeat/Release. Composer movement/deletion
currently counts Unicode scalar values; grapheme safety and forward Delete need explicit coverage.
The implementation must preserve one-shot actions, menu wrap/clamp and fail-safe focus transitions.

Real-terminal hold/release and IME acceptance on the primary terminal plus a materially different
terminal/platform remain required. Synthetic key events or PTY encoding tests alone cannot satisfy
physical keyboard/IME observations; retain Review and record missing evidence if unavailable.

## Implementation progress — 2026-11-07

Claim #869 became effective at `8a3bd94076b02d16769d3dc3a82f188de6ef1944`. Local implementation
now routes native editing/navigation Repeat through content-free focus ownership, rejects one-shot
action repeats, and uses Unicode grapheme-safe deletion/movement. Terminal focus reporting has
setup/rollback/restore coverage. Final TUI library regression run: 619 passed, 0 failed
(`/private/tmp/i318-tui-reviewed.log`). Final-source standard preflight passed (workspace check, warnings-denied Clippy, all workspace
tests and both external Runtime SDK modes; `/private/tmp/i318-preflight-final.log`). Windows MSVC
`cargo xwin check --locked -p talos-tui --tests --target x86_64-pc-windows-msvc`, TUI all-targets
Clippy, and the rebuilt CLI PTY fixture passed. Both governance validators passed. Initial sandbox
preflight failed with config-read EPERM; the successful run was outside that sandbox.

Implementation commit: `0972ef6ddf2d7f2709d7be5b98427798e4604aba`, PR #872. Independent Agent
static review approved that head against `8a3bd94076b02d16769d3dc3a82f188de6ef1944`; reviewer
ran no tests and shares the external identity. Exact-head remote CI/review and merge remain pending;
this is not merged delivery or physical acceptance evidence.

Maintainer explicitly deferred physical acceptance. **Ghostty without tmux**, second-terminal
hold/release, and Chinese IME observations remain unverified. Keep this scope open; PTY protocol
execution and synthetic event tests cannot replace these observations.
