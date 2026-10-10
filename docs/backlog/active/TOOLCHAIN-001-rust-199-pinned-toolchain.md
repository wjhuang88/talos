# TOOLCHAIN-001: Rust 1.99 Pinned Toolchain Maintenance

> Document status: Complete / Closed

| Field | Value |
|---|---|
| Type | Toolchain and CI maintenance |
| Priority | P2 |
| Source | Maintainer-authorized maintenance request |
| Selected Iteration | I296 (Complete; implementation #695 merged as `435b6cf5`) |
| Depends On | Current `main`; effective I296 Collaboration Claim |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | I296/TOOLCHAIN-001: Rust 1.99 pins, matching docs/assertion and compiler-required source compatibility; no MSRV, dependency, lockfile, behavior or release change. |
| Claimed At | 2026-10-10 |
| Source Issue | None |
| Governance Claim PR | #688 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer authorized serial I297/I296 closeout on 2026-10-10. Independent human unavailable; Agent review, exact-head CI, validators and CAS required. #688 merged as `7097f1b5`; review/CAS comment `6093090485` and CI `38018790824` establish the effective claim. |
| Implementation PR | #695 |
| Last Updated | 2026-10-10 |
| Handoff / Release Condition | Implementation merged; owner-first closeout needs its own checks, review and CAS. No release authorized. |

## Scope

- Pin the developer, CI, and release Rust toolchain to Rust 1.99.
- Update the current-toolchain assertion and installation documentation to the same version.
- Address only direct source incompatibilities reported by Rust 1.99.

## Non-Goals

- No Cargo dependency or `Cargo.lock` update.
- No workspace MSRV change; it remains Rust 1.95.
- No release, tag, publication, runtime behavior change, or unrelated cleanup.

## Acceptance

- `rust-toolchain.toml`, CI, release workflow, assertion and installation documentation agree on Rust 1.99.
- The workspace continues to compile, test and lint using the pinned toolchain with locked resolution.
- Direct Rust 1.99 compatibility edits preserve existing behavior and remain compatible with the declared MSRV.

## Planned Validation

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets --locked`
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- `./scripts/release_preflight.sh`
- `./scripts/validate_public_site.sh v0.10.1`
- `./scripts/validate_installers.sh`

## Effective Activation - 2026-10-10

Claim #688 merged as `7097f1b503063539a2205e17ae29aed09c7ae32e` after
exact-head CI, independent Agent review and CAS passed. I296 implementation is
now authorized from that merge; full local release preflight passed, including
workspace tests and the external Runtime SDK fixture. All-target locked check and
all-feature Clippy passed; direct atomic compatibility edits also passed an isolated
Rust 1.95 harness. Exact-head remote CI and independent review remain pending. Earlier claim
wording records the proposal stage, not a remaining activation blocker.

## Residuals

- Windows CI watchdog failure and bounded test-only diagnostic follow-up are
  recorded in I296's Windows Validation Follow-up checkpoint. Production behavior
  is unchanged; final-head Windows acceptance passed in CI `38029259357`.
- Existing third-party future-incompatibility notice for transitive `block v0.1.6` is not changed by this maintenance slice.

## Completion - 2026-10-10

- Completion Commit: 435b6cf5f3cb11ebb34271bd666a614c4bc8092b
- Implementation #695 merged with exact head `63b0c1e4b125cf0f20196ac997d76050c53325dc`
  and base `e63e0ee3793e2e8fd914f676224f982c6cae5593`.
- CI `38029259357`: all six jobs passed. Independent Agent APPROVE is #695
  comment `6094474271`; merge-time CAS is comment `6094762104`.
- Pins, locale assertion and bilingual installation docs agree on Rust 1.99;
  locked full validation passed. Atomic changes preserve behavior and passed an
  isolated Rust 1.95 harness; no full-workspace MSRV test is claimed.
- I296 records the complete inventory, original Windows failure, diagnostic-only
  follow-up, rebase evidence, local disk limitation and #682 serial coordination.
  Earlier pending statements above are historical and superseded by this checkpoint.
- No lockfile, dependency, MSRV, public API, runtime policy or release change.
  Complete/Closed takes effect on main with the separately validated closeout merge.
