# SKILL-003: Policy-Controlled Linked Skill Discovery

| Field | Value |
| --- | --- |
| Story ID | SKILL-003 |
| Type | Product / Security Story |
| Priority | P2 |
| Status | In Progress (I163 Active, 2026-07-27) |
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

Configuration wiring was originally excluded, but the maintainer explicitly authorized the
application-level `discover_shared` default and runtime wiring during I163. This bounded
exception does not authorize additional Skill configuration expansion.

Runtime/agent behavior changes, new native dependencies or `unsafe`, and public `Skill`
struct field changes remain excluded.

## Design Constraints

- Reuse `WalkDir`; do not introduce a second traversal engine.
- `canonicalize` must handle broken links, permission errors, and unavailable targets gracefully (produce warnings, not panics).
- `SkillSource` semantics unchanged: identifies the discovery-priority root, not the canonical filesystem target.
- Search-root priority preserved: Project > UserGlobal > Parent > Shared.

## Acceptance

### Shared discovery default (maintainer-authorized)

- [x] `Config::default().skills.discover_shared == true`.
- [x] Missing `[skills]` section → true (deserialization test).
- [x] Empty `[skills]` section → true (deserialization test).
- [x] Explicit `discover_shared = false` → false (preserved).
- [x] Explicit `discover_shared = true` → true (preserved).
- [x] Runtime wiring: default Config → `discover_runtime_skills` → `~/.agents/skills/`
  in search paths (temp HOME, no real-HOME pollution).
- [x] Explicit `false` excludes shared root at runtime.
- [x] Shared root is lowest priority (workspace shadows shared end-to-end).
- [x] README synchronized (shared discovery in "Currently shipped", not "Not shipped yet").
- [x] ADR-022 clarification appended; I163/SKILL-003/Program/Board/Backlog synchronized.

### Link safety

- [x] `discover_shared` does not implicitly enable `follow_directory_links`.
- [x] Linked project root rejected by default (`RootLinkDenied`).
- [x] Linked shared root rejected by default (`RootLinkDenied`).
- [x] Root link governed by `SkillDiscoveryPolicy` (pre-walker `symlink_metadata`).
- [x] `AllowAnyReadable` permits linked root when `follow=true`.
- [x] External directory rejected before descent (`WalkDir::filter_entry`).
- [x] Alias subtree descended once (canonical directory dedup at descent time, budget
  proof).
- [x] Root-level `SKILL.md` parsed when `follow=true` (no canonical-dir dedup skip).
- [x] File symlink policy: `follow=false` denies symlinked `SKILL.md`; `follow=true`
  checks canonical file target against `ExternalTargetPolicy`.
- [x] Canonical file dedup: same physical file via two links parsed once.
- [x] Cycle: skill found once, `LinkLoop` warning present, no budget exhaustion.
- [x] `LinkLoop` warning asserted in real cycle test.

### Resource bounds

- [x] `max_depth` definition: search root = depth 0, root/SKILL.md = depth 1.
- [x] Skill at exact `max_depth` discovered.
- [x] Skill beyond `max_depth` not discovered.
- [x] `DepthLimitReached` warning reachable (observation_depth = max_depth + 1).
- [x] Single `DepthLimitReached` warning per root.
- [x] `max_entries` is global across all search roots.
- [x] Unique-name budget test (50 distinct names, budget exhausted once).
- [x] Later roots skipped after budget exhaustion.
- [x] Deterministic ordering (`sort_by_file_name`).

### Tests

- [x] Real two-link chain (actual filesystem symlinks).
- [x] Real cycle (`follow=true`, `LinkLoop` asserted).
- [x] Root link deny/allow (default rejects, `AllowAnyReadable` permits).
- [x] External pre-descent rejection (no `InvalidSkill`, no budget exhaustion).
- [x] Alias subtree single descent (budget proof).
- [x] Direct file link (external denied, internal enabled-only, two-links-once).
- [x] Config deserialization matrix (missing/empty/explicit-true/explicit-false).
- [x] Runtime wiring (Config → `discover_runtime_skills` → SkillLoader).
- [x] Priority end-to-end (workspace shadows shared).
- [x] Misleading tests removed/renamed (`nested_symlink_chain` → `regular_nested_skill`;
  non-following cycle replaced with real `follow=true` cycle).

### Governance

- [x] Maintainer default decision recorded (I163 + ADR-022 clarification).
- [x] `04999f1` scope variance resolved (owner docs synchronized).
- [x] SKILL-003 In Progress.
- [x] I163 Active.
- [x] I163 corrective interruption (between I156 and I157).
- [x] I157 Planned (resumes after I163 disposition).
- [x] I156 Complete.
- [x] Only one Active implementation iteration (I163).
- [x] No release/version changes.

### Residuals

- [x] Windows state honest (Unix: automated; Windows symlink/junction: unverified).
- [x] Level-2 reference-path security residual recorded (separate follow-up Story).
- [x] Public struct-literal compatibility documented accurately (constructor-compatible,
  struct-literal source-incompatible, pre-1.0).
- [x] CI/local evidence distinction accurate (local validation only; no GitHub Actions
  evidence for this iteration).

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
