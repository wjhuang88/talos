# ARCH-005: God Module Decomposition

**Status**: Complete (→ I029, 2026-06-18)
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

- [x] No behavior changes are intentionally introduced in decomposition commits.
- [x] Each split is committed independently by crate or module family.
- [x] Public API churn is avoided unless already approved by ARCH-003/004.
- [x] `cargo test --workspace` and `cargo clippy --workspace -- -D warnings` pass after each slice.
- [x] Architecture reference is updated if module ownership changes.

## Completed Slices (2026-06-18)

| File | Before | After | Status |
|------|--------|-------|--------|
| `talos-agent/src/lib.rs` | 2833 | 862 lines | ✅ Extracted: tool_execution.rs, tests.rs, helpers.rs |
| `talos-tools/src/lib.rs` | 2484 | 23 lines | ✅ Extracted: bash_tool.rs, file_tools.rs, search_tools.rs, diff_stat.rs |
| `talos-tui/src/app.rs` | 2516 | 950 lines | ✅ Extracted: scrollback.rs, tool_display.rs, app/app_tests.rs |
| `talos-cli/src/main.rs` | 2236 | 1250 lines | ⚠️ Partial: registry.rs, provider_setup.rs, session_setup.rs, tui_bridge.rs extracted |

## Follow-up Stories

- `talos-session/src/lib.rs`: 1737 lines, not decomposed → **ARCH-008**
- `talos-skill/src/lib.rs`: 1484 lines, not decomposed → **ARCH-009**
- `talos-cli/src/main.rs`: 1250 lines remaining → **ARCH-010**
- `talos-tools/src/file_tools.rs`: 1308 lines (new, should be watched) → **ARCH-010** or future

These follow-ups are separate post-ARCH-005 residual stories. They do not keep the I029
ARCH-005 slice open.

## Verification Notes

Use file-size inventory in `docs/reference/ARCHITECTURE-AUDIT-2026-06-18.md` as the baseline.
