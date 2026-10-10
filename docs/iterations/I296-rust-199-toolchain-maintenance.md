# Iteration I296: Rust 1.99 Toolchain Maintenance

> Document status: Review
> Published plan date: 2026-10-09
> Planned objective: advance the pinned development, CI and release compiler to Rust 1.99 without changing the Rust 1.95 workspace MSRV or dependency graph.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a locked, runnable workspace using the Rust 1.99 pinned toolchain.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | I296/TOOLCHAIN-001 only: Rust 1.99 development/CI/release pins, matching docs/assertion and compiler-required source compatibility. Preserve MSRV 1.95, lockfile, dependencies, behavior and release version. |
| Claimed At | 2026-10-10 |
| Source Issue | None |
| Governance Claim PR | #688 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer authorized serial I297 then I296 closeout on 2026-10-10. No independent human maintainer is available; independent Agent review, exact-head CI, both validators and merge-time CAS remain mandatory. #688 merged as `7097f1b5`; review/CAS comment `6093090485` and CI `38018790824` establish effective activation. |
| Implementation PR | Pending stable candidate publication |
| Last Updated | 2026-10-10 |
| Handoff / Release Condition | Atomic claim+activation must merge before code/configuration commits are created. |

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| TOOLCHAIN-001 | None | Planned / Unclaimed | Effective I296 claim | Rust 1.99 is the consistent pinned development, CI and release toolchain. |

### Selection Inventory And Disposition

- I293 and I294 are Active / Claimed under separate Search work slices; no overlap.
- I291 and I290 are Review / Claimed; no overlap.
- DEPENDENCY-003-A is Review / Claimed; its `libc` migration is excluded.
- I295 is Complete / Closed; I249 remains deferred Planned; I164 remains Paused.

### Scope

- Update the Rust pin in local development, CI and release configuration to 1.99.
- Keep documentation and the toolchain measurement assertion aligned with that pin.
- Make only compiler-required direct source compatibility corrections.

### Non-Goals

- No package/MSRV/dependency-resolution changes, including `Cargo.lock`.
- No Rust business, permission, sandbox, process-hardening or release behavior change.
- No tag or publication.

### Acceptance

- Given a clean checkout using the pinned toolchain, when locked workspace validation runs, then format, check, Clippy, tests and release preflight pass.
- Given the declared Rust 1.95 workspace MSRV, when the direct compatibility edits are compiled with it, then the edits remain source-compatible.
- Given installation documentation and automation, when they name the current compiler, then they name Rust 1.99.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets --locked`
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- `./scripts/release_preflight.sh`
- `./scripts/validate_public_site.sh v0.10.1`
- `./scripts/validate_installers.sh`
- `scripts/validate_project_governance.sh .`
- `bash scripts/validate_collaboration_claims.sh .`

### Documentation To Update

- `rust-toolchain.toml`, CI and release workflow pins.
- `site/install.html`, `site/zh/install.html`, and the version assertion used by the locale measurement script.
- TOOLCHAIN-001 owner, I296 execution evidence, Backlog, iteration index and Board.

### Risks And Rollback

- Risk: Rust 1.99 identifies source/API incompatibilities or exposes a dependency future-incompatibility notice.
- Rollback: retain the existing pin and route any behavior-changing repair to a separately governed owner.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-10-09 | Proposed selection | Local candidate exploration passed the listed checks, but is uncommitted and non-authoritative until the atomic claim+activation reaches `main`. |

## Verification Evidence

- Pre-claim local exploration: candidate passed all planned validation commands, including full `./scripts/release_preflight.sh`; this is not implementation evidence until recreated from the effective claim merge.

## Completion Evidence

- Completion Commit: pending

## Atomic Activation Checkpoint - 2026-10-10

This candidate closes I297 owner-first using existing main implementation
`97b29bd8c0762b780c4ab97b9bd42fd9cc106303` and then proposes I296 activation.
No I296 implementation is included. The Published Baseline remains unchanged.
Target main is `97b29bd8`; #694 has all six CI jobs successful, Agent APPROVE
`6092606598` and CAS `6092949030`.

Current inventory: I293/I294 Active, I277/I290/I291 Review, I249 Planned/deferred,
I164 Paused, I295/I298 Complete: retain their ownership and non-overlapping scopes.
I297 is Complete in this proposed closeout; it becomes terminal on main when this
candidate merges. Open #682 and #693 own Auto locale and synthetic Search evidence;
neither changes toolchain pins. No blocked owner is selected or overridden.

Implementation starts only from this claim merge or later main. Recreate validation
from that baseline; pre-claim experiments are not completion evidence. Preserve both
existing stashes and other-session work. No release/tag/publication is authorized.

## Effective Activation And Local Implementation - 2026-10-10

#688 merged as `7097f1b503063539a2205e17ae29aed09c7ae32e`; the claim and
activation are now effective. Independent Agent APPROVE and merge-time CAS are
recorded in comment `6093090485`; exact-head CI `38018790824` passed all
applicable checks. Implementation branch `chore/i296-rust-199` starts from that
merge. Published Baseline and earlier proposed checkpoints remain unchanged.

The implementation updates compiler pins and matching documentation/assertion.
Validation is in progress on the real pinned Rust 1.99.0 toolchain. No completion
or release is claimed. Existing exploration and provider-experiment stashes are
preserved. Authorized Cargo cache cleanup removed 10.6 GiB before rebuilding.

### Local Verification Checkpoint - 2026-10-10

- `rustc --version`: `rustc 1.99.0 (b940084d7 2026-09-28)`.
- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace --all-targets --locked`: passed.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`: passed.
- Rust 1.99 initially reported deprecated `AtomicU64::fetch_update`. Two direct
  replacements use its renamed `try_update` with unchanged closures, orderings and
  result handling. A standalone Rust 1.95.0 harness compiled and executed both
  closure/order combinations, including approval-ID overflow rejection.
- `python3 scripts/measure_auto_locale.py --self-test`: six tests passed on Rust 1.99.
- `./scripts/validate_public_site.sh v0.10.1`: 16 HTML files, zero errors/warnings.
- `./scripts/validate_installers.sh`: zero errors.
- Both governance validators passed with zero warnings during preflight; text
  boundary validation and all 14 CI classifier tests passed.
- `scripts/assess_project_scale.sh .`: high-risk / release-managed / on-demand,
  one worktree; no profile change required.
- Complete `./scripts/release_preflight.sh` passed (exit 0), including workspace
  tests and the independently rooted external Runtime SDK fixture with locked resolution.
- Independent read-only Agent inspection found no code blockers; stable-head
  approval and remote CI remain pending. Cargo.lock, MSRV and release version
  remain unchanged. Delivery is Review / Claimed; remote gates remain pending.

## Variance And Residuals

- Existing transitive `block v0.1.6` future-incompatibility notice remains outside scope.
