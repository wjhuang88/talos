# Talos Desktop

## Problem

Talos currently serves terminal-first workflows. A desktop shell could make session browsing, diff
review, configuration, permission approval, and richer visualizations easier for users who do not
want to live in a terminal.

## Proposed Approach

Treat desktop as a proposal only. The likely path is a Rust core with a desktop shell that reuses
`talos-core`, `talos-agent`, `talos-provider`, `talos-tools`, and the existing permission model.
Tauri plus a web frontend is a candidate, but it needs an ADR because it introduces web frontend
code and WebView supply-chain/platform questions.

## Alternatives Considered

- Pure Rust GUI such as `egui` or `iced`.
- Keep TUI-first and add richer panels, mouse support, and dashboard features.
- Hybrid WebView plus native views for specialized rendering.

## Open Questions

- Whether JS/TS frontend code is allowed in this Rust-first repository.
- How desktop release cadence relates to CLI release cadence.
- How permission approval, secrets, and local dashboard/web control boundaries are shared.
- Whether the desktop code lives in this monorepo or a separate repository.

## Dependencies

- `RUNTIME-001` reusable runtime facade.
- Permission sandbox decision work (`PERM-004`).
- Session/export service work (`SESSION-004` and WEB-001 follow-ups).
- A new ADR before implementation.

## Source

- [GitHub Issue #29](https://github.com/wjhuang88/talos/issues/29)

