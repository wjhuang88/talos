# TUI-APPROVAL-001: Compact large permission previews

> Document status: Review / Claimed (emergency local candidate)
> Emergency local candidate authorized by the maintainer; delivery completion not claimed.

## Native Windows validation — 2026-11-10

Source `c065236a1960e45920e9732212003f92bdefc57b`: pinned Rust 1.99.0 MSVC
`cargo +1.99.0-x86_64-pc-windows-msvc test --locked -p talos-tui --lib`
passed 652/652 with no failures or ignored tests, including localized compact approval
and large-scope rendering regressions. [Full evidence and limits](../reference/WINDOWS-VALIDATION-2026-11-10.md).
This supplies native unit-test evidence only; real Windows terminal/manual acceptance,
independent review and completion requirements remain as recorded. Status stays Review.

## Problem and deliverable

Thousands of exact Git Read facets fill the approval panel and displace controls.
The old wrapper also flattens explicit newlines, joining independent scope entries.
Default approval presentation must remain bounded, with exact details available.

The candidate caps collapsed panels at 12 rows and expanded panels at 20 rows
(subject to terminal space), counts Read/Write/Execute/Network entries, preserves
explicit newlines, and provides `d` and PageUp/PageDown navigation. Summary labels
are presentation only: the original preview and permission request remain intact.
Unknown/explanatory lines are counted as detail lines and retained in exact view.

## Authorization invariants and validation

Approve once and Deny work from summary. Always approve is gated until all exact
rows have appeared contiguously in successfully rendered pages. Input cannot skip
unrendered pages. A new request or width change resets review progress. Last-page
navigation must not produce an empty page. Rendering is evidence of display, not
proof that a human read the content. No automatic review or permission pipeline
changes are part of this task.

Regression coverage uses 2000 exact paths, bounded summary, visible controls,
blocked premature reusable approval, contiguous paging, final-page behavior,
resize invalidation, new-request reset, single-use approval, and explicit newlines.
User-facing documentation: README approval behavior.

## Delivery status

Local implementation and validation only. Effective claim, committed candidate,
applicable exact-head review/CI and real-terminal user acceptance remain pending.
Do not close based on the local test results. Git preparation repairs retain their
separate owner: 2026-11-10-git-tracked-snapshot-capacity.md.

## Local verification record

- `cargo test --locked -p talos-tui --lib`: 642 passed.
- `./scripts/release_preflight.sh`: passed, including locked workspace tests,
  check/Clippy, governance gates and the external runtime SDK fixture.
- `cargo xwin check --locked -p talos-tui --tests --target x86_64-pc-windows-msvc`:
  passed; native Windows execution remains separate.
- `git diff --check`: passed. Ephemeral logs: `/tmp/talos-approval-display-tests.log`,
  `/tmp/talos-approval-preflight.log`, `/tmp/talos-approval-xwin.log`.

## User validation follow-up: keyboard routing and cursor

User screenshots exposed two failures missed by the first local suite: global
history paging intercepted PageUp/PageDown before the approval handler, and a
visible terminal cursor moved through diff writes before returning to the composer.
Approval also incorrectly retained composer cursor ownership.

Local correction gives pending approval first refusal of paging keys, hides the
cursor before terminal writes and keeps it hidden while approval owns input.
Dismissal restores the composer cursor. Pagination regression now enters through
`handle_input_event`, not the leaf approval handler; it covers both directions.
Cursor regressions cover drawing and modal open/repaint/details/dismissal.
Real terminal verification remains pending; previous passing leaf-handler tests
were insufficient evidence of working keyboard navigation.

## User follow-up: decisions after paging

User reports numeric decisions apparently unresponsive at the last page. A new
real-dispatch/live-oneshot test verifies both 1 and 2 after paging. The exact live
incident cause is not established from screenshots. Inspection independently found
closed receiver handling could leave the panel and approval state inconsistent,
and output queued behind an expired approval was only drained after keyboard input.

The local candidate retires closed requests each event-loop iteration, closes both
modal states, restores viewport state, reports expiry/cancellation, and drains queued
output without requiring a key. It checks send failure before reporting approval.
The clipboard completion path uses the same retirement logic. It never grants an
expired request or extends permission deadlines. Tests cover live choices and
closed receiver retirement, including clipboard completion; 646 TUI tests passed.

## Accepted follow-up: operation semantics and repeat backlog

The user requested operation-level Git approval and repair of long-held key actions
continuing after release. The initial count-only summary is insufficient product
acceptance. ADR-102 records trusted typed operation metadata and the public Rust
constructor migration. Checkout summaries now name the operation, root, stage and
effects, with optional exact evidence. Numeric 1/2 both submit single-use approval
when clearly labelled as such. This never merges permission stages or authorizes
new effects. Conservative tracked-source capture remains unchanged.

Input now drains a bounded ready batch before rendering. Protocol-labelled repeats
are coalesced; a release removes pending repeats of that key, while independent
presses, paste and unrelated events retain order. A continuing hold makes at most
one repeat step per contiguous burst. Terminals without event-type/release reporting
retain Press semantics: the application cannot distinguish auto-repeat from rapid
independent presses, and must not discard real input. Ghostty's negotiated keyboard
protocol supplies repeat/release events. No new host utility dependency is added.

