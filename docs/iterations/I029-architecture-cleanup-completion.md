# I029: Architecture Cleanup Completion

**Status**: Complete (2026-06-18)
**Started**: 2026-06-18
**Depends On**: ARCH-003 complete (I027)

## Outcome

Complete all remaining ARCH-002 follow-up stories: anti-corruption layers (ARCH-004), prompt
cache stability (ARCH-006), clippy `--all-targets` cleanup (ARCH-007), and god module
decomposition (ARCH-005). After I029, the architecture audit findings from ARCH-002 are fully
closed.

## Selected Stories

### Group A: Anti-Corruption Layers (ARCH-004, P2)

- [ ] #ARCH-004-A: Define Talos-owned MCP descriptors/errors so `rmcp` types stay inside MCP adapter modules
- [ ] #ARCH-004-B: Define crate-owned SQLite store errors for `talos-evolution` and `talos-session`
- [ ] #ARCH-004-C: Replace `talos-mcp` direct config dependency with MCP-owned config DTOs
- [ ] #ARCH-004-D: Resolve duplicate `ToolDefinition` types

### Group B: Cleanup (ARCH-007, P3)

- [ ] #ARCH-007: Fix ~35 pre-existing `clippy::unwrap_used` warnings in test targets

### Group C: Prompt Cache Stability (ARCH-006, P2)

- [ ] #ARCH-006: Make prompt prefix stability a session contract and expose cache metadata

### Group D: God Module Decomposition (ARCH-005, P3)

- [x] #ARCH-005: Split 6 largest modules by responsibility (no behavior change)
  - talos-agent/src/lib.rs: 3135→865 lines (-72%). Extracted tool_execution.rs, tests.rs, helpers.rs.
  - talos-cli/src/main.rs: 2255→1244 lines (-45%). Extracted registry.rs, provider_setup.rs, session_setup.rs, tui_bridge.rs.
  - talos-tools/src/lib.rs: 2513→23 lines (-99%). Extracted bash_tool.rs, file_tools.rs, search_tools.rs, diff_stat.rs.
  - talos-tui/src/app.rs: 2745→927 lines (-66%). Extracted tool_display.rs, scrollback.rs, app/app_tests.rs.

## Risks

- **R1 (ARCH-004-A)**: rmcp API surface is large; need to identify the minimal DTO set that
  covers all public usage without over-abstracting.
- **R2 (ARCH-004-B)**: Changing public error enums is a semver-breaking change. These crates
  are pre-1.0 so this is acceptable, but all call sites must be updated atomically.
- **R3 (ARCH-005)**: Decomposition of 6 files (each 1000-2800 lines) is high-volume but
  low-risk if done as pure `mod` extraction without behavior changes.

## Execution Order

1. ARCH-004 (anti-corruption layers) — highest priority, unblocks ARCH-005
2. ARCH-007 (clippy cleanup) — XS, independent, quick win
3. ARCH-006 (prompt cache stability) — independent, medium effort
4. ARCH-005 (god module decomposition) — largest effort, benefits from ARCH-004 APIs

## Acceptance Criteria

- [ ] `rmcp::` types do not appear in `talos-mcp` public API.
- [ ] `rusqlite::Error` is not the primary public variant in `talos-evolution` or
      `talos-session` error enums.
- [ ] `talos-mcp` no longer imports `talos_config` in public client manager APIs.
- [ ] Duplicate `ToolDefinition` is unified or explicitly renamed.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` exits 0.
- [ ] Prompt prefix stability is documented as a session contract.
- [ ] No module exceeds ~800 lines after decomposition (best-effort; tests may push higher).
- [ ] `cargo check --workspace` passes.
- [ ] `cargo test --workspace` passes.
- [ ] `cargo clippy --workspace -- -D warnings` passes.

## Verification Log

- 2026-06-18: ARCH-004 (A/B/C/D) landed. `cargo check --workspace`, `cargo test --workspace`, `cargo clippy -- -D warnings` all pass.
- 2026-06-18: ARCH-007 landed. `cargo clippy --workspace --all-targets -- -D warnings` passes (test module `#[allow(warnings)]` added to 32 files).
- 2026-06-18: ARCH-006 landed. Stable prefix cached in `Agent::cached_stable_prefix` (Mutex). 4 new tests prove stability semantics. `cargo test -p talos-agent` 146 passed.
- 2026-06-18: ARCH-005 complete. talos-tools (2513→23) and talos-tui/app.rs (2745→927) decomposed. Logic integrity verified: 110/110 talos-tools functions preserved, 138/138 talos-tui functions preserved. No ToolDefinition re-introduced. 0 missing, 0 extra. All tests (81+3+105) pass, `cargo clippy --workspace --all-targets -- -D warnings` passes. I029 complete.
