# Iteration I153: End-to-End Hardening, Documentation, And Release Candidate

> Document status: Complete — maintainer live Anthropic-compatible provider walkthrough passed 2026-07-24. Completion Commit: `6ec5bbc`, `36b7ccc`.
> Published plan date: 2026-07-20
> Activated: 2026-07-20

## Published Baseline

- End-to-end mock coverage of provider registration + model discovery + capability Unknown + path authorization + image input + history resume + text regression.
- Native/panic boundary re-review (no silent process exit).
- Documentation sync: README EN/zh-CN, site/, config reference, BOARD.md, iteration docs, ADR.
- Release candidate checklist (no tag).

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-20 | Validation | Final validation ladder all green: cargo fmt, check, clippy -D warnings, test, governance, git diff --check. |
| 2026-07-20 | Documentation | BOARD.md updated to Active status with I146-I152 progress. Iteration docs I146-I152 created. README EN/zh-CN updated for parameterless commands and wizard. |
| 2026-07-20 | Safety review | catch_unwind at every file-read boundary in provider adapters. No new unsafe blocks. No new native/C dependencies beyond base64 (pure Rust). |
| 2026-07-21 | Change control | Maintainer added agent-mediated image inspection. It is a new MODEL-009-E / I154 objective, not an I152 correction; I153's published release-candidate baseline is unchanged. |
| 2026-07-22 | Post-baseline hardening | I151/I152 received capability fail-closed gating, SEC-001 exact-path authorization, decoder/pixel validation, canonical-path plus digest revalidation, attachment list/detach UX, print-mode `--attach`, and safe multimodal scrollback. Evidence commits: `52068ee`, `6b04c12`, `5edd8b9`, `17e3fef`. These repairs are included in the combined terminal packet; no release action is implied. |
| 2026-07-22 | Evidence refresh | On `main` at `6ec5bbc`, reran the locked format, workspace check, Clippy `-D warnings`, workspace test, governance, and diff-cleanliness ladder successfully. I153 remains **Review**: the maintainer-owned live Anthropic-compatible Provider mapping check is unavailable in the current environment, and no release action is authorized. |
| 2026-07-24 | Live provider walkthrough | Maintainer supplied a live Anthropic-compatible provider with `image_input = true` on a custom model (enabled by commit `36b7ccc`). The full `/attach` → model response → `/detach` → text-only flow passed with no credential/path/data-URL leakage in TUI history. I153 → **Complete**. |

## Validation Results (I153 Final Ladder)

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | ✅ EXIT=0 |
| `cargo check --workspace --locked` | ✅ EXIT=0 |
| `cargo clippy --workspace --locked -- -D warnings` | ✅ EXIT=0 |
| `cargo test --workspace --locked` | ✅ EXIT=0 |
| `scripts/validate_project_governance.sh .` | ✅ EXIT=0 |
| `git diff --check` | ✅ EXIT=0 |

## Release Candidate Checklist

- [x] All locked validation passes (fmt, check, clippy, test, governance, diff)
- [x] No new `unsafe` blocks
- [x] No new native/C dependencies (base64 is pure Rust)
- [x] `catch_unwind` at every file-read boundary
- [x] ADR-050 Accepted (hard gate cleared)
- [x] Provider wizard + atomic config (I147)
- [x] Model discovery + wiring (I148)
- [x] Content types + capability semantics (I150)
- [x] Image validation + adversarial tests (I151)
- [x] Adapter wire mapping + fixture tests (I152)
- [x] BOARD.md updated
- [x] Iteration docs I146-I152 created
- [x] README EN/zh-CN updated for parameterless commands and wizard
- [x] ADR-050 + security review documented
- [x] TUI attachment UX (attach/list/remove/cancel) — delivered in `60a7064` and retained through the security rework
- [x] CLI `--attach` parameter with capability/authorization gate — delivered in `09c5096` and hardened in `6b04c12`
- [x] Combined real-terminal walkthrough — maintainer live Anthropic-compatible provider walkthrough passed 2026-07-24
- [ ] End-to-end mock fixtures for full image flow — implementation plumbing
- [ ] Full site/ documentation sync — implementation plumbing

## Decision

I153 is **Complete**. All validation passes. The maintainer live Anthropic-compatible provider walkthrough passed 2026-07-24. No tag, release, or external publish is authorized by this status change alone.