Verification includes a 500-repeat/release backlog with independent keys preserved,
held-key progress, operation-summary approval through real input and live oneshot,
2000 exact native facets retained, and foreign provenance rejection. This local
follow-up has no completion SHA and remains Review / Unclaimed.

Earlier full-validation attempts exposed unrelated fixture races: a released socket
port was claimed before rebinding; a session recovery fixture failed a runtime lock
acquisition. Preserve logs `/tmp/talos-approval-final-preflight.log` and
`/tmp/talos-approval-final-preflight-retry.log`; do not claim those runs passed.

## Cursor correction after user blink feedback

Hiding and showing on every redraw was an incomplete fix: it resets terminal blink
timing. The candidate now submits cell differences and the final cursor inside
standard synchronized-output markers. It emits visibility commands only when focus
changes, emits no cursor commands for unchanged frames, and closes synchronization
even after a write error. The legacy direct draw helper still supports hidden-cursor
frames; the app uses the frame-and-cursor path. Tests cover idle frames, dirty frames
without visibility toggles, modal hiding and focus restoration. Supporting terminals
such as Ghostty render the synchronized final state; terminals ignoring markers
retain functional rendering but may expose intermediate paint positions.

Primary implementation reference: crossterm 0.29.0 `event/stream.rs` poll_next uses
zero-timeout native polling for available input and a live task waker when Pending;
ready-input draining uses the live waker, not a noop-waker future probe. No second
stdin reader or unbounded repeat queue is introduced.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / gpt-6 |
| Work Slice | Operation approval UI, modal lifecycle, input repeat batching and atomic cursor drawing |
| Claimed At | 2026-11-10 |
| Source Issue | None |
| Governance Claim PR | Direct commit 259a2c65 |
| Authorization Mode | Emergency override |
| Authorization Evidence | Maintainer explicitly answered “授权本次紧急本地提交” in this session after review of the concrete local repair branch; repeated Git business unavailability and unusable approval/input UI. Local commit only, not merge authority. |
| Implementation PR | #903 |
| Last Updated | 2026-11-10 |
| Handoff / Release Condition | Reconcile claim on target branch within two business days; independent security/API review and exact-head CI before merge. |

Incident scope is limited to this owner's Work Slice. Branch:
`fix/git-tui-availability`, base `b1dbb983`. Preserve the user's separate
`feat/bg-default-continuation` checkout. Earlier uncommitted/Unclaimed notes describe
investigation history and are superseded for local commit authority by this record.
Rollback: revert this slice's implementation commit on the repair branch; do not
reset or clean the user's active worktree. Native Windows and real terminal manual
acceptance remain deferred; local MSVC compilation is not native execution.

## Emergency local commit validation

- `RUST_TEST_THREADS=1 ./scripts/release_preflight.sh`: exit 0, including locked
  workspace check/Clippy/tests and external SDK fixtures. Serial test scheduling
  avoids the unrelated shared-host fixture races recorded above; no tests skipped.
- `cargo xwin check --locked -p talos-cli -p talos-tui -p talos-tools --all-features
  --tests --target x86_64-pc-windows-msvc`: exit 0 without new warnings.
- TUI focused suite: 650 passed. Git write focused suite: 304 passed (other tools
  covered by full preflight). Checkout operation presentation regression passed.
- Both repository governance validators passed in the isolated repair worktree;
  scale assessment reports high-risk / release-managed / required worktrees.
- Staged source matches the validated original workspace byte-for-byte. Staged
  diff reviewed; `git diff --cached --check` passed. No unrelated proposals staged.
- Logs: `/tmp/talos-operation-complete-preflight.log`,
  `/tmp/talos-operation-complete-xwin.log`, `/tmp/talos-commit-governance.log`.
- Independent merge review, remote/native Windows CI, and final human terminal
  acceptance are outstanding. No completion claim is made by the local commits.

Local implementation commit: `259a2c65`. This records local preservation only; status remains Review.

## Main-branch consolidation

The maintainer subsequently requested consolidation into main and cleanup of all
safely removable branches. PR #903 carries this emergency repair and its claim
reconciliation. Independent security/API review and exact-head CI remain required
before merge; device/manual acceptance remains in this owner and status stays Review.
The final combined source matches the full preflight and MSVC cross-check candidate;
both governance validators passed after merging main `f9bfac77`.

## Real TUI locale regression correction

The macOS PTY acceptance revealed that compact scope summaries hid the existing
localized model effect/decision explanation. Collapsed summaries now keep up to
five explanation lines visible, truncating long rows with the existing marker;
full text remains in exact details. This applies to ordinary and operation
presentations without changing grants or reviewed-row accounting. Summary height
remains bounded (16 rows with review text; operation plus review at most 20).
Add a real rendered-frame regression for Chinese, English and Japanese and rerun
the existing loopback real-TUI locale/resume/rotation acceptance.
