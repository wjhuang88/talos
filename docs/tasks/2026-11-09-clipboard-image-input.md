# CLIPBOARD-IMAGE-001: Explicit clipboard image attachments

> Document status: Active / Claimed (proposed; effective only after governance merge)

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
