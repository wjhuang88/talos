# ARCH-005: God Module Decomposition

**Status**: Planned
**Priority**: P3
**Source**: ARCH-002 audit
**Depends on**: ARCH-003 and ARCH-004 preferred

## Problem

Several modules exceed 1,000 lines and mix runtime behavior, rendering, registry construction, and
tests. This increases review cost and makes narrow changes harder to verify.

## Scope

Decompose without behavior changes:

- `talos-agent/src/lib.rs` into turn loop, tool execution, event flow, and tests.
- `talos-tui/src/app.rs` into scrollback, tool display, markdown rendering, and event loop units.
- `talos-tools/src/lib.rs` into focused tool-family modules.
- `talos-cli/src/main.rs` into registry, provider setup, session setup, and TUI/RPC bridge modules.
- Extract session/skill tests or helpers where this can be done without changing public APIs.

## Acceptance Criteria

- [ ] No behavior changes are intentionally introduced in decomposition commits.
- [ ] Each split is committed independently by crate or module family.
- [ ] Public API churn is avoided unless already approved by ARCH-003/004.
- [ ] `cargo test --workspace` and `cargo clippy --workspace -- -D warnings` pass after each slice.
- [ ] Architecture reference is updated if module ownership changes.

## Verification Notes

Use file-size inventory in `docs/reference/ARCHITECTURE-AUDIT-2026-06-18.md` as the baseline.
