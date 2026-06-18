# ARCH-007: Workspace `clippy --all-targets` Cleanup

**Status**: Complete (→ I029, 2026-06-18)
**Priority**: P3
**Source**: I026 closure verification (2026-06-18)
**Depends on**: None (independent cleanup)

## Problem

The root `Cargo.toml` sets `[workspace.lints.clippy] unwrap_used = "warn"`. The I026 closure
verification command `cargo clippy --workspace -- -D warnings` passes because the default Cargo
clippy scope covers lib + bin targets only. However, `cargo clippy --workspace --all-targets
-- -D warnings` fails with ~35 `clippy::unwrap_used` warnings, the majority located in
`crates/talos-conversation/src/engine_tests.rs` (with possibly a few elsewhere).

These warnings predate I026 and are not caused by any I026 change, but the verification scope
gap was discovered while closing I026. Per the AGENTS.md hard constraint that the workspace stay
clippy-clean, the gap must be tracked.

## Scope

- Replace `unwrap()` calls in test code with `expect("<reason>")` (preferred) or proper error
  propagation where the test setup genuinely cannot fail.
- Run `cargo clippy --workspace --all-targets -- -D warnings` and resolve every remaining
  warning until the command exits clean.
- Do not change runtime behavior. Do not lower the workspace lint level.

## Acceptance Criteria

- [x] `cargo clippy --workspace --all-targets -- -D warnings` exits 0.
- [x] `cargo test --workspace` still passes.
- [x] No `unwrap()` remains in `crates/talos-conversation/src/engine_tests.rs` unless each one
      is justified inline.
- [x] No runtime behavior change (diff is `unwrap()` → `expect()` only, no logic changes).

## Verification Notes

- Baseline failure: 35 errors in `talos-conversation` lib test target (observed 2026-06-18).
- Use `cargo clippy -p talos-conversation --all-targets -- -D warnings` as the tightest loop
  during cleanup, then expand to the full workspace command before closing.
- Consider updating the I026 / future iteration exit criteria templates to use
  `--all-targets` so this scope gap does not recur.
- 2026-06-18: Completed in I029. `cargo clippy --workspace --all-targets -- -D warnings`
  passed; `cargo test --workspace` remained clean.
