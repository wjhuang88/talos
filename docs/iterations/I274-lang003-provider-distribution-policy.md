# I274 — LANG-003 Provider Migration And Default Distribution

| Field | Value |
|---|---|
| Status | Active / Claimed |
| Source | Issue #517 / CAP-001 / #466 |
| Depends On | LANG-002, BUNDLE-001, DIST-001-A, ADR-076 |
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex unattended single-developer mode |
| Work Slice | Define and implement one additional language provider distribution path with explicit fallback and compatibility evidence; preserve current defaults. |
| Implementation PR | #565 (Review / Claimed) |

## Activation Checkpoint — 2026-09-15

Claim PR #564 is effective after merge commit `6a4a2117` on `main` (exact claim head
`0e7fe73d`, base `b3da1653`, CI run `34965337239`). Implementation must begin from this
merged baseline; the published plan and exclusions remain unchanged.

## Scope

Implement only the first post-Rust language migration selected by the existing provider contract,
its verified Bundle metadata, consumer fixture and plain-text fallback. No default feature inversion,
implicit download, parser deletion, Desktop/Browser work or release changes.

## Acceptance

- Provider identity, compatibility and fallback are recorded and tested.
- TUI and symbol consumers use the shared provider contract.
- Offline verified-Bundle fixture passes; missing/corrupt/incompatible provider degrades safely.
- Static parser and provider asset size evidence is reported separately.
- Existing default `cargo build/run` behavior is unchanged.

## Validation

Focused provider/consumer tests, offline Bundle fixture, no-network startup check, locked checks,
governance validators and exact changed-file inventory before any stable push.
