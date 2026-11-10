# CLIPBOARD-IMAGE-001: Explicit clipboard image attachments

> Document status: Review / Claimed

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / gpt-6 |
| Work Slice | Explicit TUI clipboard image paste with native Wayland clipboard access, bounded ingestion, pending attachment integration and terminal documentation |
| Claimed At | 2026-11-09 |
| Source Issue | None |
| Governance Claim PR | #894 |
| Authorization Mode | Independent review |
| Authorization Evidence | User reports Linux Wayland+niri+Ghostty Ctrl+Shift+V cannot paste images and authorizes following Codex implementation logic; independent boundary review required |
| Implementation PR | Not started |
| Last Updated | 2026-11-09 |
| Handoff / Release Condition | Local full locked preflight, cargo-xwin, exact-head CI/review and honest native clipboard acceptance |

## Deliverable and acceptance

A user presses Ctrl+V or Alt+V in the ordinary TUI composer to explicitly import
clipboard image data into a visible pending attachment; Enter remains the send action.
Ctrl+Shift+V works only when the terminal forwards it. Ghostty's default text paste
binding remains a terminal-owned action; document the distinction and optional
performable binding. Text paste remains unchanged and modal/credential inputs must
never trigger image acquisition. Repeat events must not create duplicate attachments.

Follow Codex e54a5713caf331f42a6eef0166186df5f7563c29 logic: native arboard with
wayland-data-control, explicit image acquisition, PNG staging and pending attachment.
Preserve Talos model capability, byte/pixel/count limits, exact file authorization
for clipboard file-list entries, and session ownership. Never turn pasted ordinary
text into implicit filesystem access. Host clipboard access occurs only on user action.

Check unsupported models before clipboard reads; distinguish no image/unavailable/
busy/failure visibly. Contain dependency panics and bound host calls so UI cannot
freeze. File staging is host-owned input ingestion, not model tool write authority.
No auto-submit, clipboard polling, model-triggered clipboard read, screenshot capture,
provider protocol expansion, or Ghostty config mutation is authorized by implementation.

## Required Reads

AGENTS.md; docs/sop/AGENT-COLLABORATION.md; docs/sop/LOCAL-DEV.md;
MODEL-009-C/D owners; image_validation.rs, image_authorization.rs, tui_bridge.rs;
talos-tui app/input.rs and clipboard.rs; Codex clipboard_paste.rs and chatwidget/clipboard.rs.

## Constraints and validation

Hard: existing permission and image ingestion boundaries; no unapproved native/unsafe
Talos code; pinned locked checks. Soft: add Rust clipboard library rather than requiring
wl-paste; review transitive native boundary before implementation. Assumption: actual
Ghostty key forwarding needs device acceptance; Ctrl+V is primary supported shortcut.

Tests cover key routing/modal isolation/repeats, capability-before-read, valid image,
invalid or oversized data, failure/timeout, pending attachment/send and cleanup ownership.
Run release_preflight.sh, cargo xwin MSVC --tests and exact-head CI. README documents
shortcuts, limits and remote/headless constraints. This owner retains any residuals.

## Inventory and preservation

Existing MODEL-009-C/D are Complete and explicitly excluded clipboard acquisition;
this is a new bounded task, not a reinterpretation of their acceptance. Board and
iteration inventory inspected: I277 deferred device acceptance, I302 handle evidence,
I318 key-repeat review and existing permission/search/provider work remain with their
owners; none admits clipboard work. Open PRs #838/#849/#861 and #893 are non-overlapping.
Preserve BG-CONTINUE files/worktree and session-auto-approval proposal. No iteration
activation is selected; this standalone task owns the runnable deliverable.

## Execution record

Governance claim #894 merged as `682d47cf`; implementation in progress on
`fix/clipboard-image-input`. Independent claim review approved exact head
`f9d5ef57714c499200ac16ead55958b330c658c9`. Local governance validators passed.

ADR-100 records the native clipboard OS boundary, isolated worker and durable
clipboard file policy. Final implementation `485e1a9fe93cc5c21e1ecbbdd7d51545e85695a9` passed the full
locked release preflight and cargo-xwin MSVC `--tests` check. One preflight attempt
hit an unrelated git-daemon fixture port collision before Talos dispatch; the
focused coding-feature test and complete unchanged-source preflight rerun passed.
Independent exact-head technical/permission/native review approved this commit
against `682d47cf5023649f3c08f47e1f9dca966e237ebd` with shared-identity disclosure.
After the user copied an image, the native Wayland worker returned PNG successfully:
1356x852 pixels, 312719 PNG bytes. No image contents were logged or saved. This
validates host acquisition/encoding; final Ghostty key-to-composer acceptance remains. Tests cover draft retention,
repeat/modal guards, cancellable file authorization, current limits, staged cleanup
and committed-image retention. A deterministic child-process fixture passed timeout/cancellation reaping checks. Native Windows execution remains a remote CI gate.
Residual owned here: reference-aware cleanup of successfully submitted clipboard
images is not implemented; retain these files for session recovery. Device acceptance
on Wayland+niri+Ghostty remains required. Do not mark this task Complete yet.

### 2026-11-10 attachment removal regression follow-up

Within this owner's pending-attachment acceptance scope, the user reported `/detach`
reaching a session-mutation error after clipboard attachment. Local reproduction found
that entering arguments kept the command menu open with no matches, reset selection
to index zero, and Enter executed that hidden item. On the inspected revision the
hidden item is `/compact`; the screenshot's exact session command is not established.
This is an input-routing defect, not evidence that attachment removal needs relaxed
session-mutation permissions.

Local repair closes command completion when arguments start and checks the selected
item against the current filtered results before Enter/Tab can activate it. Tests
first reproduced both failures, then passed for typed/pasted arguments and the full
keyboard → command → attachment-removal/status path. No permission or session guard
is removed. README documents direct argument entry. This follow-up is prepared on
`fix/clipboard-detach-routing` under Review; delivery completion is not claimed.

Validation: `cargo test --locked -p talos-tui --lib` passed (640 tests).
`cargo xwin check --locked -p talos-tui --tests --target x86_64-pc-windows-msvc`
passed. The first full preflight passed checks/Clippy but failed two talos-core
source-layout tests: the reused executable embedded deleted worktree paths instead
of the current `CARGO_MANIFEST_DIR`. After `cargo clean -p talos-core`, the unchanged
`i179_source_layout` suite passed both tests. The complete unchanged-source preflight
rerun passed (exit 0), including locked workspace checks, Clippy, tests and external
runtime SDK fixture. Logs: `/tmp/talos-detach-preflight-rerun.log` and
`/tmp/talos-detach-xwin.log` (local, ephemeral). `git diff --check` passed.
User acceptance (2026-11-10): after the request to restart with `cargo run -- -c`
and verify attachment removal in Ghostty, the user confirmed “验证过了，没问题”.
This closes the reported `/detach` real-terminal regression acceptance; the user did
not specify which removal variant was exercised. Both variants have automated coverage.
This confirmation does not establish all clipboard lifecycle/platform acceptance.
Native Windows runtime, exact-head remote CI/review and merge remain delivery gates.
The user authorized committing and pushing this follow-up after terminal acceptance.
