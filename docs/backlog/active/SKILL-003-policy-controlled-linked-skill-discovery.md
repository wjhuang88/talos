# SKILL-003: Policy-Controlled Linked Skill Discovery

| Field | Value |
| --- | --- |
| Story ID | SKILL-003 |
| Type | Product / Security Story |
| Priority | P2 |
| Status | Refinement → I163 Active (2026-07-27) |
| Selected Iteration | I163 (Active) |
| Source | Commit `1f5e451` introduced unconditional `follow_links(true)` to `SkillLoader::discover` under incorrect `#TUI-035` tag; this Story owns all subsequent corrections |
| Parent Epic | None |
| Depends on | ADR-022 (agent config compatibility boundary) |
| Blocks | None |

## Problem

Users may want to share Skill directories through filesystem links (symlinks on Unix, directory symlinks or junctions on Windows). Commit `1f5e451` enabled this by setting `WalkDir::follow_links(true)` unconditionally, but the implementation has security and correctness gaps:

1. All search paths follow links unconditionally — a symlink in the project repo can silently expand scanning outside the configured trust boundary.
2. `max_depth(32)` limits depth but is not cycle detection; the doc comment incorrectly calls it "built-in cycle detection".
3. No total entry budget — a symlink to a large external directory can cause unbounded traversal.
4. The same physical directory can be scanned repeatedly through multiple link aliases.
5. `Vec::dedup_by_key` only removes adjacent duplicates, not global duplicates.
6. All `WalkDir` errors are silently dropped by `filter_map(|e| e.ok())`.
7. The method doc says errors are "logged" but no logging exists.
8. Windows junction support is claimed without any Windows test evidence.
9. Test names are misleading ("nested symlink chain" creates no symlinks; "cycle" only asserts `>= 1`).
10. Test data has a `tiggers` typo.

## Goal / Value

Allow Skill directory sharing via filesystem links with an explicit, safe-by-default policy. External targets require explicit opt-in. Traversal is bounded. Failures are observable. Cross-platform evidence is real.

## Scope

- `SkillDiscoveryPolicy` struct with `follow_directory_links`, `external_target_policy`, `max_depth`, `max_entries`
- Default: `follow_links = false`, external targets denied, `max_depth = 32`, `max_entries = 10_000`
- Canonical-path deduplication of directories and SKILL.md files
- Global (non-adjacent) skill-name deduplication with first-wins priority
- `SkillDiscoveryWarning` model with structured observability
- Backward-compatible API: existing constructors use safe defaults
- Real Unix symlink tests: chain, alias, cycle, broken link, external target, depth, budget
- Windows tests or honest documentation of unverified status
- Reference path boundary residual documented

## Explicit Exclusions

- CLI configuration wiring (deferred to a follow-up wiring Story)
- Runtime/agent behavior changes
- New native dependencies or `unsafe`
- Public `Skill` struct field changes (canonical path tracked in warnings, not in Skill)

## Design Constraints

- Reuse `WalkDir`; do not introduce a second traversal engine.
- `canonicalize` must handle broken links, permission errors, and unavailable targets gracefully (produce warnings, not panics).
- `SkillSource` semantics unchanged: identifies the discovery-priority root, not the canonical filesystem target.
- Search-root priority preserved: Project > UserGlobal > Parent > Shared.

## Acceptance

- [ ] Default `follow_directory_links = false`; backward compatible with pre-`1f5e451` behavior.
- [ ] `follow_directory_links = true` follows directory symlinks.
- [ ] External targets denied by default; warning recorded.
- [ ] Same canonical directory scanned at most once (alias dedup).
- [ ] Same canonical SKILL.md parsed at most once.
- [ ] Non-adjacent duplicate skill names resolved by first-wins priority.
- [ ] Search-root priority preserved (Project shadows Shared even through aliases).
- [ ] `max_depth` bounds traversal depth; skill at exactly `max_depth` is found, beyond is not.
- [ ] `max_entries` bounds total entries; budget exhaustion produces warning.
- [ ] Broken links produce `BrokenLink` warning.
- [ ] Symlink cycles produce `LinkLoop` warning and do not duplicate skills.
- [ ] Permission errors produce `PermissionDenied` warning.
- [ ] Invalid SKILL.md files produce `InvalidSkill` warning.
- [ ] `discovery_warnings()` returns structured evidence.
- [ ] No `filter_map(|e| e.ok())` in the discovery path.
- [ ] No `dedup_by_key` for skill-name precedence.
- [ ] No `follow_links(true)` hardcoded outside policy control.
- [ ] No `tiggers` typo in test data.
- [ ] No misleading test names.
- [ ] Windows junction claims removed or backed by Windows tests.
- [ ] "at any depth" documentation corrected.
- [ ] No reference to `#TUI-035` in new commits.

## Required Reads

- `crates/talos-skill/src/loader.rs`
- `crates/talos-skill/src/lib.rs`
- `crates/talos-skill/src/types.rs`
- `crates/talos-skill/src/manager.rs`
- `crates/talos-config/src/types.rs`
- `docs/decisions/022-agent-config-compatibility-boundary.md`

## Minimum Validation

- `cargo test --locked -p talos-skill` — all tests pass including new symlink/alias/cycle/broken/external/depth/budget tests
- `cargo fmt --all`, `cargo clippy --workspace --locked -- -D warnings`, `cargo test --workspace --locked`
- `scripts/validate_project_governance.sh .`
- No `follow_links(true)` outside policy control (source scan)
- No `filter_map(|e| e.ok())` in discovery path (source scan)
- No `dedup_by_key` for skill-name precedence (source scan)
